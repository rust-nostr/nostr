// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use async_utility::futures_util::stream::BoxStream;
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
use crate::monitor::Monitor;
use crate::relay::flags::FlagCheck;
use crate::relay::options::{RelayOptions, ReqExitPolicy, SyncOptions};
use crate::relay::Relay;
use crate::shared::SharedState;
use crate::stream::ReceiverStream;
use crate::{Reconciliation, RelayServiceFlags, SubscribeOptions};

/// Relay Pool Notification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayPoolNotification {
    /// Received a new [`Event`] from a relay.
    ///
    /// This notification is sent only the **first time** the [`Event`] is seen.
    /// Events sent by this client are not included.
    /// This is useful when you only need to process new incoming events
    /// and avoid handling the same events multiple times.
    ///
    /// If you require notifications for all messages, including previously sent or received events,
    /// consider using the [`RelayPoolNotification::Message`] variant instead.
    Event {
        /// The URL of the relay from which the event was received.
        relay_url: RelayUrl,
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// The received event.
        event: Box<Event>,
    },
    /// Received a [`RelayMessage`].
    ///
    /// This notification is sent **every time** a [`RelayMessage`] is received,
    /// regardless of whether it has been received before.
    ///
    /// May includes messages wrapping events that were sent by this client.
    Message {
        /// The URL of the relay from which the message was received.
        relay_url: RelayUrl,
        /// The received relay message.
        message: RelayMessage<'static>,
    },
    /// Shutdown
    ///
    /// This notification variant is sent after [`RelayPool::shutdown`] method is called and all connections have been closed.
    Shutdown,
}

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
    /// sends [`RelayPoolNotification::Shutdown`] notification.
    ///
    /// After this method has been called, the [`RelayPool`] can no longer be used (i.e. can't add relays).
    #[inline]
    pub async fn shutdown(&self) {
        self.inner.shutdown().await
    }

    /// Get new **pool** notification listener
    ///
    /// <div class="warning">When you call this method, you subscribe to the notifications channel from that precise moment. Anything received by relay/s before that moment is not included in the channel!</div>
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
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

    fn internal_relays_with_flag<'a>(
        &self,
        txn: &'a RwLockReadGuard<'a, Relays>,
        flag: RelayServiceFlags,
        check: FlagCheck,
    ) -> impl Iterator<Item = (&'a RelayUrl, &'a Relay)> + 'a {
        txn.iter().filter(move |(_, r)| r.flags().has(flag, check))
    }

    /// Get relay URLs with specific flag/s
    #[doc(hidden)]
    pub async fn __relay_urls_with_flag(
        &self,
        flag: RelayServiceFlags,
        check: FlagCheck,
    ) -> Vec<RelayUrl> {
        let relays = self.inner.atomic.relays.read().await;
        self.internal_relays_with_flag(&relays, flag, check)
            .map(|(k, ..)| k.clone())
            .collect()
    }

    /// Get relays with `READ` or `WRITE` relays
    #[doc(hidden)]
    pub async fn __relay_urls(&self) -> Vec<RelayUrl> {
        self.__relay_urls_with_flag(
            RelayServiceFlags::READ | RelayServiceFlags::WRITE,
            FlagCheck::Any,
        )
        .await
    }

    /// Get only READ relays
    #[doc(hidden)]
    pub async fn __read_relay_urls(&self) -> Vec<RelayUrl> {
        self.__relay_urls_with_flag(RelayServiceFlags::READ, FlagCheck::All)
            .await
    }

    /// Get only WRITE relays
    #[doc(hidden)]
    pub async fn __write_relay_urls(&self) -> Vec<RelayUrl> {
        self.__relay_urls_with_flag(RelayServiceFlags::WRITE, FlagCheck::All)
            .await
    }

    /// Get all relays
    ///
    /// This method returns all relays added to the pool, including the ones for gossip protocol or other services.
    pub async fn all_relays(&self) -> HashMap<RelayUrl, Relay> {
        let relays = self.inner.atomic.relays.read().await;
        relays.clone()
    }

    /// Get relays with `READ` or `WRITE` flags
    pub async fn relays(&self) -> HashMap<RelayUrl, Relay> {
        self.relays_with_flag(
            RelayServiceFlags::READ | RelayServiceFlags::WRITE,
            FlagCheck::Any,
        )
        .await
    }

    /// Get relays that have a certain [`RelayServiceFlags`] enabled
    pub async fn relays_with_flag(
        &self,
        flag: RelayServiceFlags,
        check: FlagCheck,
    ) -> HashMap<RelayUrl, Relay> {
        let relays = self.inner.atomic.relays.read().await;
        self.internal_relays_with_flag(&relays, flag, check)
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
    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: RelayUrl = url.try_into_url()?;
        let relays = self.inner.atomic.relays.read().await;
        self.internal_relay(&relays, &url).cloned()
    }

    /// Add new relay
    ///
    /// If the [`RelayServiceFlags::READ`] flag is set in [`RelayOptions`]
    /// and the pool has some subscriptions, the new added relay will inherit them.
    /// Use [`RelayPool::subscribe_to`] method instead of [`RelayPool::subscribe`],
    /// to avoid setting pool subscriptions.
    ///
    /// Connection is **NOT** automatically started, remember to call [`RelayPool::connect`] or [`RelayPool::connect_relay`]!
    #[inline]
    pub async fn add_relay<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert into url
        let url: RelayUrl = url.try_into_url()?;

        // Check if the pool has been shutdown
        if self.is_shutdown() {
            return Err(Error::Shutdown);
        }

        // Get relays
        let mut relays = self.inner.atomic.relays.write().await;

        // Check if map already contains url
        if relays.contains_key(&url) {
            return Ok(false);
        }

        // Check number fo relays and limit
        if let Some(max) = self.inner.opts.max_relays {
            if relays.len() >= max {
                return Err(Error::TooManyRelays { limit: max });
            }
        }

        // Compose new relay
        let mut relay: Relay = Relay::new(url, self.inner.state.clone(), opts);

        // Set notification sender
        relay
            .inner
            .set_notification_sender(self.inner.notification_sender.clone());

        // If relay has `READ` flag, inherit pool subscriptions
        if relay.flags().has_read() {
            let subscriptions = self.inner.atomic.inherit_subscriptions.read().await;
            for (id, filter) in subscriptions.iter() {
                relay
                    .inner
                    .update_subscription(id.clone(), filter.clone(), false)
                    .await;
            }
        }

        // Insert relay into map
        relays.insert(relay.url().clone(), relay);

        Ok(true)
    }

    // Private API
    //
    // Try to get relay by `url` or add it to pool.
    // Return `Some(..)` only if the relay already exists.
    #[inline]
    #[doc(hidden)]
    pub async fn __get_or_add_relay(
        &self,
        url: RelayUrl,
        opts: RelayOptions,
    ) -> Result<Option<Relay>, Error> {
        match self.relay(&url).await {
            Ok(relay) => Ok(Some(relay)),
            Err(..) => {
                self.add_relay(url, opts).await?;
                Ok(None)
            }
        }
    }

    async fn _remove_relay<U>(&self, url: U, force: bool) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert into url
        let url: RelayUrl = url.try_into_url()?;

        // Acquire write lock
        let mut relays = self.inner.atomic.relays.write().await;

        // Remove relay
        let relay: Relay = relays.remove(&url).ok_or(Error::RelayNotFound)?;

        // If NOT force, check if it has `GOSSIP` flag
        if !force {
            // If can't be removed, re-insert it.
            if !can_remove_relay(&relay) {
                relays.insert(url, relay);
                return Ok(());
            }
        }

        // Disconnect
        relay.disconnect();

        Ok(())
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has [`RelayServiceFlags::GOSSIP`], it will not be removed from the pool and its
    /// flags will be updated (remove [`RelayServiceFlags::READ`],
    /// [`RelayServiceFlags::WRITE`] and [`RelayServiceFlags::DISCOVERY`] flags).
    ///
    /// To fore remove a relay use [`RelayPool::force_remove_relay`].
    #[inline]
    pub async fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self._remove_relay(url, false).await
    }

    /// Force remove and disconnect relay
    ///
    /// Note: this method will remove the relay, also if it's in use for the gossip model or other service!
    #[inline]
    pub async fn force_remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self._remove_relay(url, true).await
    }

    /// Disconnect and remove all relays
    ///
    /// This method may not remove all relays.
    /// Use [`RelayPool::force_remove_all_relays`] to remove every relay.
    #[inline]
    pub async fn remove_all_relays(&self) {
        // Acquire write lock
        let mut relays = self.inner.atomic.relays.write().await;

        // Retains all relays that can't be removed
        relays.retain(|_, r| !can_remove_relay(r));
    }

    /// Disconnect and force remove all relays
    #[inline]
    pub async fn force_remove_all_relays(&self) {
        self.inner.force_remove_all_relays().await
    }

    /// Connect to all added relays
    ///
    /// Attempts to initiate a connection for every relay currently in
    /// [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`].
    /// A background connection task is spawned for each such relay, which then tries
    /// to establish the connection.
    /// Any relay not in one of these two statuses is skipped.
    ///
    /// For further details, see the documentation of [`Relay::connect`].
    ///
    /// [`RelayStatus::Initialized`]: crate::relay::RelayStatus::Initialized
    /// [`RelayStatus::Terminated`]: crate::relay::RelayStatus::Terminated
    pub async fn connect(&self) {
        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Connect
        for relay in relays.values() {
            relay.connect()
        }
    }

    /// Waits for relays connections
    ///
    /// Wait for relays connections at most for the specified `timeout`.
    /// The code continues when the relays are connected or the `timeout` is reached.
    pub async fn wait_for_connection(&self, timeout: Duration) {
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

    /// Try to establish a connection with the relays.
    ///
    /// Attempts to establish a connection for every relay currently in
    /// [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`]
    /// without spawning the connection task if it fails.
    /// This means that if the connection fails, no automatic retries are scheduled.
    /// Use [`RelayPool::connect`] if you want to immediately spawn a connection task,
    /// regardless of whether the initial connection succeeds.
    ///
    /// For further details, see the documentation of [`Relay::try_connect`].
    ///
    /// [`RelayStatus::Initialized`]: crate::relay::RelayStatus::Initialized
    /// [`RelayStatus::Terminated`]: crate::relay::RelayStatus::Terminated
    pub async fn try_connect(&self, timeout: Duration) -> Output<()> {
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
    pub async fn connect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert url
        let url: RelayUrl = url.try_into_url()?;

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
    pub async fn try_connect_relay<U>(&self, url: U, timeout: Duration) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert url
        let url: RelayUrl = url.try_into_url()?;

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Get relay
        let relay: &Relay = self.internal_relay(&relays, &url)?;

        // Try to connect
        relay.try_connect(timeout).await?;

        Ok(())
    }

    /// Disconnect relay
    pub async fn disconnect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert url
        let url: RelayUrl = url.try_into_url()?;

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
    pub async fn send_msg_to<I, U>(
        &self,
        urls: I,
        msg: ClientMessage<'_>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.batch_msg_to(urls, vec![msg]).await
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage<'_>>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Compose URLs
        let set: HashSet<RelayUrl> = urls
            .into_iter()
            .map(|u| u.try_into_url())
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
        for url in set.into_iter() {
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

    /// Send event to all relays with `WRITE` flag (check [`RelayServiceFlags`] for more details).
    pub async fn send_event(&self, event: &Event) -> Result<Output<EventId>, Error> {
        let urls: Vec<RelayUrl> = self.__write_relay_urls().await;
        self.send_event_to(urls, event).await
    }

    /// Send event to specific relays
    pub async fn send_event_to<I, U>(
        &self,
        urls: I,
        event: &Event,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Compose URLs
        let set: HashSet<RelayUrl> = urls
            .into_iter()
            .map(|u| u.try_into_url())
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

        // Save event into database
        self.inner.state.database().save_event(event).await?;

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
            urls.push(url);
            futures.push(relay.send_event(event));
        }

        // Join futures
        let list = future::join_all(futures).await;

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

    /// Subscribe to filters to all relays with `READ` flag.
    ///
    /// Check [`RelayPool::subscribe_with_id_to`] docs to learn more.
    pub async fn subscribe<F>(
        &self,
        filters: F,
        opts: SubscribeOptions,
    ) -> Result<Output<SubscriptionId>, Error>
    where
        F: Into<Vec<Filter>>,
    {
        let id: SubscriptionId = SubscriptionId::generate();
        let output: Output<()> = self.subscribe_with_id(id.clone(), filters, opts).await?;
        Ok(Output {
            val: id,
            success: output.success,
            failed: output.failed,
        })
    }

    /// Subscribe to filters with custom [SubscriptionId] to all relays with `READ` flag.
    ///
    /// Check [`RelayPool::subscribe_with_id_to`] docs to learn more.
    pub async fn subscribe_with_id<F>(
        &self,
        id: SubscriptionId,
        filters: F,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error>
    where
        F: Into<Vec<Filter>>,
    {
        // Convert filters
        let filters: Vec<Filter> = filters.into();

        // Check if isn't auto-closing subscription
        if !opts.is_auto_closing() {
            // Save subscription
            self.save_subscription(id.clone(), filters.clone()).await;
        }

        // Get relay urls
        let urls: Vec<RelayUrl> = self.__read_relay_urls().await;

        // Subscribe
        self.subscribe_with_id_to(urls, id, filters, opts).await
    }

    /// Subscribe to filters to specific relays
    ///
    /// Check [`RelayPool::subscribe_with_id_to`] docs to learn more.
    pub async fn subscribe_to<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: SubscribeOptions,
    ) -> Result<Output<SubscriptionId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let id: SubscriptionId = SubscriptionId::generate();
        let output: Output<()> = self
            .subscribe_with_id_to(urls, id.clone(), filter, opts)
            .await?;
        Ok(Output {
            val: id,
            success: output.success,
            failed: output.failed,
        })
    }

    /// Subscribe to filters with custom [SubscriptionId] to specific relays
    ///
    /// This method doesn't add relays!
    /// All the relays must be added to the pool with [`RelayPool::add_relay`].
    /// If the specified relays don't exist, [`Error::RelayNotFound`] is returned.
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    ///
    /// Auto-closing subscriptions aren't saved in the subscription map!
    pub async fn subscribe_with_id_to<I, U, F>(
        &self,
        urls: I,
        id: SubscriptionId,
        filters: F,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        F: Into<Vec<Filter>>,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let filters: Vec<Filter> = filters.into();
        let targets = urls.into_iter().map(|u| (u, filters.clone()));
        self.subscribe_targeted(id, targets, opts).await
    }

    /// Targeted subscription
    ///
    /// Subscribe to specific relays with specific filters.
    pub async fn subscribe_targeted<I, U, F>(
        &self,
        id: SubscriptionId,
        targets: I,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = (U, F)>,
        U: TryIntoUrl,
        F: Into<Vec<Filter>>,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Collect targets
        let targets: HashMap<RelayUrl, Vec<Filter>> = targets
            .into_iter()
            .map(|(u, f)| Ok((u.try_into_url()?, f.into())))
            .collect::<Result<_, Error>>()?;

        // Check if urls set is empty
        if targets.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Check if relays map is empty
        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // Check if urls set contains ONLY already added relays
        if !targets.keys().all(|url| relays.contains_key(url)) {
            return Err(Error::RelayNotFound);
        }

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(targets.len());
        let mut futures = Vec::with_capacity(targets.len());
        let mut output: Output<()> = Output::default();

        // Compose futures
        for (url, filter) in targets.into_iter() {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            let id: SubscriptionId = id.clone();
            urls.push(url);
            futures.push(relay.subscribe_with_id(id, filter, opts));
        }

        // Join futures
        let list = future::join_all(futures).await;

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

    /// Sync events with relays (negentropy reconciliation)
    pub async fn sync(
        &self,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        let urls: Vec<RelayUrl> = self.__relay_urls().await;
        self.sync_with(urls, filter, opts).await
    }

    /// Sync events with specific relays (negentropy reconciliation)
    pub async fn sync_with<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Get items
        let items: Vec<(EventId, Timestamp)> = self
            .inner
            .state
            .database()
            .negentropy_items(filter.clone())
            .await?;

        let tup: (Filter, Vec<(EventId, Timestamp)>) = (filter, items);

        // Reconcile
        let targets = urls.into_iter().map(|u| (u, tup.clone()));
        self.sync_targeted(targets, opts).await
    }

    /// Sync events with specific relays and filters (negentropy reconciliation)
    pub async fn sync_targeted<I, U>(
        &self,
        targets: I,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = (U, (Filter, Vec<(EventId, Timestamp)>))>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Collect targets
        let targets: HashMap<RelayUrl, (Filter, Vec<(EventId, Timestamp)>)> = targets
            .into_iter()
            .map(|(u, v)| Ok((u.try_into_url()?, v)))
            .collect::<Result<_, Error>>()?;

        // Check if urls set is empty
        if targets.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Check if empty
        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // Check if urls set contains ONLY already added relays
        if !targets.keys().all(|url| relays.contains_key(url)) {
            return Err(Error::RelayNotFound);
        }

        // TODO: shared reconciliation output to avoid to request duplicates?

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(targets.len());
        let mut futures = Vec::with_capacity(targets.len());
        let mut output: Output<Reconciliation> = Output::default();

        // Compose futures
        for (url, (filter, items)) in targets.into_iter() {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            urls.push(url);
            futures.push(relay.sync_with_items(filter, items, opts));
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

        // Check if sync failed (no success)
        if output.success.is_empty() {
            return Err(Error::NegentropyReconciliationFailed);
        }

        Ok(output)
    }

    /// Fetch events from relays with [`RelayServiceFlags::READ`] flag.
    pub async fn fetch_events<F>(
        &self,
        filters: F,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<Events, Error>
    where
        F: Into<Vec<Filter>>,
    {
        let urls: Vec<RelayUrl> = self.__read_relay_urls().await;
        self.fetch_events_from(urls, filters, timeout, policy).await
    }

    /// Fetch events from specific relays
    pub async fn fetch_events_from<I, U, F>(
        &self,
        urls: I,
        filters: F,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<Events, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        F: Into<Vec<Filter>>,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert filters
        let filters: Vec<Filter> = filters.into();

        // Construct a new events collection
        let mut events: Events = if filters.len() == 1 {
            // SAFETY: this can't panic because the filters are already verified that list isn't empty.
            let filter: &Filter = &filters[0];
            Events::new(filter)
        } else {
            // More than a filter, so we can't ensure to respect the limit -> construct a default collection.
            Events::default()
        };

        // Stream events
        let mut stream = self
            .stream_events_from(urls, filters, timeout, policy)
            .await?;
        while let Some(event) = stream.next().await {
            // To find out more about why the `force_insert` was used, search for EVENTS_FORCE_INSERT ine the code.
            events.force_insert(event);
        }

        Ok(events)
    }

    /// Stream events from relays with `READ` flag.
    pub async fn stream_events(
        &self,
        filter: Filter,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<BoxStream<Event>, Error> {
        let urls: Vec<RelayUrl> = self.__read_relay_urls().await;
        self.stream_events_from(urls, filter, timeout, policy).await
    }

    /// Stream events from specific relays
    pub async fn stream_events_from<I, U, F>(
        &self,
        urls: I,
        filters: F,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<BoxStream<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        F: Into<Vec<Filter>>,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let filters: Vec<Filter> = filters.into();
        let targets = urls.into_iter().map(|u| (u, filters.clone()));
        self.stream_events_targeted(targets, timeout, policy).await
    }

    /// Targeted streaming events
    ///
    /// Stream events from specific relays with specific filters
    pub async fn stream_events_targeted<I, U, F>(
        &self,
        targets: I,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<BoxStream<Event>, Error>
    where
        I: IntoIterator<Item = (U, F)>,
        U: TryIntoUrl,
        F: Into<Vec<Filter>>,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Collect targets
        let targets: HashMap<RelayUrl, Vec<Filter>> = targets
            .into_iter()
            .map(|(u, v)| Ok((u.try_into_url()?, v.into())))
            .collect::<Result<_, Error>>()?;

        // Check if `targets` map is empty
        if targets.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.inner.atomic.relays.read().await;

        // Check if empty
        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        let mut urls = Vec::with_capacity(targets.len());
        let mut futures = Vec::with_capacity(targets.len());

        for (url, filter) in targets.into_iter() {
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

        // Re-construct streams
        let mut streams: Vec<(RelayUrl, BoxStream<_>)> = Vec::with_capacity(awaited.len());

        // Zip-up urls and futures into a single iterator
        let iter = urls.into_iter().zip(awaited.into_iter());

        for (url, stream) in iter {
            match stream {
                Ok(stream) => streams.push((url, stream)),
                Err(e) => tracing::error!(url = %url, error = %e, "Failed to stream events."),
            }
        }

        // Create a new channel
        // NOTE: the events are deduplicated, so here isn't necessary a huge capacity.
        let (tx, rx) = mpsc::channel(512);

        // Single driver task: polls all streams, de-duplicates, forwards
        task::spawn(async move {
            // IDs collection, needed to check if an event was already sent to the stream
            let ids: Arc<Mutex<HashSet<EventId>>> = Arc::new(Mutex::new(HashSet::new()));

            let mut futures = Vec::with_capacity(streams.len());

            for (url, mut stream) in streams.into_iter() {
                let tx = tx.clone();
                let ids = ids.clone();

                futures.push(async move {
                    while let Some(res) = stream.next().await {
                        match res {
                            Ok(event) => {
                                let mut ids = ids.lock().await;

                                // Check if ID was already seen or insert into set.
                                if ids.insert(event.id) {
                                    // Immediately drop the set
                                    drop(ids);

                                    // Send event
                                    let _ = tx.send(event).await;
                                }
                            }
                            Err(e) => {
                                tracing::error!(url = %url, error = %e, "Failed to stream events.")
                            }
                        }
                    }
                });
            }

            // Wait that all futures complete
            future::join_all(futures).await;

            // Close the channel
            drop(tx);
        });

        // Return stream
        Ok(Box::pin(ReceiverStream::new(rx)))
    }

    /// Handle notifications
    pub async fn handle_notifications<F, Fut>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(RelayPoolNotification) -> Fut,
        Fut: Future<Output = Result<bool>>,
    {
        let mut notifications = self.notifications();
        while let Ok(notification) = notifications.recv().await {
            let shutdown: bool = RelayPoolNotification::Shutdown == notification;
            let exit: bool = func(notification)
                .await
                .map_err(|e| Error::Handler(e.to_string()))?;
            if exit || shutdown {
                break;
            }
        }
        Ok(())
    }
}

/// Return `true` if the relay can be removed
///
/// If it CAN'T be removed,
/// the flags are automatically updated (remove `READ`, `WRITE` and `DISCOVERY` flags).
fn can_remove_relay(relay: &Relay) -> bool {
    let flags = relay.flags();
    if flags.has_any(RelayServiceFlags::GOSSIP) {
        // Remove READ, WRITE and DISCOVERY flags
        flags.remove(
            RelayServiceFlags::READ | RelayServiceFlags::WRITE | RelayServiceFlags::DISCOVERY,
        );

        // Relay has `GOSSIP` flag so it can't be removed.
        return false;
    }

    // Relay can be removed
    true
}

#[cfg(test)]
mod tests {
    use nostr_relay_builder::MockRelay;

    use super::*;

    fn relay_gossip_opts() -> RelayOptions {
        let mut flags: RelayServiceFlags = RelayServiceFlags::default();
        flags.add(RelayServiceFlags::GOSSIP);
        RelayOptions::default().flags(flags)
    }

    #[tokio::test]
    async fn test_shutdown() {
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url();

        let pool = RelayPool::default();

        pool.add_relay(&url, RelayOptions::default()).await.unwrap();

        pool.connect().await;

        assert!(!pool.is_shutdown());

        tokio::time::sleep(Duration::from_secs(1)).await;

        pool.shutdown().await;

        assert!(pool.is_shutdown());

        assert!(matches!(
            pool.add_relay(&url, RelayOptions::default())
                .await
                .unwrap_err(),
            Error::Shutdown
        ));
    }

    #[tokio::test]
    async fn test_remove_nonexistent_relay() {
        let pool = RelayPool::default();

        let opts: RelayOptions = RelayOptions::default();
        pool.add_relay("ws://127.0.0.1:6666", opts).await.unwrap();

        assert!(matches!(
            pool.remove_relay("ws://127.0.0.1:7777").await.unwrap_err(),
            Error::RelayNotFound
        ));
    }

    #[tokio::test]
    async fn test_remove_relay() {
        let pool = RelayPool::default();

        let opts: RelayOptions = RelayOptions::default();
        pool.add_relay("ws://127.0.0.1:6666", opts).await.unwrap();

        let opts: RelayOptions = relay_gossip_opts();
        pool.add_relay("ws://127.0.0.1:8888", opts).await.unwrap();

        assert_eq!(pool.relays().await.len(), 2);
        assert_eq!(pool.all_relays().await.len(), 2);

        // Remove the non-gossip relay
        assert!(pool.remove_relay("ws://127.0.0.1:6666").await.is_ok());
        assert!(matches!(
            pool.relay("ws://127.0.0.1:6666").await.unwrap_err(),
            Error::RelayNotFound
        ));
        assert_eq!(pool.relays().await.len(), 1);
        assert_eq!(pool.all_relays().await.len(), 1);

        // Try to remove the gossip relay (will not be removed)
        assert!(pool.remove_relay("ws://127.0.0.1:8888").await.is_ok());
        assert!(pool.relay("ws://127.0.0.1:8888").await.is_ok()); // The relay exists in the pool!
        assert!(pool.relays().await.is_empty()); // This gets only the READ/WRITE relays, which are now 0
        assert_eq!(pool.all_relays().await.len(), 1);
    }

    #[tokio::test]
    async fn test_force_remove_relay() {
        let pool = RelayPool::default();

        let opts: RelayOptions = RelayOptions::default();
        pool.add_relay("ws://127.0.0.1:6666", opts).await.unwrap();

        let opts: RelayOptions = relay_gossip_opts();
        pool.add_relay("ws://127.0.0.1:8888", opts).await.unwrap();

        assert_eq!(pool.relays().await.len(), 2);
        assert_eq!(pool.all_relays().await.len(), 2);

        // Force remove the non-gossip relay
        assert!(pool.force_remove_relay("ws://127.0.0.1:6666").await.is_ok());
        assert!(matches!(
            pool.relay("ws://127.0.0.1:6666").await.unwrap_err(),
            Error::RelayNotFound
        ));
        assert_eq!(pool.relays().await.len(), 1);
        assert_eq!(pool.all_relays().await.len(), 1);

        // Force remove the gossip relay
        assert!(pool.force_remove_relay("ws://127.0.0.1:8888").await.is_ok());
        assert!(matches!(
            pool.relay("ws://127.0.0.1:8888").await.unwrap_err(),
            Error::RelayNotFound
        ));
        assert!(pool.relays().await.is_empty());
        assert!(pool.all_relays().await.is_empty());
    }

    #[tokio::test]
    async fn test_remove_all_relays() {
        let pool = RelayPool::default();

        let opts: RelayOptions = RelayOptions::default();
        pool.add_relay("ws://127.0.0.1:6666", opts).await.unwrap();

        let opts: RelayOptions = RelayOptions::default();
        pool.add_relay("ws://127.0.0.1:7777", opts).await.unwrap();

        let opts: RelayOptions = relay_gossip_opts();
        pool.add_relay("ws://127.0.0.1:8888", opts).await.unwrap();

        assert_eq!(pool.relays().await.len(), 3);
        assert_eq!(pool.all_relays().await.len(), 3);

        // Remove all relays
        pool.remove_all_relays().await;
        assert!(matches!(
            pool.relay("ws://127.0.0.1:6666").await.unwrap_err(),
            Error::RelayNotFound
        ));
        assert!(matches!(
            pool.relay("ws://127.0.0.1:7777").await.unwrap_err(),
            Error::RelayNotFound
        ));
        assert!(pool.relay("ws://127.0.0.1:8888").await.is_ok()); // The GOSSIP relay still exists
        assert!(pool.relays().await.is_empty()); // This gets only the READ/WRITE relays, which are now 0
        assert_eq!(pool.all_relays().await.len(), 1); // The GOSSIP relay still exists
    }

    #[tokio::test]
    async fn test_force_remove_all_relays() {
        let pool = RelayPool::default();

        let opts: RelayOptions = RelayOptions::default();
        pool.add_relay("ws://127.0.0.1:6666", opts).await.unwrap();

        let opts: RelayOptions = RelayOptions::default();
        pool.add_relay("ws://127.0.0.1:7777", opts).await.unwrap();

        let opts: RelayOptions = relay_gossip_opts();
        pool.add_relay("ws://127.0.0.1:8888", opts).await.unwrap();

        assert_eq!(pool.relays().await.len(), 3);
        assert_eq!(pool.all_relays().await.len(), 3);

        // Force remove all relays
        pool.force_remove_all_relays().await;

        // Check if relays map is empty
        assert!(pool.relays().await.is_empty());
        assert!(pool.all_relays().await.is_empty());

        // Double check that relays doesn't exist
        assert!(matches!(
            pool.relay("ws://127.0.0.1:6666").await.unwrap_err(),
            Error::RelayNotFound
        ));
        assert!(matches!(
            pool.relay("ws://127.0.0.1:7777").await.unwrap_err(),
            Error::RelayNotFound
        ));
        assert!(matches!(
            pool.relay("ws://127.0.0.1:8888").await.unwrap_err(),
            Error::RelayNotFound
        ));
    }
}
