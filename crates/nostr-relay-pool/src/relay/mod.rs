// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay

use std::cmp;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use async_wsocket::futures_util::Future;
use atomic_destructor::AtomicDestructor;
#[cfg(feature = "nip11")]
use nostr::nips::nip11::RelayInformationDocument;
use nostr::{ClientMessage, Event, EventId, Filter, RelayMessage, SubscriptionId, Timestamp, Url};
use nostr_database::{DynNostrDatabase, MemoryDatabase};
use tokio::sync::broadcast;

pub mod flags;
mod internal;
pub mod limits;
pub mod options;
pub mod stats;
mod status;

pub use self::flags::{AtomicRelayServiceFlags, RelayServiceFlags};
pub use self::internal::Error;
use self::internal::InternalRelay;
pub use self::limits::Limits;
pub use self::options::{
    FilterOptions, NegentropyDirection, NegentropyOptions, RelayOptions, RelaySendOptions,
    SubscribeAutoCloseOptions, SubscribeOptions,
};
pub use self::stats::RelayConnectionStats;
pub use self::status::RelayStatus;
use crate::pool::RelayPoolNotification;

/// Relay Notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayNotification {
    /// Received an [`Event`]. Does not include events sent by this client.
    Event {
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// Event
        event: Box<Event>,
    },
    /// Received a [`RelayMessage`]. Includes messages wrapping events that were sent by this client.
    Message {
        /// Relay Message
        message: RelayMessage,
    },
    /// Relay status changed
    RelayStatus {
        /// Relay Status
        status: RelayStatus,
    },
    /// Stop
    Stop,
    /// Shutdown
    Shutdown,
}

/// Relay
#[derive(Debug, Clone)]
pub struct Relay {
    pub(crate) inner: AtomicDestructor<InternalRelay>,
}

impl PartialEq for Relay {
    fn eq(&self, other: &Self) -> bool {
        self.inner.url == other.inner.url
    }
}

impl Eq for Relay {}

impl PartialOrd for Relay {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Relay {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.url.cmp(&other.inner.url)
    }
}

impl Relay {
    /// Create new `Relay` with **default** `options` and `limits` and `in-memory database`
    pub fn new(url: Url) -> Self {
        Self::with_opts(url, RelayOptions::default())
    }

    /// Create new `Relay` with default `in-memory database` custom `options` and/or `limits`
    pub fn with_opts(url: Url, opts: RelayOptions) -> Self {
        let database = Arc::new(MemoryDatabase::default());
        Self::custom(url, database, opts, Limits::default())
    }

    /// Create new `Relay` with **custom** `options`, `database` and/or `limits`
    pub fn custom(
        url: Url,
        database: Arc<DynNostrDatabase>,
        opts: RelayOptions,
        limits: Limits,
    ) -> Self {
        Self {
            inner: AtomicDestructor::new(InternalRelay::new(url, database, opts, limits)),
        }
    }

    /// Get relay url
    pub fn url(&self) -> Url {
        self.inner.url()
    }

    /// Get proxy
    #[cfg(not(target_arch = "wasm32"))]
    pub fn proxy(&self) -> Option<SocketAddr> {
        self.inner.proxy()
    }

    /// Get [`RelayStatus`]
    pub async fn status(&self) -> RelayStatus {
        self.inner.status().await
    }

    /// Get Relay Service Flags
    pub fn flags(&self) -> AtomicRelayServiceFlags {
        self.inner.flags()
    }

    /// Check if [`Relay`] is connected
    pub async fn is_connected(&self) -> bool {
        self.inner.is_connected().await
    }

    /// Get [`RelayInformationDocument`]
    #[cfg(feature = "nip11")]
    pub async fn document(&self) -> RelayInformationDocument {
        self.inner.document().await
    }

    /// Get subscriptions
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        self.inner.subscriptions().await
    }

    /// Get filters by [SubscriptionId]
    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        self.inner.subscription(id).await
    }

    /// Get [`RelayOptions`]
    pub fn opts(&self) -> RelayOptions {
        self.inner.opts()
    }

    /// Get [`RelayConnectionStats`]
    pub fn stats(&self) -> RelayConnectionStats {
        self.inner.stats()
    }

    /// Get queue len
    pub fn queue(&self) -> usize {
        self.inner.queue()
    }

    /// Get new **relay** notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayNotification> {
        self.inner.internal_notification_sender.subscribe()
    }

    /// Set external notification sender
    pub async fn set_notification_sender(
        &self,
        notification_sender: Option<broadcast::Sender<RelayPoolNotification>>,
    ) {
        self.inner
            .set_notification_sender(notification_sender)
            .await
    }

    /// Connect to relay and keep alive connection
    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        self.inner.connect(connection_timeout).await
    }

    /// Disconnect from relay and set status to 'Stopped'
    pub async fn stop(&self) -> Result<(), Error> {
        self.inner.stop().await
    }

    /// Disconnect from relay and set status to 'Terminated'
    pub async fn terminate(&self) -> Result<(), Error> {
        self.inner.terminate().await
    }

    /// Send msg to relay
    pub async fn send_msg(&self, msg: ClientMessage, opts: RelaySendOptions) -> Result<(), Error> {
        self.batch_msg(vec![msg], opts).await
    }

    /// Send multiple [`ClientMessage`] at once
    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        self.inner.batch_msg(msgs, opts).await
    }

    /// Send event and wait for `OK` relay msg
    pub async fn send_event(&self, event: Event, opts: RelaySendOptions) -> Result<EventId, Error> {
        self.inner.send_event(event, opts).await
    }

    /// Send multiple [`Event`] at once
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        self.inner.batch_event(events, opts).await
    }

    /// Subscribe to filters
    ///
    /// Internally generate a new random [SubscriptionId]. Check `subscribe_with_id` method to use a custom [SubscriptionId].
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub async fn subscribe(
        &self,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<SubscriptionId, Error> {
        self.inner.subscribe(filters, opts).await
    }

    /// Subscribe with custom [SubscriptionId]
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<(), Error> {
        self.inner.subscribe_with_id(id, filters, opts).await
    }

    /// Unsubscribe
    pub async fn unsubscribe(
        &self,
        id: SubscriptionId,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        self.inner.unsubscribe(id, opts).await
    }

    /// Unsubscribe from all subscriptions
    pub async fn unsubscribe_all(&self, opts: RelaySendOptions) -> Result<(), Error> {
        self.inner.unsubscribe_all(opts).await
    }

    /// Get events of filters with custom callback
    pub(crate) async fn get_events_of_with_callback<F>(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
        callback: impl Fn(Event) -> F,
    ) -> Result<(), Error>
    where
        F: Future<Output = ()>,
    {
        self.inner
            .get_events_of_with_callback(filters, timeout, opts, callback)
            .await
    }

    /// Get events of filters
    ///
    /// Get events from local database and relay
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        self.inner.get_events_of(filters, timeout, opts).await
    }

    /// Count events of filters
    pub async fn count_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<usize, Error> {
        self.inner.count_events_of(filters, timeout).await
    }

    /// Negentropy reconciliation
    ///
    /// Use events stored in database
    pub async fn reconcile(&self, filter: Filter, opts: NegentropyOptions) -> Result<(), Error> {
        self.inner.reconcile(filter, opts).await
    }

    /// Negentropy reconciliation
    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<(), Error> {
        self.inner.reconcile_with_items(filter, items, opts).await
    }

    /// Check if relay support negentropy protocol
    pub async fn support_negentropy(&self) -> Result<bool, Error> {
        self.inner.support_negentropy().await
    }
}
