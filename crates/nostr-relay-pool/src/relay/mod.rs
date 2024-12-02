// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use async_wsocket::futures_util::Future;
use async_wsocket::ConnectionMode;
use atomic_destructor::AtomicDestructor;
use nostr_database::prelude::*;
use tokio::sync::broadcast;

pub mod constants;
mod error;
mod filtering;
pub mod flags;
mod inner;
pub mod limits;
pub mod options;
mod ping;
pub mod stats;
mod status;

pub use self::error::Error;
pub use self::filtering::{RelayFiltering, RelayFilteringMode};
pub use self::flags::{AtomicRelayServiceFlags, FlagCheck, RelayServiceFlags};
use self::inner::InnerRelay;
pub use self::limits::RelayLimits;
pub use self::options::{
    FilterOptions, RelayOptions, SubscribeAutoCloseOptions, SubscribeOptions, SyncDirection,
    SyncOptions, SyncProgress,
};
pub use self::stats::RelayConnectionStats;
pub use self::status::RelayStatus;

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
    /// Authenticated to relay
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    Authenticated,
    /// Shutdown
    Shutdown,
}

// #[derive(Debug, Clone, Default, PartialEq, Eq)]
// pub struct ReconciliationFailures {
//     /// Send failures
//     pub send: HashMap<EventId, Vec<String>>,
//     // Receive failures (NOT CURRENTLY AVAILABLE)
//     // pub receive: HashMap<EventId, Vec<String>>,
// }

/// Reconciliation output
#[derive(Debug, Clone, Default)]
pub struct Reconciliation {
    /// Events that were stored locally (missing on relay)
    pub local: HashSet<EventId>,
    /// Events that were stored on relay (missing locally)
    pub remote: HashSet<EventId>,
    /// Events that are **successfully** sent to relays during reconciliation
    pub sent: HashSet<EventId>,
    /// Event that are **successfully** received from relay during reconciliation
    pub received: HashSet<EventId>,
    /// Send failures
    pub send_failures: HashMap<RelayUrl, HashMap<EventId, String>>,
}

impl Reconciliation {
    pub(crate) fn merge(&mut self, other: Reconciliation) {
        self.local.extend(other.local);
        self.remote.extend(other.remote);
        self.sent.extend(other.sent);
        self.received.extend(other.received);
        self.send_failures.extend(other.send_failures);
    }
}

/// Relay
#[derive(Debug, Clone)]
pub struct Relay {
    pub(crate) inner: AtomicDestructor<InnerRelay>,
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
    /// Create new relay with **default** options and in-memory database
    pub fn new(url: RelayUrl) -> Self {
        Self::with_opts(url, RelayOptions::default())
    }

    /// Create new relay with default in-memory database and custom options
    pub fn with_opts(url: RelayUrl, opts: RelayOptions) -> Self {
        let database = Arc::new(MemoryDatabase::default());
        Self::custom(url, database, opts)
    }

    /// Create new relay with **custom** database and/or options
    pub fn custom<T>(url: RelayUrl, database: T, opts: RelayOptions) -> Self
    where
        T: IntoNostrDatabase,
    {
        let database: Arc<dyn NostrDatabase> = database.into_nostr_database();
        let filtering: RelayFiltering = RelayFiltering::new(opts.filtering_mode);
        Self::internal_custom(url, database, filtering, opts)
    }

    pub(crate) fn internal_custom(
        url: RelayUrl,
        database: Arc<dyn NostrDatabase>,
        filtering: RelayFiltering,
        opts: RelayOptions,
    ) -> Self {
        Self {
            inner: AtomicDestructor::new(InnerRelay::new(url, database, filtering, opts)),
        }
    }

    /// Get relay url
    #[inline]
    pub fn url(&self) -> &RelayUrl {
        &self.inner.url
    }

    /// Get connection mode
    #[inline]
    pub fn connection_mode(&self) -> &ConnectionMode {
        self.inner.connection_mode()
    }

    /// Get status
    #[inline]
    pub fn status(&self) -> RelayStatus {
        self.inner.status()
    }

    /// Get Relay Service Flags
    #[inline]
    pub fn flags(&self) -> &AtomicRelayServiceFlags {
        &self.inner.flags
    }

    /// Get relay filtering
    #[inline]
    pub fn filtering(&self) -> &RelayFiltering {
        &self.inner.filtering
    }

    /// Check if relay is connected
    #[inline]
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Get [`RelayInformationDocument`]
    #[inline]
    #[cfg(feature = "nip11")]
    pub async fn document(&self) -> RelayInformationDocument {
        self.inner.document().await
    }

    /// Get subscriptions
    #[inline]
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        self.inner.subscriptions().await
    }

    /// Get filters by [SubscriptionId]
    #[inline]
    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        self.inner.subscription(id).await
    }

    /// Get options
    #[inline]
    pub fn opts(&self) -> &RelayOptions {
        &self.inner.opts
    }

    /// Get [`RelayConnectionStats`]
    #[inline]
    pub fn stats(&self) -> &RelayConnectionStats {
        &self.inner.stats
    }

    /// Get queue len
    #[inline]
    pub fn queue(&self) -> usize {
        self.inner.queue()
    }

    /// Get new **relay** notification listener
    ///
    /// <div class="warning">When you call this method, you subscribe to the notifications channel from that precise moment. Anything received by relay/s before that moment is not included in the channel!</div>
    #[inline]
    pub fn notifications(&self) -> broadcast::Receiver<RelayNotification> {
        self.inner.internal_notification_sender.subscribe()
    }

    /// Connect to relay and keep alive connection
    #[inline]
    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        self.inner.connect(connection_timeout).await
    }

    /// Disconnect from relay and set status to 'Terminated'
    #[inline]
    pub fn disconnect(&self) -> Result<(), Error> {
        self.inner.disconnect()
    }

    /// Send msg to relay
    #[inline]
    pub fn send_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        self.batch_msg(vec![msg])
    }

    /// Send multiple [`ClientMessage`] at once
    #[inline]
    pub fn batch_msg(&self, msgs: Vec<ClientMessage>) -> Result<(), Error> {
        self.inner.batch_msg(msgs)
    }

    /// Send event and wait for `OK` relay msg
    #[inline]
    pub async fn send_event(&self, event: Event) -> Result<EventId, Error> {
        self.inner.send_event(event).await
    }

    /// Send multiple [`Event`] at once
    #[deprecated(since = "0.38.0")]
    pub async fn batch_event(&self, _events: Vec<Event>) -> Result<(), Error> {
        unimplemented!()
    }

    /// Send client authentication event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub async fn auth(&self, event: Event) -> Result<(), Error> {
        self.inner.auth(event).await
    }

    /// Resubscribe to all **closed** or not yet initiated subscriptions
    #[inline]
    pub async fn resubscribe(&self) -> Result<(), Error> {
        self.inner.resubscribe().await
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
    #[inline]
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
    #[inline]
    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<(), Error> {
        self.inner.subscribe_with_id(id, filters, opts).await
    }

    /// Unsubscribe
    #[inline]
    pub async fn unsubscribe(&self, id: SubscriptionId) -> Result<(), Error> {
        self.inner.unsubscribe(id).await
    }

    /// Unsubscribe from all subscriptions
    #[inline]
    pub async fn unsubscribe_all(&self) -> Result<(), Error> {
        self.inner.unsubscribe_all().await
    }

    /// Get events of filters with custom callback
    #[inline]
    pub(crate) async fn fetch_events_with_callback<F>(
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
            .fetch_events_with_callback(filters, timeout, opts, callback)
            .await
    }

    /// Fetch events
    #[inline]
    pub async fn fetch_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Events, Error> {
        self.inner.fetch_events(filters, timeout, opts).await
    }

    /// Count events
    #[inline]
    pub async fn count_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<usize, Error> {
        self.inner.count_events(filters, timeout).await
    }

    /// Sync events with relays (negentropy reconciliation)
    #[inline]
    pub async fn sync(&self, filter: Filter, opts: &SyncOptions) -> Result<Reconciliation, Error> {
        self.inner.sync(filter, opts).await
    }

    /// Sync events with relays (negentropy reconciliation)
    #[inline]
    pub async fn sync_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: &SyncOptions,
    ) -> Result<Reconciliation, Error> {
        self.inner.sync_with_items(filter, items, opts).await
    }

    /// Sync events with relays (negentropy reconciliation)
    #[inline]
    pub async fn sync_multi(
        &self,
        map: HashMap<Filter, Vec<(EventId, Timestamp)>>,
        opts: &SyncOptions,
    ) -> Result<Reconciliation, Error> {
        self.inner.sync_multi(map, opts).await
    }

    /// Handle notifications
    pub async fn handle_notifications<F, Fut>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(RelayNotification) -> Fut,
        Fut: Future<Output = Result<bool>>,
    {
        let mut notifications = self.notifications();
        while let Ok(notification) = notifications.recv().await {
            let shutdown: bool = RelayNotification::Shutdown == notification;
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

#[cfg(test)]
mod tests {
    use async_utility::time;
    use nostr_relay_builder::prelude::*;

    use super::*;

    #[tokio::test]
    async fn test_ok_msg() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = Relay::new(url);

        relay.connect(Some(Duration::from_millis(100))).await;

        let keys = Keys::generate();
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        relay.send_event(event).await.unwrap();
    }

    #[tokio::test]
    async fn test_status_with_reconnection_enabled() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = Relay::new(url);

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect(Some(Duration::from_millis(100))).await;

        assert_eq!(relay.status(), RelayStatus::Connected);

        mock.shutdown();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Disconnected);

        assert!(relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_status_with_reconnection_disabled() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = Relay::with_opts(url, RelayOptions::default().reconnect(false));

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect(Some(Duration::from_millis(100))).await;

        assert_eq!(relay.status(), RelayStatus::Connected);

        mock.shutdown();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_disconnect() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = Relay::new(url);

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect(Some(Duration::from_millis(100))).await;

        assert_eq!(relay.status(), RelayStatus::Connected);

        relay.disconnect().unwrap();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_disconnect_non_connected_relay() {
        let url = RelayUrl::parse("wss://127.0.0.1:666").unwrap();

        let opts = RelayOptions::default()
            .adjust_retry_interval(false)
            .retry_interval(Duration::from_secs(1));
        let relay = Relay::with_opts(url, opts);

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect(Some(Duration::from_millis(100))).await;

        assert!(relay.inner.is_running());

        assert_eq!(relay.status(), RelayStatus::Disconnected);

        time::sleep(Duration::from_secs(3)).await;

        relay.disconnect().unwrap();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_disconnect_unresponsive_relay_that_connect() {
        // Mock relay
        let opts = RelayTestOptions {
            unresponsive_connection: Some(Duration::from_secs(2)),
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = Relay::new(url);

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect(None).await;

        time::sleep(Duration::from_secs(1)).await;

        assert_eq!(relay.status(), RelayStatus::Connecting);

        time::sleep(Duration::from_secs(2)).await;

        assert_eq!(relay.status(), RelayStatus::Connected);

        relay.disconnect().unwrap();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_disconnect_unresponsive_relay_that_not_connect() {
        // Mock relay
        let opts = RelayTestOptions {
            unresponsive_connection: Some(Duration::from_secs(10)),
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = Relay::new(url);

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect(None).await;

        time::sleep(Duration::from_secs(1)).await;

        assert_eq!(relay.status(), RelayStatus::Connecting);

        relay.disconnect().unwrap();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }
}
