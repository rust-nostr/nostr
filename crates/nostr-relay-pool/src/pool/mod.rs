// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use atomic_destructor::{AtomicDestructor, StealthClone};
use nostr_database::prelude::*;
use tokio::sync::broadcast;

pub mod constants;
mod error;
mod inner;
pub mod options;
mod output;

pub use self::error::Error;
use self::inner::InnerRelayPool;
pub use self::options::RelayPoolOptions;
pub use self::output::Output;
use crate::relay::flags::FlagCheck;
use crate::relay::options::{RelayOptions, ReqExitPolicy, SyncOptions};
use crate::relay::{Relay, RelayFiltering, RelayStatus};
use crate::shared::SharedState;
use crate::stream::ReceiverStream;
use crate::{Reconciliation, RelayServiceFlags, SubscribeOptions};

/// Relay Pool Notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayPoolNotification {
    /// Received an [`Event`]. Does not include events sent by this client.
    Event {
        /// Relay url
        relay_url: RelayUrl,
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// Event
        event: Box<Event>,
    },
    /// Received a [`RelayMessage`]. Includes messages wrapping events that were sent by this client.
    Message {
        /// Relay url
        relay_url: RelayUrl,
        /// Relay Message
        message: RelayMessage,
    },
    /// Relay status changed
    #[deprecated(since = "0.37.0")]
    RelayStatus {
        /// Relay url
        relay_url: RelayUrl,
        /// Relay Status
        status: RelayStatus,
    },
    /// Authenticated to relay
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[deprecated(since = "0.38.0")]
    Authenticated {
        /// Relay url
        relay_url: RelayUrl,
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
        Self::new(RelayPoolOptions::default())
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
    /// Create new `RelayPool`
    #[inline]
    pub fn new(opts: RelayPoolOptions) -> Self {
        Self::__with_shared_state(opts, SharedState::default())
    }

    #[inline]
    #[doc(hidden)]
    pub fn __with_shared_state(opts: RelayPoolOptions, state: SharedState) -> Self {
        Self {
            inner: AtomicDestructor::new(InnerRelayPool::new(opts, state)),
        }
    }

    /// Completely shutdown pool
    ///
    /// This method disconnects and removes all relays from the [`RelayPool`] and then
    /// sends [`RelayPoolNotification::Shutdown`] notification.
    ///
    /// After this method has been called, the [`RelayPool`] can no longer be used (i.e. can't add relays).
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

    /// Get relay filtering
    #[inline]
    pub fn filtering(&self) -> &RelayFiltering {
        self.inner.state.filtering()
    }

    /// Get all relays
    ///
    /// This method return all relays added to the pool, including the ones for gossip protocol or other services.
    #[inline]
    pub async fn all_relays(&self) -> HashMap<RelayUrl, Relay> {
        self.inner.all_relays().await
    }

    /// Get relays with `READ` or `WRITE` flags
    #[inline]
    pub async fn relays(&self) -> HashMap<RelayUrl, Relay> {
        self.inner.relays().await
    }

    /// Get relays that have a certain [RelayServiceFlag] enabled
    #[inline]
    pub async fn relays_with_flag(
        &self,
        flag: RelayServiceFlags,
        check: FlagCheck,
    ) -> HashMap<RelayUrl, Relay> {
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
    /// If are set pool subscriptions, the new added relay will inherit them.
    /// Use [`RelayPool::subscribe_to`] method instead of [`RelayPool::subscribe`],
    /// to avoid to set pool subscriptions.
    ///
    /// Connection is **NOT** automatically started with relay, remember to call [`RelayPool::connect`] or [`RelayPool::connect_relay`]!
    #[inline]
    pub async fn add_relay<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.add_relay(url, true, opts).await
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
        inherit_pool_subscriptions: bool,
        opts: RelayOptions,
    ) -> Result<Option<Relay>, Error> {
        self.inner
            .get_or_add_relay(url, inherit_pool_subscriptions, opts)
            .await
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
    ///
    /// This method may not remove all relays.
    /// Use [`RelayPool::force_remove_all_relays`] to remove every relay.
    #[inline]
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        self.inner.remove_all_relays(false).await
    }

    /// Disconnect and force remove all relays
    #[inline]
    pub async fn force_remove_all_relays(&self) -> Result<(), Error> {
        self.inner.remove_all_relays(true).await
    }

    /// Connect to all added relays
    #[inline]
    pub async fn connect(&self) {
        self.inner.connect().await
    }

    /// Waits for relays connections
    ///
    /// Wait for relays connections at most for the specified `timeout`.
    /// The code continues when the relays are connected or the `timeout` is reached.
    #[inline]
    pub async fn wait_for_connection(&self, timeout: Duration) {
        self.inner.wait_for_connection(timeout).await
    }

    /// Try to establish a connection with the relays.
    ///
    /// Attempts to establish a connection without spawning the connection task if it fails.
    /// This means that if the connection fails, no automatic retries are scheduled.
    /// Use [`RelayPool::connect`] if you want to immediately spawn a connection task,
    /// regardless of whether the initial connection succeeds.
    ///
    /// For further details, see the documentation of [`Relay::try_connect`].
    #[inline]
    pub async fn try_connect(&self, timeout: Duration) -> Output<()> {
        self.inner.try_connect(timeout).await
    }

    /// Disconnect from all relays
    #[inline]
    pub async fn disconnect(&self) -> Result<(), Error> {
        self.inner.disconnect().await
    }

    /// Connect to a previously added relay
    ///
    /// This method doesn't provide any information on if the connection was successful or not.
    ///
    /// Return [`Error::RelayNotFound`] if the relay doesn't exist in the pool.
    #[inline]
    pub async fn connect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.connect_relay(url).await
    }

    /// Try to connect to a previously added relay
    ///
    /// For further details, see the documentation of [`Relay::try_connect`].
    #[inline]
    pub async fn try_connect_relay<U>(&self, url: U, timeout: Duration) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.try_connect_relay(url, timeout).await
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
    pub async fn send_msg_to<I, U>(&self, urls: I, msg: ClientMessage) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.send_msg_to(urls, msg).await
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    #[inline]
    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.batch_msg_to(urls, msgs).await
    }

    /// Send event to all relays with `WRITE` flag (check [`RelayServiceFlags`] for more details).
    #[inline]
    pub async fn send_event(&self, event: Event) -> Result<Output<EventId>, Error> {
        self.inner.send_event(event).await
    }

    /// Send multiple events at once to all relays with `WRITE` flag (check [`RelayServiceFlags`] for more details).
    #[deprecated(since = "0.38.0")]
    pub async fn batch_event(&self, _events: Vec<Event>) -> Result<Output<()>, Error> {
        unimplemented!()
    }

    /// Send event to specific relays
    #[inline]
    pub async fn send_event_to<I, U>(&self, urls: I, event: Event) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.send_event_to(urls, event).await
    }

    /// Send multiple events at once to specific relays
    #[deprecated(since = "0.38.0")]
    pub async fn batch_event_to<I, U>(
        &self,
        _urls: I,
        _events: Vec<Event>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        unimplemented!()
    }

    /// Subscribe to filters to all relays with `READ` flag.
    ///
    /// Check [`RelayPool::subscribe_with_id_to`] docs to learn more.
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
    /// Check [`RelayPool::subscribe_with_id_to`] docs to learn more.
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
    /// Check [`RelayPool::subscribe_with_id_to`] docs to learn more.
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
    /// This method doesn't add relays!
    /// All the relays must be added to the pool with [`RelayPool::add_relay`].
    /// If the specified relays don't exist, [`Error::RelayNotFound`] is returned.
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    ///
    /// Auto-closing subscriptions aren't saved in the subscription map!
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
    /// Subscribe to specific relays with specific filters.
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
    pub async fn unsubscribe(&self, id: SubscriptionId) {
        self.inner.unsubscribe(id).await
    }

    /// Unsubscribe from all subscriptions
    #[inline]
    pub async fn unsubscribe_all(&self) {
        self.inner.unsubscribe_all().await
    }

    /// Sync events with relays (negentropy reconciliation)
    #[inline]
    pub async fn sync(
        &self,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        self.inner.sync(filter, opts).await
    }

    /// Sync events with specific relays (negentropy reconciliation)
    #[inline]
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
        self.inner.sync_with(urls, filter, opts).await
    }

    /// Sync events with specific relays and filters (negentropy reconciliation)
    #[inline]
    pub async fn sync_targeted<I, U>(
        &self,
        targets: I,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = (U, HashMap<Filter, Vec<(EventId, Timestamp)>>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner.sync_targeted(targets, opts).await
    }

    /// Fetch events from relays with [`RelayServiceFlags::READ`] flag.
    #[inline]
    pub async fn fetch_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<Events, Error> {
        self.inner.fetch_events(filters, timeout, policy).await
    }

    /// Fetch events from specific relays
    #[inline]
    pub async fn fetch_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<Events, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .fetch_events_from(urls, filters, timeout, policy)
            .await
    }

    /// Stream events from relays with `READ` flag.
    #[inline]
    pub async fn stream_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<ReceiverStream<Event>, Error> {
        self.inner.stream_events(filters, timeout, policy).await
    }

    /// Stream events from specific relays
    #[inline]
    pub async fn stream_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .stream_events_from(urls, filters, timeout, policy)
            .await
    }

    /// Targeted streaming events
    ///
    /// Stream events from specific relays with specific filters
    #[inline]
    pub async fn stream_events_targeted<I, U>(
        &self,
        source: I,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = (U, Vec<Filter>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.inner
            .stream_events_targeted(source, timeout, policy)
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

#[cfg(test)]
mod tests {
    use nostr_relay_builder::MockRelay;

    use super::*;

    #[tokio::test]
    async fn test_shutdown() {
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url();

        let pool = RelayPool::default();

        pool.add_relay(&url, RelayOptions::default()).await.unwrap();

        pool.connect().await;

        assert!(!pool.inner.is_shutdown());

        tokio::time::sleep(Duration::from_secs(1)).await;

        pool.shutdown().await.unwrap();

        assert!(pool.inner.is_shutdown());

        assert!(matches!(
            pool.add_relay(&url, RelayOptions::default())
                .await
                .unwrap_err(),
            Error::Shutdown
        ));
    }
}
