// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use atomic_destructor::{AtomicDestructor, StealthClone};
use nostr::{
    ClientMessage, Event, EventId, Filter, RelayMessage, Result, SubscriptionId, Timestamp,
    TryIntoUrl, Url,
};
use nostr_database::{DynNostrDatabase, IntoNostrDatabase, MemoryDatabase};
use tokio::sync::broadcast;
pub use tokio_stream::wrappers::ReceiverStream;

mod error;
mod internal;
pub mod options;
mod result;

pub use self::error::Error;
use self::internal::InternalRelayPool;
pub use self::options::RelayPoolOptions;
pub use self::result::Output;
use crate::relay::options::{FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions};
use crate::relay::{Relay, RelayBlacklist, RelayStatus};
use crate::{Reconciliation, SubscribeOptions};

/// Relay Pool Notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayPoolNotification {
    /// Received an [`Event`]. Does not include events sent by this client.
    Event {
        /// Relay url
        relay_url: Url,
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// Event
        event: Box<Event>,
    },
    /// Received a [`RelayMessage`]. Includes messages wrapping events that were sent by this client.
    Message {
        /// Relay url
        relay_url: Url,
        /// Relay Message
        message: RelayMessage,
    },
    /// Relay status changed
    RelayStatus {
        /// Relay url
        relay_url: Url,
        /// Relay Status
        status: RelayStatus,
    },
    /// Authenticated to relay
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    Authenticated {
        /// Relay url
        relay_url: Url,
    },
    /// Shutdown
    Shutdown,
}

/// Relay Pool
#[derive(Debug, Clone)]
pub struct RelayPool {
    inner: AtomicDestructor<InternalRelayPool>,
}

impl Default for RelayPool {
    fn default() -> Self {
        Self::new(RelayPoolOptions::default())
    }
}

impl StealthClone for RelayPool {
    #[inline(always)]
    fn stealth_clone(&self) -> Self {
        Self {
            inner: self.inner.stealth_clone(),
        }
    }
}

impl RelayPool {
    /// Create new `RelayPool`
    #[inline]
    pub fn new(opts: RelayPoolOptions) -> Self {
        Self::with_database(opts, Arc::new(MemoryDatabase::default()))
    }

    /// New with database
    #[inline]
    pub fn with_database<D>(opts: RelayPoolOptions, database: D) -> Self
    where
        D: IntoNostrDatabase,
    {
        Self {
            inner: AtomicDestructor::new(InternalRelayPool::with_database(opts, database)),
        }
    }

    /// Completely shutdown pool
    #[inline]
    pub async fn shutdown(self) -> Result<(), Error> {
        self.inner.shutdown().await
    }

    /// Get new **pool** notification listener
    ///
    /// <div class="warning">When you call this method, you subscribe to the notifications channel from that precise moment. Anything received by relay/s before that moment is not included in the channel!</div>
    #[inline]
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.inner.notifications()
    }

    /// Get database
    #[inline]
    pub fn database(&self) -> Arc<DynNostrDatabase> {
        self.inner.database()
    }

    /// Get blacklist
    #[inline]
    pub fn blacklist(&self) -> RelayBlacklist {
        self.inner.blacklist()
    }

    /// Get relays
    #[inline]
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        self.inner.relays().await
    }

    /// Get [`Relay`]
    #[inline]
    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.relay(url).await
    }

    /// Add new relay by Url
    ///
    /// If there are pool subscriptions, the added relay will inherit them.
    /// Use `subscribe_to` instead of `subscribe` to prevent a subscription from being inherited.
    #[inline]
    pub async fn add_relay_url(&self, url: Url, opts: RelayOptions) -> bool {
        self.inner.add_relay(url, opts).await
    }

    /// Add new relay
    ///
    /// If there are pool subscriptions, the added relay will inherit them.
    /// Use `subscribe_to` instead of `subscribe` to prevent a subscription from being inherited.
    #[inline]
    pub async fn add_relay<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.inner.add_relay(url.try_into_url()?, opts).await)
    }

    /// Disconnect and remove relay
    #[inline]
    pub async fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.remove_relay(url).await
    }

    /// Disconnect and remove all relays
    #[inline]
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        self.inner.remove_all_relays().await
    }

    /// Connect to all added relays and keep connection alive
    #[inline]
    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        self.inner.connect(connection_timeout).await
    }

    /// Disconnect from all relays
    #[inline]
    pub async fn disconnect(&self) -> Result<(), Error> {
        self.inner.disconnect().await
    }

    /// Connect to relay
    #[inline]
    pub async fn connect_relay<U>(
        &self,
        url: U,
        connection_timeout: Option<Duration>,
    ) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let relay = self.relay(url).await?;
        Ok(relay.connect(connection_timeout).await)
    }

    /// Get subscriptions
    #[inline]
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        self.inner.subscriptions().await
    }

    /// Get subscription
    #[inline]
    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        self.inner.subscription(id).await
    }

    /// Register subscription in the [RelayPool]
    ///
    /// When a new relay will be added, saved subscriptions will be automatically used for it.
    #[inline]
    pub async fn save_subscription(&self, id: SubscriptionId, filters: Vec<Filter>) {
        self.inner.save_subscription(id, filters).await
    }

    /// Send client message to all connected relays
    #[inline]
    pub async fn send_msg(
        &self,
        msg: ClientMessage,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        self.inner.send_msg(msg, opts).await
    }

    /// Send multiple client messages at once to all connected relays
    #[inline]
    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        self.inner.batch_msg(msgs, opts).await
    }

    /// Send client message to specific relays
    ///
    /// Note: **the relays must already be added!**
    #[inline]
    pub async fn send_msg_to<I, U>(
        &self,
        urls: I,
        msg: ClientMessage,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.send_msg_to(urls, msg, opts).await
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    #[inline]
    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.batch_msg_to(urls, msgs, opts).await
    }

    /// Send event to **all connected relays** and wait for `OK` message
    #[inline]
    pub async fn send_event(
        &self,
        event: Event,
        opts: RelaySendOptions,
    ) -> Result<Output<EventId>, Error> {
        self.inner.send_event(event, opts).await
    }

    /// Send multiple [`Event`] at once to **all connected relays** and wait for `OK` message
    #[inline]
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        self.inner.batch_event(events, opts).await
    }

    /// Send event to **specific relays** and wait for `OK` message
    #[inline]
    pub async fn send_event_to<I, U>(
        &self,
        urls: I,
        event: Event,
        opts: RelaySendOptions,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.send_event_to(urls, event, opts).await
    }

    /// Send multiple events at once to **specific relays** and wait for `OK` message
    #[inline]
    pub async fn batch_event_to<I, U>(
        &self,
        urls: I,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.batch_event_to(urls, events, opts).await
    }

    /// Subscribe to filters to all connected relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    #[inline]
    pub async fn subscribe(
        &self,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<SubscriptionId>, Error> {
        self.inner.subscribe(filters, opts).await
    }

    /// Subscribe to filters with custom [SubscriptionId] to all connected relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    #[inline]
    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error> {
        self.inner.subscribe_with_id(id, filters, opts).await
    }

    /// Subscribe to filters to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    #[inline]
    pub async fn subscribe_to<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<SubscriptionId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.subscribe_to(urls, filters, opts).await
    }

    /// Subscribe to filters with custom [SubscriptionId] to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    #[inline]
    pub async fn subscribe_with_id_to<I, U>(
        &self,
        urls: I,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .subscribe_with_id_to(urls, id, filters, opts)
            .await
    }

    /// Unsubscribe from subscription
    #[inline]
    pub async fn unsubscribe(&self, id: SubscriptionId, opts: RelaySendOptions) {
        self.inner.unsubscribe(id, opts).await
    }

    /// Unsubscribe from all subscriptions
    #[inline]
    pub async fn unsubscribe_all(&self, opts: RelaySendOptions) {
        self.inner.unsubscribe_all(opts).await
    }

    /// Get events of filters
    #[inline]
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let relays = self.relays().await;
        self.get_events_from(relays.into_keys(), filters, timeout, opts)
            .await
    }

    /// Get events of filters from **specific relays**
    #[inline]
    pub async fn get_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .get_events_from(urls, filters, timeout, opts)
            .await
    }

    /// Stream events of filters
    #[inline]
    pub async fn stream_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<ReceiverStream<Event>, Error> {
        let relays = self.relays().await;
        self.stream_events_from(relays.into_keys(), filters, timeout, opts)
            .await
    }

    /// Stream events of filters from **specific relays**
    #[inline]
    pub async fn stream_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .stream_events_from(urls, filters, timeout, opts)
            .await
    }

    /// Negentropy reconciliation with all connected relays
    #[inline]
    pub async fn reconcile(
        &self,
        filter: Filter,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        self.inner.reconcile(filter, opts).await
    }

    /// Negentropy reconciliation with specified relays
    #[inline]
    pub async fn reconcile_with<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.reconcile_with(urls, filter, opts).await
    }

    /// Negentropy reconciliation with all relays and custom items
    #[inline]
    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        self.inner.reconcile_with_items(filter, items, opts).await
    }

    /// Negentropy reconciliation with custom relays and items
    #[inline]
    pub async fn reconcile_advanced<I, U>(
        &self,
        urls: I,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .reconcile_advanced(urls, filter, items, opts)
            .await
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
