use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::future::{Future, IntoFuture};
use std::iter::Zip;
use std::mem;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::vec::IntoIter;

use async_utility::task;
use futures::{future, StreamExt};
use nostr_database::prelude::*;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

mod builder;
mod error;

pub(crate) use self::builder::RelayPoolBuilder;
pub(crate) use self::error::Error;
use crate::client::{ClientNotification, Output};
use crate::monitor::Monitor;
use crate::relay::{
    self, AtomicRelayCapabilities, Reconciliation, Relay, RelayCapabilities, RelayOptions,
    ReqExitPolicy, SubscribeAutoCloseOptions, SyncOptions,
};
use crate::shared::SharedState;
use crate::stream::{BoxedStream, ReceiverStream};

pub(super) type Relays = HashMap<RelayUrl, Relay>;

// IMPORTANT: we rely on the Drop trait for shutting down the pool,
// so it's important that the RelayPool can't be cloned, otherwise may cause a non-expected shutdown.
#[derive(Debug)]
pub(crate) struct RelayPool {
    state: SharedState,
    relays: RwLock<Relays>,
    notification_sender: broadcast::Sender<ClientNotification>,
    shutdown: AtomicBool,
    max_relays: Option<usize>,
}

// Shutdown the pool on the **FIRST** drop
// TODO: use AsyncDrop when will be stable: https://doc.rust-lang.org/std/future/trait.AsyncDrop.html
impl Drop for RelayPool {
    fn drop(&mut self) {
        // Take the relays
        let relays: RwLock<Relays> = mem::take(&mut self.relays);

        // Consume the RwLock
        let mut relays: Relays = relays.into_inner();

        // Shutdown
        shutdown(&self.shutdown, &mut relays, &self.notification_sender)
    }
}

impl RelayPool {
    fn from_builder(builder: RelayPoolBuilder) -> Self {
        let (notification_sender, _) = broadcast::channel(builder.notification_channel_size);

        Self {
            state: SharedState::new(
                builder.database,
                builder.websocket_transport,
                builder.signer,
                builder.admit_policy,
                builder.nip42_auto_authentication,
                builder.monitor,
            ),
            relays: RwLock::new(HashMap::new()),
            notification_sender,
            shutdown: AtomicBool::new(false),
            max_relays: builder.max_relays,
        }
    }

    #[inline]
    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    #[inline]
    pub(crate) async fn shutdown(&self) {
        // Acquire write lock
        let mut relays = self.relays.write().await;

        // Shutdown
        shutdown(&self.shutdown, &mut relays, &self.notification_sender)
    }

    #[inline]
    pub(crate) fn notifications(&self) -> broadcast::Receiver<ClientNotification> {
        self.notification_sender.subscribe()
    }

    #[inline]
    pub(crate) fn monitor(&self) -> Option<&Monitor> {
        self.state.monitor.as_ref()
    }

    #[inline]
    pub(crate) fn state(&self) -> &SharedState {
        &self.state
    }

    #[inline]
    pub(crate) fn database(&self) -> &Arc<dyn NostrDatabase> {
        self.state.database()
    }

    #[inline]
    pub(crate) async fn relay_urls_with_any_cap(
        &self,
        capabilities: RelayCapabilities,
    ) -> HashSet<RelayUrl> {
        let relays = self.relays.read().await;
        filter_relays_with_any_cap(&relays, capabilities)
            .map(|(k, ..)| k.clone())
            .collect()
    }

    #[inline]
    pub(crate) async fn read_relay_urls(&self) -> HashSet<RelayUrl> {
        self.relay_urls_with_any_cap(RelayCapabilities::READ).await
    }

    #[inline]
    pub(crate) async fn write_relay_urls(&self) -> HashSet<RelayUrl> {
        self.relay_urls_with_any_cap(RelayCapabilities::WRITE).await
    }

    // Get **all** relays
    #[inline]
    pub(crate) async fn all_relays(&self) -> HashMap<RelayUrl, Relay> {
        let relays = self.relays.read().await;
        relays.clone()
    }

    // Get relays that have any of the specified [`RelayCapabilities`]
    pub(crate) async fn relays_with_any_cap(
        &self,
        capabilities: RelayCapabilities,
    ) -> HashMap<RelayUrl, Relay> {
        let relays = self.relays.read().await;
        filter_relays_with_any_cap(&relays, capabilities)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    #[inline]
    pub(crate) async fn relay(&self, url: &RelayUrl) -> Option<Relay> {
        // Acquire read lock
        let relays = self.relays.read().await;

        // Get relay and clone it
        relays.get(url).cloned()
    }

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
        let mut relays = self.relays.write().await;

        // Check if the relay already exists
        if let Some(relay) = relays.get(&url) {
            // Add capabilities to the existing relay
            let current_capabilities: &AtomicRelayCapabilities = relay.capabilities();
            current_capabilities.add(capabilities);

            // Return
            return Ok(false);
        }

        // Check number fo relays and limit
        if let Some(max) = self.max_relays {
            if relays.len() >= max {
                return Err(Error::TooManyRelays { limit: max });
            }
        }

        // Get owned url
        let url: RelayUrl = url.into_owned();

        // Compose new relay
        let mut relay: Relay =
            Relay::new_shared(url.clone(), self.state.clone(), capabilities, opts);

        // Set notification sender
        relay.set_notification_sender(self.notification_sender.clone());

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
        let mut relays = self.relays.write().await;

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

    pub(crate) async fn remove_all_relays(&self, force: bool) {
        // Acquire write lock
        let mut relays = self.relays.write().await;

        // Remove all relays
        if force {
            force_remove_all_relays(&mut relays);
        } else {
            remove_all_relays(&mut relays);
        }
    }

    pub(crate) async fn connect(&self) {
        // Lock with read shared access
        let relays = self.relays.read().await;

        // Connect
        for relay in relays.values() {
            relay.connect()
        }
    }

    pub(crate) async fn wait_for_connection(&self, timeout: Duration) {
        // Lock with read shared access
        let relays = self.relays.read().await;

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
        let relays = self.relays.read().await;

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(relays.len());
        let mut futures = Vec::with_capacity(relays.len());
        let mut output: Output<()> = Output::default();

        // Filter only relays that can connect and compose futures
        for relay in relays.values().filter(|r| r.status().can_connect()) {
            urls.push(relay.url().clone());
            futures.push(relay.try_connect().timeout(timeout).into_future());
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

    pub(crate) async fn disconnect(&self) {
        // Lock with read shared access
        let relays = self.relays.read().await;

        // Iter values and disconnect
        for relay in relays.values() {
            relay.disconnect();
        }
    }

    pub(crate) async fn connect_relay(&self, url: &RelayUrl) -> Result<(), Error> {
        // Lock with read shared access
        let relays = self.relays.read().await;

        // Get relay
        let relay: &Relay = relays.get(url).ok_or(Error::RelayNotFound)?;

        // Connect
        relay.connect();

        Ok(())
    }

    pub(crate) async fn try_connect_relay(
        &self,
        url: &RelayUrl,
        timeout: Duration,
    ) -> Result<(), Error> {
        // Lock with read shared access
        let relays = self.relays.read().await;

        // Get relay
        let relay: &Relay = relays.get(url).ok_or(Error::RelayNotFound)?;

        // Try to connect
        relay.try_connect().timeout(timeout).await?;

        Ok(())
    }

    pub(crate) async fn disconnect_relay(&self, url: &RelayUrl) -> Result<(), Error> {
        // Lock with read shared access
        let relays = self.relays.read().await;

        // Get relay
        let relay: &Relay = relays.get(url).ok_or(Error::RelayNotFound)?;

        // Disconnect
        relay.disconnect();

        Ok(())
    }

    #[inline]
    pub(crate) async fn subscriptions(
        &self,
    ) -> HashMap<SubscriptionId, HashMap<RelayUrl, Vec<Filter>>> {
        // Lock with read shared access
        let relays = self.relays.read().await;

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

    #[inline]
    pub(crate) async fn subscription(&self, id: &SubscriptionId) -> HashMap<RelayUrl, Vec<Filter>> {
        // Lock with read shared access
        let relays = self.relays.read().await;

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

    pub(crate) async fn send_msg(
        &self,
        urls: HashSet<RelayUrl>,
        msg: ClientMessage<'_>,
    ) -> Result<Output<()>, Error> {
        // Check if urls set is empty
        if urls.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.relays.read().await;

        // if let ClientMessage::Event(event) = msg {
        //     self.state.database().save_event(event).await?;
        // }

        let mut output: Output<()> = Output::default();

        // Batch messages and construct outputs
        for url in urls {
            let relay: &Relay = relays.get(&url).ok_or(Error::RelayNotFound)?;
            match relay.send_msg(msg.clone()) {
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

    pub(crate) async fn send_event<I>(
        &self,
        urls: I,
        event: &Event,
        wait_for_ok_timeout: Duration,
        wait_for_authentication_timeout: Duration,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = RelayUrl>,
    {
        // Compose URLs
        let set: HashSet<RelayUrl> = urls.into_iter().collect();

        // Check if urls set is empty
        if set.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.relays.read().await;

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(set.len());
        let mut futures = Vec::with_capacity(set.len());
        let mut output: Output<EventId> = Output {
            val: event.id,
            success: HashSet::new(),
            failed: HashMap::new(),
        };

        // Compose futures
        for url in set.into_iter() {
            let relay: &Relay = relays.get(&url).ok_or(Error::RelayNotFound)?;
            urls.push(url);
            futures.push(
                relay
                    .send_event(event)
                    .ok_timeout(wait_for_ok_timeout)
                    .authentication_timeout(wait_for_authentication_timeout)
                    .into_future(),
            );
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
        let relays = self.relays.read().await;

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
            let mut sub = relay.subscribe(filter).with_id(id);

            if let Some(auto_close) = auto_close {
                sub = sub.close_on(auto_close);
            }

            // Create future
            urls.push(url);
            futures.push(sub.into_future());
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

    pub(crate) async fn unsubscribe(&self, id: &SubscriptionId) -> Output<()> {
        // Lock with read shared access
        let relays = self.relays.read().await;

        let mut urls: Vec<&RelayUrl> = Vec::with_capacity(relays.len());
        let mut futures = Vec::with_capacity(relays.len());

        let mut output: Output<()> = Output::default();

        // Compose futures
        for relay in relays.values() {
            // Create future
            urls.push(relay.url());
            futures.push(relay.unsubscribe(id));
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Iter results and construct output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(true) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url.clone());
                }
                // Subscription isn't found or auto-closing: do nothing
                Ok(false) => {}
                Err(e) => {
                    output.failed.insert(url.clone(), e.to_string());
                }
            }
        }

        output
    }

    pub(crate) async fn unsubscribe_all(&self) -> Output<()> {
        // Lock with read shared access
        let relays = self.relays.read().await;

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(relays.len());
        let mut futures = Vec::with_capacity(relays.len());

        let mut output: Output<()> = Output::default();

        // Compose futures
        for relay in relays.values() {
            // Create future
            urls.push(relay.url().clone());
            futures.push(relay.unsubscribe_all());
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Iter results and construct output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(()) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                }
                Err(e) => {
                    output.failed.insert(url, e.to_string());
                }
            }
        }

        output
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
        let relays = self.relays.read().await;

        // TODO: shared reconciliation output to avoid to request duplicates?

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(targets.len());
        let mut futures = Vec::with_capacity(targets.len());
        let mut output: Output<Reconciliation> = Output::default();

        // Compose futures
        for (url, (filter, items)) in targets.into_iter() {
            let relay: &Relay = relays.get(&url).ok_or(Error::RelayNotFound)?;
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
        let relays = self.relays.read().await;

        // Create a new channel
        // NOTE: the events are deduplicated and the send method awaits, so a huge capacity isn't necessary.
        let (tx, rx) = mpsc::channel(1024);

        let mut urls: Vec<RelayUrl> = Vec::with_capacity(filters.len());
        let mut futures = Vec::with_capacity(filters.len());

        for (url, filter) in filters {
            // Try to get the relay
            let relay: &Relay = relays.get(&url).ok_or(Error::RelayNotFound)?;

            // Push url
            urls.push(url);

            // Push stream events future
            futures.push(
                relay
                    .stream_events(filter)
                    .maybe_timeout(timeout)
                    .policy(policy)
                    .into_future(),
            );
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

#[inline]
fn filter_relays_with_any_cap(
    relays: &Relays,
    capabilities: RelayCapabilities,
) -> impl Iterator<Item = (&RelayUrl, &Relay)> {
    relays
        .iter()
        .filter(move |(_, r)| r.capabilities().has_any(capabilities))
}

// Return `true` if the relay can be removed
//
// If it CAN'T be removed,
// the capabilities are automatically updated (remove `READ`, `WRITE` and `DISCOVERY` capabilities).
fn can_remove_relay(relay: &Relay) -> bool {
    let capabilities = relay.capabilities();
    if capabilities.has_any(RelayCapabilities::GOSSIP) {
        // Remove READ, WRITE and DISCOVERY capabilities
        capabilities.remove(
            RelayCapabilities::READ | RelayCapabilities::WRITE | RelayCapabilities::DISCOVERY,
        );

        // Relay has `GOSSIP` capability, so it can't be removed.
        return false;
    }

    // Relay can be removed
    true
}

// Shutdown and remove all relays
fn remove_all_relays(relays: &mut Relays) {
    // Drain the map to get owned keys and values
    let old_relays: Relays = mem::take(&mut *relays);

    for (url, relay) in old_relays {
        // Check if it can be removed
        if can_remove_relay(&relay) {
            relay.shutdown();
        } else {
            // Re-insert into the map
            relays.insert(url, relay);
        }
    }
}

// Shutdown and force-remove all relays
fn force_remove_all_relays(relays: &mut Relays) {
    // Make sure to disconnect all relays
    for relay in relays.values() {
        relay.shutdown();
    }

    // Clear map
    relays.clear();
}

fn shutdown(
    shutdown: &AtomicBool,
    relays: &mut Relays,
    notification_sender: &broadcast::Sender<ClientNotification>,
) {
    // Mark as shutdown
    // If the previous value was `true`,
    // meaning that was already shutdown, immediately returns.
    if shutdown.swap(true, Ordering::SeqCst) {
        return;
    }

    // Disconnect and force remove all relays
    force_remove_all_relays(relays);

    // Send shutdown notification
    let _ = notification_sender.send(ClientNotification::Shutdown);
}
