// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use atomic_destructor::{AtomicDestructor, StealthClone};
use nostr::prelude::*;
use nostr_database::{DynNostrDatabase, Events, IntoNostrDatabase, MemoryDatabase};
use tokio::sync::broadcast;
pub use tokio_stream::wrappers::ReceiverStream;

mod error;
mod internal;
pub mod options;
mod output;

pub use self::error::Error;
use self::internal::InternalRelayPool;
pub use self::options::RelayPoolOptions;
pub use self::output::Output;
use crate::relay::flags::FlagCheck;
use crate::relay::options::{FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions};
use crate::relay::{Relay, RelayFiltering, RelayStatus};
use crate::{Reconciliation, RelayServiceFlags, SubscribeOptions};

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
    pub async fn shutdown(&self) -> Result<(), Error> {
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

    /// Get relay filtering
    #[inline]
    pub fn filtering(&self) -> RelayFiltering {
        self.inner.filtering()
    }

    /// Get all relays
    ///
    /// This method return all relays added to the pool, including the ones for gossip protocol or other services.
    #[inline]
    pub async fn all_relays(&self) -> HashMap<Url, Relay> {
        self.inner.all_relays().await
    }

    /// Get relays with `READ` or `WRITE` flags
    #[inline]
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        self.inner.relays().await
    }

    /// Get relays that have a certain [RelayServiceFlag] enabled
    #[inline]
    pub async fn relays_with_flag(
        &self,
        flag: RelayServiceFlags,
        check: FlagCheck,
    ) -> HashMap<Url, Relay> {
        self.inner.relays_with_flag(flag, check).await
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

    /// Add new relay
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    #[inline]
    pub async fn add_relay<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.add_relay(url, true, opts).await
    }

    /// Try to get relay by `url` or add it to pool.
    ///
    /// Return `Some(..)` only if the relay already exists.
    #[inline]
    pub async fn get_or_add_relay<U>(
        &self,
        url: U,
        inherit_pool_subscriptions: bool,
        opts: RelayOptions,
    ) -> Result<Option<Relay>, Error>
    where
        U: TryIntoUrl + Clone,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .get_or_add_relay(url, inherit_pool_subscriptions, opts)
            .await
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has `INBOX` or `OUTBOX` flags, it will not be removed from the pool and its
    /// flags will be updated (remove `READ`, `WRITE` and `DISCOVERY` flags).
    #[inline]
    pub async fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.remove_relay(url, false).await
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
        self.inner.remove_relay(url, true).await
    }

    /// Disconnect and remove all relays
    #[deprecated(since = "0.36.0")]
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        Ok(())
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
        self.inner.connect_relay(url, connection_timeout).await
    }

    /// Disconnect relay
    #[inline]
    pub async fn disconnect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.disconnect_relay(url).await
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

    /// Send event to all relays with `WRITE` flag (check [`RelayServiceFlags`] for more details).
    #[inline]
    pub async fn send_event(
        &self,
        event: Event,
        opts: RelaySendOptions,
    ) -> Result<Output<EventId>, Error> {
        self.inner.send_event(event, opts).await
    }

    /// Send multiple events at once to all relays with `WRITE` flag (check [`RelayServiceFlags`] for more details).
    #[inline]
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        self.inner.batch_event(events, opts).await
    }

    /// Send event to specific relays
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

    /// Send multiple events at once to specific relays
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

    /// Subscribe to filters to all relays with `READ` flag.
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

    /// Subscribe to filters with custom [SubscriptionId] to all relays with `READ` flag.
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

    /// Targeted subscription
    ///
    /// Subscribe to specific relays with specific filters
    #[inline]
    pub async fn subscribe_targeted<I, U>(
        &self,
        id: SubscriptionId,
        targets: I,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = (U, Vec<Filter>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.subscribe_targeted(id, targets, opts).await
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

    /// Fetch events from relays with [`RelayServiceFlags::READ`] flag.
    #[inline]
    pub async fn fetch_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Events, Error> {
        self.inner.fetch_events(filters, timeout, opts).await
    }

    /// Get events of filters from relays with `READ` flag.
    #[deprecated(since = "0.36.0", note = "Use `fetch_events` instead")]
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        Ok(self
            .inner
            .fetch_events(filters, timeout, opts)
            .await?
            .to_vec())
    }

    /// Fetch events from specific relays
    #[inline]
    pub async fn fetch_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Events, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .fetch_events_from(urls, filters, timeout, opts)
            .await
    }

    /// Get events of filters from specific relays
    #[deprecated(since = "0.36.0", note = "Use `fetch_events_from` instead")]
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
        Ok(self
            .inner
            .fetch_events_from(urls, filters, timeout, opts)
            .await?
            .to_vec())
    }

    /// Stream events
    #[deprecated(since = "0.36.0", note = "Use `stream_events` instead")]
    pub async fn stream_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<ReceiverStream<Event>, Error> {
        self.stream_events(filters, timeout, opts).await
    }

    /// Stream events from relays with `READ` flag.
    #[inline]
    pub async fn stream_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<ReceiverStream<Event>, Error> {
        self.inner.stream_events(filters, timeout, opts).await
    }

    /// Stream events from specific relays
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

    /// Targeted streaming events
    ///
    /// Stream events from specific relays with specific filters
    pub async fn stream_events_targeted<I, U>(
        &self,
        source: I,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = (U, Vec<Filter>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .stream_events_targeted(source, timeout, opts)
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

    /// Targeted negentropy reconciliation
    ///
    /// Reconcile events with specific relays and filters
    #[inline]
    pub async fn reconcile_targeted<I, U>(
        &self,
        targets: I,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = (U, HashMap<Filter, Vec<(EventId, Timestamp)>>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.reconcile_targeted(targets, opts).await
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
