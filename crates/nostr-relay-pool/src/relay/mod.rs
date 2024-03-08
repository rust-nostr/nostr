// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay

use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::{cmp, fmt};

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
    RequestAutoCloseOptions, RequestOptions,
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

/// Internal Subscription ID
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InternalSubscriptionId {
    /// Default
    Default,
    /// Pool
    Pool,
    /// Custom
    Custom(String),
}

impl fmt::Display for InternalSubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Pool => write!(f, "pool"),
            Self::Custom(c) => write!(f, "{c}"),
        }
    }
}

impl<S> From<S> for InternalSubscriptionId
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        let s: String = s.into();
        match s.as_str() {
            "default" => Self::Default,
            "pool" => Self::Pool,
            _ => Self::Custom(s),
        }
    }
}

/// Relay instance's actual subscription with its unique id
#[derive(Debug, Clone)]
pub struct ActiveSubscription {
    /// SubscriptionId to update or cancel subscription
    id: SubscriptionId,
    /// Subscriptions filters
    filters: Vec<Filter>,
}

impl Default for ActiveSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveSubscription {
    /// Create new empty [`ActiveSubscription`]
    pub fn new() -> Self {
        Self {
            id: SubscriptionId::generate(),
            filters: Vec::new(),
        }
    }

    /// Create new empty [`ActiveSubscription`]
    pub fn with_filters(filters: Vec<Filter>) -> Self {
        Self {
            id: SubscriptionId::generate(),
            filters,
        }
    }

    /// Get [`SubscriptionId`]
    pub fn id(&self) -> SubscriptionId {
        self.id.clone()
    }

    /// Get subscription filters
    pub fn filters(&self) -> Vec<Filter> {
        self.filters.clone()
    }
}

/// Relay
#[derive(Debug, Clone)]
pub struct Relay {
    inner: AtomicDestructor<InternalRelay>,
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

    /// Get [`ActiveSubscription`]
    pub async fn subscriptions(&self) -> HashMap<InternalSubscriptionId, ActiveSubscription> {
        self.inner.subscriptions().await
    }

    /// Get [`ActiveSubscription`] by [`InternalSubscriptionId`]
    pub async fn subscription(
        &self,
        internal_id: &InternalSubscriptionId,
    ) -> Option<ActiveSubscription> {
        self.inner.subscription(internal_id).await
    }

    pub(crate) async fn update_subscription_filters(
        &self,
        internal_id: InternalSubscriptionId,
        filters: Vec<Filter>,
    ) {
        self.inner
            .update_subscription_filters(internal_id, filters)
            .await
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

    /// Send `REQ` to relay
    ///
    /// Automatically close `REQ` if set in [RequestOptions].
    ///
    /// Internally generate a new random [SubscriptionId]. Check `send_req_with_id` method to use a custom [SubscriptionId].
    pub async fn send_req(
        &self,
        filters: Vec<Filter>,
        opts: RequestOptions,
    ) -> Result<SubscriptionId, Error> {
        let id: SubscriptionId = SubscriptionId::generate();
        self.inner.send_req(id.clone(), filters, opts).await?;
        Ok(id)
    }

    /// Send `REQ` to relay with custom [SubscriptionId]
    ///
    /// Automatically close `REQ` if set in [RequestOptions]
    pub async fn send_req_with_id(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: RequestOptions,
    ) -> Result<(), Error> {
        self.inner.send_req(id, filters, opts).await
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
    /// Internal Subscription ID set to `InternalSubscriptionId::Default`
    pub async fn subscribe(
        &self,
        filters: Vec<Filter>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        self.inner.subscribe(filters, opts).await
    }

    /// Subscribe with custom internal ID
    pub async fn subscribe_with_internal_id(
        &self,
        internal_id: InternalSubscriptionId,
        filters: Vec<Filter>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        self.inner
            .subscribe_with_internal_id(internal_id, filters, opts)
            .await
    }

    /// Unsubscribe
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Default`
    pub async fn unsubscribe(&self, opts: RelaySendOptions) -> Result<(), Error> {
        self.inner.unsubscribe(opts).await
    }

    /// Unsubscribe with custom internal id
    pub async fn unsubscribe_with_internal_id(
        &self,
        internal_id: InternalSubscriptionId,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        self.inner
            .unsubscribe_with_internal_id(internal_id, opts)
            .await
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

    /// Request events of filter. All events will be sent to notification listener,
    /// until the EOSE "end of stored events" message is received from the relay.
    pub fn req_events_of(&self, filters: Vec<Filter>, timeout: Duration, opts: FilterOptions) {
        self.inner.req_events_of(filters, timeout, opts)
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
    pub async fn reconcile(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<(), Error> {
        self.inner.reconcile(filter, items, opts).await
    }

    /// Check if relay support negentropy protocol
    pub async fn support_negentropy(&self) -> Result<bool, Error> {
        self.inner.support_negentropy().await
    }
}
