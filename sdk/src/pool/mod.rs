// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::iter::Zip;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::vec::IntoIter;

use async_utility::futures_util::{future, StreamExt};
use async_utility::task;
use atomic_destructor::{AtomicDestructor, StealthClone};
use nostr_database::prelude::*;
use tokio::sync::{broadcast, mpsc, Mutex, RwLockReadGuard};

pub mod builder;
pub mod constants;
mod error;
mod inner;
pub mod options;
mod output;

pub use self::builder::RelayPoolBuilder;
pub use self::error::Error;
use self::inner::{InnerRelayPool, Relays};
pub use self::options::RelayPoolOptions;
pub use self::output::Output;
use crate::client::ClientNotification;
use crate::monitor::Monitor;
use crate::relay::capabilities::RelayCapabilities;
use crate::relay::options::{RelayOptions, ReqExitPolicy, SubscribeOptions, SyncOptions};
use crate::relay::{
    self, AtomicRelayCapabilities, Reconciliation, Relay, SubscribeAutoCloseOptions,
};
use crate::shared::SharedState;
use crate::stream::{BoxedStream, ReceiverStream};

/// Relay Pool
#[derive(Debug, Clone)]
pub struct RelayPool {
    inner: AtomicDestructor<InnerRelayPool>,
}

impl Default for RelayPool {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl StealthClone for RelayPool {
    fn stealth_clone(&self) -> Self {
        Self {
            inner: self.inner.stealth_clone(),
        }
    }
}

impl RelayPool {
    /// Construct new default relay pool
    ///
    /// Use [`RelayPool::builder`] to customize it.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// New relay pool builder
    #[inline]
    pub fn builder() -> RelayPoolBuilder {
        RelayPoolBuilder::default()
    }

    #[inline]
    fn from_builder(builder: RelayPoolBuilder) -> Self {
        Self {
            inner: AtomicDestructor::new(InnerRelayPool::from_builder(builder)),
        }
    }

    /// Check if the relay pool is shutdown.
    #[inline]
    pub fn is_shutdown(&self) -> bool {
        self.inner.atomic.shutdown.load(Ordering::SeqCst)
    }

    /// Completely shutdown pool
    ///
    /// This method disconnects and removes all relays from the [`RelayPool`] and then
    /// sends [`ClientNotification::Shutdown`] notification.
    ///
    /// After this method has been called, the [`RelayPool`] can no longer be used (i.e. can't add relays).
    #[inline]
    pub async fn shutdown(&self) {
        self.inner.shutdown().await
    }

    pub(crate) fn notifications(&self) -> broadcast::Receiver<ClientNotification> {
        self.inner.notification_sender.subscribe()
    }

    /// Returns the reference to the monitor, if any.
    ///
    /// Returns `None` if the monitor is not configured (see [`RelayPoolBuilder::monitor`] ).
    pub fn monitor(&self) -> Option<&Monitor> {
        self.inner.state.monitor.as_ref()
    }

    /// Get shared state
    #[inline]
    pub fn state(&self) -> &SharedState {
        &self.inner.state
    }

    /// Get database
    #[inline]
    pub fn database(&self) -> &Arc<dyn NostrDatabase> {
        self.inner.state.database()
    }

    fn internal_relays_with_any_cap<'a>(
        &self,
        txn: &'a RwLockReadGuard<'a, Relays>,
        capabilities: RelayCapabilities,
    ) -> impl Iterator<Item = (&'a RelayUrl, &'a Relay)> + 'a {
        txn.iter()
            .filter(move |(_, r)| r.capabilities().has_any(capabilities))
    }

    pub(crate) async fn relay_urls_with_any_cap(
        &self,
        capabilities: RelayCapabilities,
    ) -> Vec<RelayUrl> {
        let relays = self.inner.atomic.relays.read().await;
        self.internal_relays_with_any_cap(&relays, capabilities)
            .map(|(k, ..)| k.clone())
            .collect()
    }

    pub(crate) async fn read_relay_urls(&self) -> Vec<RelayUrl> {
        self.relay_urls_with_any_cap(RelayCapabilities::READ).await
    }

    pub(crate) async fn write_relay_urls(&self) -> Vec<RelayUrl> {
        self.relay_urls_with_any_cap(RelayCapabilities::WRITE).await
    }

    // Get **all** relays
    pub(crate) async fn all_relays(&self) -> HashMap<RelayUrl, Relay> {
        let relays = self.inner.atomic.relays.read().await;
        relays.clone()
    }

    // Get relays that have any of the specified [`RelayCapabilities`]
    pub(crate) async fn relays_with_any_cap(
        &self,
        capabilities: RelayCapabilities,
    ) -> HashMap<RelayUrl, Relay> {
        let relays = self.inner.atomic.relays.read().await;
        self.internal_relays_with_any_cap(&relays, capabilities)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    #[inline]
    fn internal_relay<'a>(
        &self,
        txn: &'a RwLockReadGuard<'a, Relays>,
        url: &RelayUrl,
    ) -> Result<&'a Relay, Error> {
        txn.get(url).ok_or(Error::RelayNotFound)
    }

    /// Get relay
    pub async fn relay<'a, U>(&self, url: U) -> Result<Relay, Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        let url = url.into().try_into_relay_url()?;
        let relays = self.inner.atomic.relays.read().await;
        self.internal_relay(&relays, &url).cloned()
    }

    #[inline]
    pub(crate) async fn add_relay(
        &self,
        url: Cow<'_, RelayUrl>,
        capabilities: RelayCapabilities,
        connect: bool,
        opts: RelayOptions,
    ) -> Result<bool, Error> {
        // Check if the pool has been shutdown
        if self.is_shutdown() {
            return Err(Error::Shutdown);
        }

        // Get relays
        let mut relays = self.inner.atomic.relays.write().await;

        // Check if the relay already exists
        if let Some(relay) = relays.get(&url) {
            // Add capabilities to the existing relay
            let current_capabilities: &AtomicRelayCapabilities = relay.capabilities();
            current_capabilities.add(capabilities);

            // Return
            return Ok(false);
        }

        // Check number fo relays and limit
        if let Some(max) = self.inner.opts.max_relays {
            if relays.len() >= max {
                return Err(Error::TooManyRelays { limit: max });
            }
        }

        // Get owned url
        let url: RelayUrl = url.into_owned();

        // Compose new relay
        let mut relay: Relay =
            Relay::new(url.clone(), self.inner.state.clone(), capabilities, opts);

        // Set notification sender
        relay
            .inner
            .set_notification_sender(self.inner.notification_sender.clone());

        // If relay has `READ` capability, inherit pool subscriptions
        if relay.capabilities().has_read() {
            let subscriptions = self.inner.atomic.inherit_subscriptions.read().await;
            for (id, filter) in subscriptions.iter() {
                relay
                    .inner
                    .update_subscription(id.clone(), filter.clone(), false)
                    .await;
            }
        }

        // Connect
        if connect {
            relay.connect();
        }

        // Insert relay into map
        relays.insert(url, relay);

        Ok(true)
    }

    pub(crate) async fn remove_relay(
        &self,
        url: Cow<'_, RelayUrl>,
        force: bool,
    ) -> Result<(), Error> {
        // Acquire write lock
        let mut relays = self.inner.atomic.relays.write().await;

        // Remove relay
        let relay: Relay = relays.remove(&url).ok_or(Error::RelayNotFound)?;

        // If NOT force, check if it has `GOSSIP` capability
        if !force {
            // If can't be removed, re-insert it.
            if !can_remove_relay(&relay) {
                relays.insert(url.into_owned(), relay);
                return Ok(());
            }
        }

        // Disconnect
        relay.disconnect();

        Ok(())
    }

    #[inline]
    pub(crate) async fn remove_all_relays(&self, force: bool) {
        self.inner.remove_all_relays(force).await
    }

    pub(crate) async fn connect(&self) {
        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Connect
        for relay in relays.values() {
            relay.connect()
        }
    }

    pub(crate) async fn wait_for_connection(&self, timeout: Duration) {
        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Compose futures
        let mut futures = Vec::with_capacity(relays.len());
        for relay in relays.values() {
            futures.push(relay.wait_for_connection(timeout));
        }

        // Join futures
        future::join_all(futures).await;
    }

    pub(crate) async fn try_connect(&self, timeout: Duration) -> Output<()> {
        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(relays.len());
        let mut futures = Vec::with_capacity(relays.len());
        let mut output: Output<()> = Output::default();

        // Filter only relays that can connect and compose futures
        for relay in relays.values().filter(|r| r.status().can_connect()) {
            urls.push(relay.url().clone());
            futures.push(relay.try_connect(timeout));
        }

        // TODO: use semaphore to limit number concurrent connections?

        // Join futures
        let list = future::join_all(futures).await;

        // Iterate results and compose output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(..) => {
                    output.success.insert(url);
                }
                Err(e) => {
                    output.failed.insert(url, e.to_string());
                }
            }
        }

        output
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) {
        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Iter values and disconnect
        for relay in relays.values() {
            relay.disconnect();
        }
    }

    /// Connect to a previously added relay
    ///
    /// This method doesn't provide any information on if the connection was successful or not.
    ///
    /// Return [`Error::RelayNotFound`] if the relay doesn't exist in the pool.
    pub async fn connect_relay<'a, U>(&self, url: U) -> Result<(), Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        // Convert url
        let url = url.into().try_into_relay_url()?;

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Get relay
        let relay: &Relay = self.internal_relay(&relays, &url)?;

        // Connect
        relay.connect();

        Ok(())
    }

    /// Try to connect to a previously added relay
    ///
    /// For further details, see the documentation of [`Relay::try_connect`].
    pub async fn try_connect_relay<'a, U>(&self, url: U, timeout: Duration) -> Result<(), Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        // Convert url
        let url: Cow<RelayUrl> = url.into().try_into_relay_url()?;

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Get relay
        let relay: &Relay = self.internal_relay(&relays, &url)?;

        // Try to connect
        relay.try_connect(timeout).await?;

        Ok(())
    }

    /// Disconnect relay
    pub async fn disconnect_relay<'a, U>(&self, url: U) -> Result<(), Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        // Convert url
        let url: Cow<RelayUrl> = url.into().try_into_relay_url()?;

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Get relay
        let relay: &Relay = self.internal_relay(&relays, &url)?;

        // Disconnect
        relay.disconnect();

        Ok(())
    }

    /// Get subscriptions
    #[inline]
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, HashMap<RelayUrl, Vec<Filter>>> {
        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        let mut subscriptions: HashMap<SubscriptionId, HashMap<RelayUrl, Vec<Filter>>> =
            HashMap::new();

        for (url, relay) in relays.iter() {
            // Get relay subscriptions
            let relay_subscriptions = relay.subscriptions().await;

            // Iterate relay subscriptions and populate the general subscriptions map
            for (id, list) in relay_subscriptions.into_iter() {
                subscriptions
                    .entry(id)
                    .or_default()
                    .insert(url.clone(), list);
            }
        }

        subscriptions
    }

    /// Get a subscription
    #[inline]
    pub async fn subscription(&self, id: &SubscriptionId) -> HashMap<RelayUrl, Vec<Filter>> {
        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        let mut filters: HashMap<RelayUrl, Vec<Filter>> = HashMap::new();

        // Iterate relays and populate filters
        for (url, relay) in relays.iter() {
            // try to get subscription by ID from the relay
            if let Some(list) = relay.subscription(id).await {
                filters.insert(url.clone(), list);
            }
        }

        filters
    }

    /// Register subscription in the [`RelayPool`].
    ///
    /// When a new relay is added, saved subscriptions will be automatically used for it.
    #[inline]
    pub async fn save_subscription<F>(&self, id: SubscriptionId, filters: F)
    where
        F: Into<Vec<Filter>>,
    {
        let mut subscriptions = self.inner.atomic.inherit_subscriptions.write().await;
        let current: &mut Vec<Filter> = subscriptions.entry(id).or_default();
        *current = filters.into();
    }

    async fn remove_subscription(&self, id: &SubscriptionId) {
        let mut subscriptions = self.inner.atomic.inherit_subscriptions.write().await;
        subscriptions.remove(id);
    }

    async fn remove_all_subscriptions(&self) {
        let mut subscriptions = self.inner.atomic.inherit_subscriptions.write().await;
        subscriptions.clear();
    }

    /// Send a client message to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn send_msg_to<'a, I, U>(
        &self,
        urls: I,
        msg: ClientMessage<'_>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        self.batch_msg_to(urls, vec![msg]).await
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn batch_msg_to<'a, I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage<'_>>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        // Compose URLs
        let set: HashSet<Cow<RelayUrl>> = urls
            .into_iter()
            .map(|u| u.into().try_into_relay_url())
            .collect::<Result<_, _>>()?;

        // Check if urls set is empty
        if set.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // Check if urls set contains ONLY already added relays
        if !set.iter().all(|url| relays.contains_key(url)) {
            return Err(Error::RelayNotFound);
        }

        // Save events
        for msg in msgs.iter() {
            if let ClientMessage::Event(event) = msg {
                self.inner.state.database().save_event(event).await?;
            }
        }

        let mut output: Output<()> = Output::default();

        // Batch messages and construct outputs
        for url in set.into_iter().map(|r| r.into_owned()) {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            match relay.batch_msg(msgs.clone()) {
                Ok(..) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                }
                Err(e) => {
                    output.failed.insert(url, e.to_string());
                }
            }
        }

        Ok(output)
    }

    pub(crate) async fn send_event<'a, I, U>(
        &self,
        urls: I,
        event: &Event,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        // Compose URLs
        let set: HashSet<Cow<RelayUrl>> = urls
            .into_iter()
            .map(|u| u.into().try_into_relay_url())
            .collect::<Result<_, _>>()?;

        // Check if urls set is empty
        if set.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(set.len());
        let mut futures = Vec::with_capacity(set.len());
        let mut output: Output<EventId> = Output {
            val: event.id,
            success: HashSet::new(),
            failed: HashMap::new(),
        };

        // Compose futures
        for url in set.into_iter() {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            urls.push(url.into_owned());
            futures.push(relay.send_event(event));
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Iter results and construct output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(id) => {
                    // The ID must match
                    assert_eq!(id, event.id);

                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                }
                Err(e) => {
                    output.failed.insert(url, e.to_string());
                }
            }
        }

        Ok(output)
    }

    pub(crate) async fn subscribe(
        &self,
        filters: HashMap<RelayUrl, Vec<Filter>>,
        id: Option<SubscriptionId>,
        auto_close: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<SubscriptionId>, Error> {
        // Check if urls set is empty
        if filters.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(filters.len());
        let mut futures = Vec::with_capacity(filters.len());

        // Get ID
        let id: SubscriptionId = id.unwrap_or_else(SubscriptionId::generate);

        // Compose futures
        for (url, filter) in filters.into_iter() {
            // Get relay
            let relay: &Relay = relays.get(&url).ok_or(Error::RelayNotFound)?;

            // Prepare
            let id: SubscriptionId = id.clone();
            let opts: SubscribeOptions = SubscribeOptions::default().close_on(auto_close);

            // Create future
            urls.push(url);
            futures.push(relay.subscribe_with_id(id, filter, opts));
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Create an empty output
        let mut output: Output<SubscriptionId> = Output::new(id);

        // Iter results and construct output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(..) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                }
                Err(e) => {
                    output.failed.insert(url, e.to_string());
                }
            }
        }

        Ok(output)
    }

    /// Unsubscribe from subscription
    pub async fn unsubscribe(&self, id: &SubscriptionId) {
        // Remove subscription from pool
        self.remove_subscription(id).await;

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // TODO: use join_all and return `Output`?

        // Remove subscription from relays
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe(id).await {
                tracing::error!("{e}");
            }
        }
    }

    /// Unsubscribe from all subscriptions
    pub async fn unsubscribe_all(&self) {
        // Remove subscriptions from pool
        self.remove_all_subscriptions().await;

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // TODO: use join_all and return `Output`?

        // Unsubscribe relays
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe_all().await {
                tracing::error!("{e}");
            }
        }
    }

    pub(crate) async fn sync(
        &self,
        targets: HashMap<RelayUrl, (Filter, Vec<(EventId, Timestamp)>)>,
        opts: SyncOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        // Check if urls set is empty
        if targets.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // TODO: shared reconciliation output to avoid to request duplicates?

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(targets.len());
        let mut futures = Vec::with_capacity(targets.len());
        let mut output: Output<Reconciliation> = Output::default();

        // Compose futures
        for (url, (filter, items)) in targets.into_iter() {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            urls.push(url);
            futures.push(relay.sync_with_items(filter, items, &opts));
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Iter results and constructs output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(reconciliation) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                    output.merge(reconciliation);
                }
                Err(e) => {
                    output.failed.insert(url, e.to_string());
                }
            }
        }

        Ok(output)
    }

    pub(crate) async fn stream_events(
        &self,
        filters: HashMap<RelayUrl, Vec<Filter>>,
        timeout: Option<Duration>,
        policy: ReqExitPolicy,
    ) -> Result<BoxedStream<(RelayUrl, Result<Event, relay::Error>)>, Error> {
        // Check if `targets` map is empty
        if filters.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Create a new channel
        // NOTE: the events are deduplicated and the send method awaits, so a huge capacity isn't necessary.
        let (tx, rx) = mpsc::channel(1024);

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(filters.len());
        let mut futures = Vec::with_capacity(filters.len());

        for (url, filter) in filters {
            // Try to get the relay
            let relay: &Relay = self.internal_relay(&relays, &url)?;

            // Push url
            urls.push(url);

            // Push stream events future
            futures.push(relay.stream_events(filter, timeout, policy));
        }

        // Wait that futures complete
        let awaited = future::join_all(futures).await;

        // The urls and futures len MUST be the same!
        assert_eq!(urls.len(), awaited.len());

        // Zip-up urls and futures into a single iterator
        let streams: Zip<IntoIter<RelayUrl>, IntoIter<Result<_, _>>> =
            urls.into_iter().zip(awaited.into_iter());

        // Single driver task: polls all streams, de-duplicates, forwards
        task::spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            type OutFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
            #[cfg(target_arch = "wasm32")]
            type OutFuture = Pin<Box<dyn Future<Output = ()>>>;

            // IDs collection, needed to check if an event was already sent to the stream
            let ids: Arc<Mutex<HashSet<EventId>>> = Arc::new(Mutex::new(HashSet::new()));

            let mut futures: Vec<OutFuture> = Vec::with_capacity(streams.len());

            for (url, res) in streams.into_iter() {
                let tx = tx.clone();

                let future: OutFuture = match res {
                    // Streaming available
                    Ok(mut stream) => {
                        let ids = ids.clone();

                        Box::pin(async move {
                            // Start handling stream items
                            while let Some(res) = stream.next().await {
                                match res {
                                    Ok(event) => {
                                        let mut ids = ids.lock().await;

                                        // Check if ID was already seen or insert into set.
                                        if ids.insert(event.id) {
                                            // Immediately drop the set
                                            drop(ids);

                                            // Send event
                                            let _ = tx.send((url.clone(), Ok(event))).await;
                                        }
                                    }
                                    Err(e) => {
                                        // Send error
                                        let _ = tx.send((url.clone(), Err(e))).await;
                                    }
                                }
                            }
                        })
                    }
                    // No streaming available
                    Err(e) => {
                        Box::pin(async move {
                            // Send error
                            let _ = tx.send((url, Err(e))).await;
                        })
                    }
                };

                futures.push(future);
            }

            // Wait that all futures complete
            future::join_all(futures).await;

            // Close the channel
            drop(tx);
        });

        // Return stream
        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

/// Return `true` if the relay can be removed
///
/// If it CAN'T be removed,
/// the capabilities are automatically updated (remove `READ`, `WRITE` and `DISCOVERY` capabilities).
fn can_remove_relay(relay: &Relay) -> bool {
    let capabilities = relay.capabilities();
    if capabilities.has_any(RelayCapabilities::GOSSIP) {
        // Remove READ, WRITE and DISCOVERY capabilities
        capabilities.remove(
            RelayCapabilities::READ | RelayCapabilities::WRITE | RelayCapabilities::DISCOVERY,
        );

        // Relay has `GOSSIP` capability so it can't be removed.
        return false;
    }

    // Relay can be removed
    true
}
