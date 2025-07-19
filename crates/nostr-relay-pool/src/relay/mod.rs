// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay

use std::borrow::Cow;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use async_utility::time;
use async_wsocket::futures_util::Future;
use async_wsocket::ConnectionMode;
use atomic_destructor::AtomicDestructor;
use nostr_database::prelude::*;
use tokio::sync::{broadcast, mpsc};

pub mod constants;
mod error;
pub mod flags;
mod inner;
pub mod limits;
pub mod options;
mod ping;
pub mod stats;
mod status;

use self::constants::{WAIT_FOR_AUTHENTICATION_TIMEOUT, WAIT_FOR_OK_TIMEOUT};
pub use self::error::Error;
pub use self::flags::{AtomicRelayServiceFlags, FlagCheck, RelayServiceFlags};
use self::inner::InnerRelay;
pub use self::limits::RelayLimits;
pub use self::options::{
    RelayOptions, ReqExitPolicy, SubscribeAutoCloseOptions, SubscribeOptions, SyncDirection,
    SyncOptions, SyncProgress,
};
pub use self::stats::RelayConnectionStats;
pub use self::status::RelayStatus;
use crate::policy::AdmitStatus;
use crate::shared::SharedState;
use crate::transport::websocket::{BoxSink, BoxStream};

/// Subscription auto-closed reason
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionAutoClosedReason {
    /// NIP42 authentication failed
    AuthenticationFailed,
    /// Closed
    Closed(String),
    /// Completed
    Completed,
}

#[derive(Debug)]
enum SubscriptionActivity {
    /// Received an event
    ReceivedEvent(Event),
    /// Subscription closed
    Closed(SubscriptionAutoClosedReason),
}

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
        message: RelayMessage<'static>,
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
    /// Authentication failed
    AuthenticationFailed,
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    #[inline]
    pub(crate) fn new(url: RelayUrl, state: SharedState, opts: RelayOptions) -> Self {
        Self {
            inner: AtomicDestructor::new(InnerRelay::new(url, state, opts)),
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

    /// Check if relay is connected
    pub fn is_connected(&self) -> bool {
        self.status().is_connected()
    }

    /// Get Relay Service Flags
    #[inline]
    pub fn flags(&self) -> &AtomicRelayServiceFlags {
        &self.inner.flags
    }

    /// Get subscriptions
    #[inline]
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Filter> {
        self.inner.subscriptions().await
    }

    /// Get filters by [SubscriptionId]
    #[inline]
    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Filter> {
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

    /// Connect to the relay
    ///
    /// # Overview
    ///
    /// If the relay’s status is not [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`],
    /// this method returns immediately without doing anything.
    /// Otherwise, the connection task will be spawned, which will attempt to connect to relay.
    ///
    /// This method returns immediately and doesn't provide any information on if the connection was successful or not.
    ///
    /// # Automatic reconnection
    ///
    /// By default, in case of disconnection, the connection task will automatically attempt to reconnect.
    /// This behavior can be disabled by changing [`RelayOptions::reconnect`] option.
    pub fn connect(&self) {
        // Immediately return if can't connect
        if !self.status().can_connect() {
            return;
        }

        // Update status
        // Change it to pending to avoid issues with the health check (initialized check)
        self.inner.set_status(RelayStatus::Pending, false);

        // Spawn connection task
        self.inner.spawn_connection_task(None);
    }

    /// Waits for relay connection
    ///
    /// Wait for relay connection at most for the specified `timeout`.
    /// The code continues when the relay is connected or the `timeout` is reached.
    pub async fn wait_for_connection(&self, timeout: Duration) {
        let status: RelayStatus = self.status();

        // Immediately returns if the relay is already connected, if it's terminated or banned.
        if status.is_connected() || status.is_terminated() || status.is_banned() {
            return;
        }

        // Subscribe to notifications
        let mut notifications = self.inner.internal_notification_sender.subscribe();

        // Set timeout
        time::timeout(Some(timeout), async {
            while let Ok(notification) = notifications.recv().await {
                // Wait for status change. Break loop when connect.
                if let RelayNotification::RelayStatus { status } = notification {
                    match status {
                        // Waiting for connection
                        RelayStatus::Initialized
                        | RelayStatus::Pending
                        | RelayStatus::Connecting
                        | RelayStatus::Disconnected => {}
                        // Connected or terminated/banned/sleeping
                        RelayStatus::Connected
                        | RelayStatus::Terminated
                        | RelayStatus::Banned
                        | RelayStatus::Sleeping => break,
                    }
                }
            }
        })
        .await;
    }

    /// Try to establish a connection with the relay.
    ///
    /// # Overview
    ///
    /// If the relay’s status is not [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`],
    /// this method returns immediately without doing anything.
    /// Otherwise, attempts to establish a connection without spawning the connection task if it fails.
    /// This means that if the connection fails, no automatic retries are scheduled.
    /// Use [`Relay::connect`] if you want to immediately spawn a connection task,
    /// regardless of whether the initial connection succeeds.
    ///
    /// Returns an error if the connection fails or if the relay has been banned.
    ///
    /// # Automatic reconnection
    ///
    /// By default, in case of disconnection (after a first successful connection),
    /// the connection task will automatically attempt to reconnect.
    /// This behavior can be disabled by changing [`RelayOptions::reconnect`] option.
    pub async fn try_connect(&self, timeout: Duration) -> Result<(), Error> {
        let status: RelayStatus = self.status();

        if status.is_banned() {
            return Err(Error::Banned);
        }

        // Check if relay can't connect
        if !status.can_connect() {
            return Ok(());
        }

        // Check connection policy
        if let AdmitStatus::Rejected { reason } = self.inner.check_connection_policy().await? {
            // Set status to "terminated"
            self.inner.set_status(RelayStatus::Terminated, false);

            // Return error
            return Err(Error::ConnectionRejected { reason });
        }

        // Try to connect
        // This will set the status to "terminated" if the connection fails
        let stream: (BoxSink, BoxStream) = self
            .inner
            ._try_connect(timeout, RelayStatus::Terminated)
            .await?;

        // Spawn connection task
        self.inner.spawn_connection_task(Some(stream));

        Ok(())
    }

    /// Disconnect from relay and set status to [`RelayStatus::Terminated`].
    #[inline]
    pub fn disconnect(&self) {
        self.inner.disconnect()
    }

    /// Ban relay and set status to [`RelayStatus::Banned`].
    ///
    /// A banned relay can't reconnect again.
    #[inline]
    pub fn ban(&self) {
        self.inner.ban()
    }

    /// Send msg to relay
    #[inline]
    pub fn send_msg(&self, msg: ClientMessage<'_>) -> Result<(), Error> {
        self.inner.send_msg(msg)
    }

    /// Send multiple [`ClientMessage`] at once
    #[inline]
    pub fn batch_msg(&self, msgs: Vec<ClientMessage<'_>>) -> Result<(), Error> {
        self.inner.batch_msg(msgs)
    }

    async fn _send_event(
        &self,
        notifications: &mut broadcast::Receiver<RelayNotification>,
        event: &Event,
    ) -> Result<(bool, String), Error> {
        // Send the EVENT message
        self.inner
            .send_msg(ClientMessage::Event(Cow::Borrowed(event)))?;

        // Wait for OK
        self.inner
            .wait_for_ok(notifications, &event.id, WAIT_FOR_OK_TIMEOUT)
            .await
    }

    /// Send event and wait for `OK` relay msg
    pub async fn send_event(&self, event: &Event) -> Result<EventId, Error> {
        // Health, write permission and number of messages checks are executed in `batch_msg` method.

        // Subscribe to notifications
        let mut notifications = self.inner.internal_notification_sender.subscribe();

        // Send event
        let (status, message) = self._send_event(&mut notifications, event).await?;

        // Check status
        if status {
            return Ok(event.id);
        }

        // If auth required, wait for authentication adn resend it
        if let Some(MachineReadablePrefix::AuthRequired) = MachineReadablePrefix::parse(&message) {
            // Check if NIP42 auth is enabled and signer is set
            let has_signer: bool = self.inner.state.has_signer().await;
            if self.inner.state.is_auto_authentication_enabled() && has_signer {
                // Wait that relay authenticate
                self.wait_for_authentication(&mut notifications, WAIT_FOR_AUTHENTICATION_TIMEOUT)
                    .await?;

                // Try to resend event
                let (status, message) = self._send_event(&mut notifications, event).await?;

                // Check status
                return if status {
                    Ok(event.id)
                } else {
                    Err(Error::RelayMessage(message))
                };
            }
        }

        Err(Error::RelayMessage(message))
    }

    async fn wait_for_authentication(
        &self,
        notifications: &mut broadcast::Receiver<RelayNotification>,
        timeout: Duration,
    ) -> Result<(), Error> {
        time::timeout(Some(timeout), async {
            while let Ok(notification) = notifications.recv().await {
                match notification {
                    RelayNotification::Authenticated => {
                        return Ok(());
                    }
                    RelayNotification::AuthenticationFailed => {
                        return Err(Error::AuthenticationFailed);
                    }
                    RelayNotification::RelayStatus { status } => {
                        if status.is_disconnected() {
                            return Err(Error::NotConnected);
                        }
                    }
                    RelayNotification::Shutdown => break,
                    _ => (),
                }
            }

            Err(Error::PrematureExit)
        })
        .await
        .ok_or(Error::Timeout)?
    }

    /// Resubscribe to all **closed** or not yet initiated subscriptions
    #[inline]
    pub async fn resubscribe(&self) -> Result<(), Error> {
        self.inner.resubscribe().await
    }

    /// Subscribe to filters
    ///
    /// Internally generate a new random [`SubscriptionId`]. Check `subscribe_with_id` method to use a custom [SubscriptionId].
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub async fn subscribe(
        &self,
        filters: Filter,
        opts: SubscribeOptions,
    ) -> Result<SubscriptionId, Error> {
        let id: SubscriptionId = SubscriptionId::generate();
        self.subscribe_with_id(id.clone(), filters, opts).await?;
        Ok(id)
    }

    /// Subscribe with custom [`SubscriptionId`]
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filter: Filter,
        opts: SubscribeOptions,
    ) -> Result<(), Error> {
        // Check if the auto-close condition is set
        match opts.auto_close {
            Some(opts) => self.subscribe_auto_closing(id, filter, opts, None).await,
            None => self.subscribe_long_lived(id, filter).await,
        }
    }

    async fn subscribe_auto_closing(
        &self,
        id: SubscriptionId,
        filter: Filter,
        opts: SubscribeAutoCloseOptions,
        activity: Option<mpsc::Sender<SubscriptionActivity>>,
    ) -> Result<(), Error> {
        // Compose REQ message
        let msg: ClientMessage = ClientMessage::Req {
            subscription_id: Cow::Borrowed(&id),
            filter: Cow::Borrowed(&filter),
        };

        // Subscribe to notifications
        let notifications = self.inner.internal_notification_sender.subscribe();

        // Register the auto-closing subscription
        self.inner
            .add_auto_closing_subscription(id.clone(), filter.clone())
            .await;

        // Send REQ message
        if let Err(e) = self.inner.send_msg(msg) {
            // Remove previously added subscription
            self.inner.remove_subscription(&id).await;

            // Propagate error
            return Err(e);
        }

        // Spawn auto-closing handler
        self.inner
            .spawn_auto_closing_handler(id, filter, opts, notifications, activity);

        // Return
        Ok(())
    }

    async fn subscribe_long_lived(&self, id: SubscriptionId, filter: Filter) -> Result<(), Error> {
        // Compose REQ message
        let msg: ClientMessage = ClientMessage::Req {
            subscription_id: Cow::Borrowed(&id),
            filter: Cow::Borrowed(&filter),
        };

        // Send REQ message
        self.inner.send_msg(msg)?;

        // No auto-close subscription: update subscription filter
        self.inner.update_subscription(id, filter, true).await;

        // Return
        Ok(())
    }

    /// Unsubscribe
    #[inline]
    pub async fn unsubscribe(&self, id: &SubscriptionId) -> Result<(), Error> {
        self.inner.unsubscribe(id).await
    }

    /// Unsubscribe from all subscriptions
    #[inline]
    pub async fn unsubscribe_all(&self) -> Result<(), Error> {
        self.inner.unsubscribe_all().await
    }

    /// Get events of filter with custom callback
    pub(crate) async fn fetch_events_with_callback(
        &self,
        filter: Filter,
        timeout: Duration,
        policy: ReqExitPolicy,
        mut callback: impl FnMut(Event),
    ) -> Result<(), Error> {
        // Create channel
        let (tx, mut rx) = mpsc::channel(512);

        // Compose auto-closing options
        let opts: SubscribeAutoCloseOptions = SubscribeAutoCloseOptions::default()
            .exit_policy(policy)
            .timeout(Some(timeout));

        // Subscribe
        let id: SubscriptionId = SubscriptionId::generate();
        self.subscribe_auto_closing(id, filter, opts, Some(tx))
            .await?;

        // Handle subscription activity
        while let Some(activity) = rx.recv().await {
            match activity {
                SubscriptionActivity::ReceivedEvent(event) => {
                    callback(event);
                }
                SubscriptionActivity::Closed(reason) => {
                    match reason {
                        // NIP42 authentication failed
                        SubscriptionAutoClosedReason::AuthenticationFailed => {
                            return Err(Error::AuthenticationFailed);
                        }
                        // Closed by relay
                        SubscriptionAutoClosedReason::Closed(message) => {
                            return Err(Error::RelayMessage(message));
                        }
                        // Completed
                        SubscriptionAutoClosedReason::Completed => break,
                    }
                }
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) async fn fetch_events_with_callback_owned(
        self,
        filter: Filter,
        timeout: Duration,
        policy: ReqExitPolicy,
        callback: impl Fn(Event),
    ) -> Result<(), Error> {
        self.fetch_events_with_callback(filter, timeout, policy, callback)
            .await
    }

    /// Fetch events
    pub async fn fetch_events(
        &self,
        filter: Filter,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<Events, Error> {
        let mut events: Events = Events::new(&filter);
        self.fetch_events_with_callback(filter, timeout, policy, |event| {
            // Use force insert here!
            // Due to the configurable REQ exit policy, the user may want to wait for events after EOSE.
            // If the filter had a limit, the force insert allows adding events post-EOSE.
            //
            // For example, if we use `Events::insert` here,
            // if the filter is '{"kinds":[1],"limit":3}' and the policy `ReqExitPolicy::WaitForEventsAfterEOSE(1)`,
            // the events collection will discard 1 event because the filter limit is 3 and the total received events are 4.
            //
            // Events::force_insert automatically increases the capacity if needed, without discarding events.
            //
            // LOOKUP_ID: EVENTS_FORCE_INSERT
            events.force_insert(event);
        })
        .await?;
        Ok(events)
    }

    /// Count events
    pub async fn count_events(&self, filter: Filter, timeout: Duration) -> Result<usize, Error> {
        let id = SubscriptionId::generate();
        let msg = ClientMessage::Count {
            subscription_id: Cow::Borrowed(&id),
            filter: Cow::Owned(filter),
        };
        self.inner.send_msg(msg)?;

        let mut count = 0;

        let mut notifications = self.inner.internal_notification_sender.subscribe();
        time::timeout(Some(timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayNotification::Message {
                    message:
                        RelayMessage::Count {
                            subscription_id,
                            count: c,
                        },
                } = notification
                {
                    if subscription_id.as_ref() == &id {
                        count = c;
                        break;
                    }
                }
            }
        })
        .await
        .ok_or(Error::Timeout)?;

        // Unsubscribe
        self.inner.send_msg(ClientMessage::close(id))?;

        Ok(count)
    }

    /// Sync events with relays (negentropy reconciliation)
    pub async fn sync(&self, filter: Filter, opts: &SyncOptions) -> Result<Reconciliation, Error> {
        let items = self
            .inner
            .state
            .database()
            .negentropy_items(filter.clone())
            .await?;
        self.sync_with_items(filter, items, opts).await
    }

    /// Sync events with relays (negentropy reconciliation)
    pub async fn sync_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: &SyncOptions,
    ) -> Result<Reconciliation, Error> {
        // Check if relay is operational
        self.inner.ensure_operational()?;

        // Check if relay can read
        if !self.inner.flags.can_read() {
            return Err(Error::ReadDisabled);
        }

        let mut output: Reconciliation = Reconciliation::default();

        self.inner
            .sync(&filter, items.clone(), opts, &mut output)
            .await?;

        Ok(output)
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
    use std::sync::Arc;

    use async_utility::time;
    use nostr_relay_builder::prelude::*;

    use super::{Error, *};
    use crate::policy::{AdmitPolicy, PolicyError};

    #[derive(Debug)]
    struct CustomTestPolicy {
        banned_relays: HashSet<RelayUrl>,
    }

    impl AdmitPolicy for CustomTestPolicy {
        fn admit_connection<'a>(
            &'a self,
            relay_url: &'a RelayUrl,
        ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
            Box::pin(async move {
                if self.banned_relays.contains(relay_url) {
                    Ok(AdmitStatus::rejected("banned"))
                } else {
                    Ok(AdmitStatus::Success)
                }
            })
        }
    }

    fn new_relay(url: RelayUrl, opts: RelayOptions) -> Relay {
        Relay::new(url, SharedState::default(), opts)
    }

    fn new_relay_with_database(
        url: RelayUrl,
        database: Arc<dyn NostrDatabase>,
        opts: RelayOptions,
    ) -> Relay {
        let mut state = SharedState::default();
        state.database = database;
        Relay::new(url, state, opts)
    }

    /// Setup public (without NIP42 auth) relay with N events to test event fetching
    ///
    /// **Adds ONLY text notes**
    async fn setup_event_fetching_relay(num_events: usize) -> (Relay, MockRelay) {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = new_relay(url, RelayOptions::default());
        relay.connect();

        // Signer
        let keys = Keys::generate();

        // Send some events
        for i in 0..num_events {
            let event = EventBuilder::text_note(i.to_string())
                .sign_with_keys(&keys)
                .unwrap();
            relay.send_event(&event).await.unwrap();
        }

        (relay, mock)
    }

    async fn setup_subscription_relay() -> (SubscriptionId, Relay, MockRelay) {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        // Sender
        let relay: Relay = new_relay(url.clone(), RelayOptions::default());
        relay.connect();

        // Subscribe
        let filter = Filter::new().kind(Kind::TextNote);
        let id = relay
            .subscribe(filter, SubscribeOptions::default())
            .await
            .unwrap();

        (id, relay, mock)
    }

    fn check_relay_is_sleeping(relay: &Relay) {
        assert_eq!(relay.status(), RelayStatus::Sleeping);
        assert!(relay.status().can_connect());
        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_ok_msg() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        relay.try_connect(Duration::from_secs(3)).await.unwrap();

        let keys = Keys::generate();
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        relay.send_event(&event).await.unwrap();
    }

    #[tokio::test]
    async fn test_status_with_reconnection_enabled() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.try_connect(Duration::from_secs(3)).await.unwrap();

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

        let relay: Relay = new_relay(url, RelayOptions::default().reconnect(false));

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.try_connect(Duration::from_secs(3)).await.unwrap();

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

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.try_connect(Duration::from_secs(3)).await.unwrap();

        assert_eq!(relay.status(), RelayStatus::Connected);

        relay.disconnect();

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
        let relay: Relay = new_relay(url, opts);

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect();

        time::sleep(Duration::from_secs(1)).await;

        assert!(relay.inner.is_running());

        assert_eq!(relay.status(), RelayStatus::Disconnected);

        time::sleep(Duration::from_secs(3)).await;

        relay.disconnect();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_connect() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect();

        relay.wait_for_connection(Duration::from_secs(1)).await;

        assert_eq!(relay.status(), RelayStatus::Connected);
        assert!(relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_connect_to_unreachable_relay() {
        let url = RelayUrl::parse("wss://127.0.0.1:666").unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect();

        time::sleep(Duration::from_secs(1)).await;

        assert_eq!(relay.status(), RelayStatus::Disconnected);
        assert!(relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_try_connect() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.try_connect(Duration::from_millis(500)).await.unwrap();

        assert_eq!(relay.status(), RelayStatus::Connected);

        time::sleep(Duration::from_millis(500)).await;

        assert!(relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_try_connect_to_unreachable_relay() {
        let url = RelayUrl::parse("wss://127.0.0.1:666").unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        let res = relay.try_connect(Duration::from_secs(2)).await;
        assert!(matches!(res.unwrap_err(), Error::Transport(..)));

        assert_eq!(relay.status(), RelayStatus::Terminated);

        // Connection failed, the connection task is not running
        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_disconnect_unresponsive_relay_that_connect() {
        // Mock relay
        let opts = RelayTestOptions {
            unresponsive_connection: Some(Duration::from_secs(2)),
            ..Default::default()
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect();

        time::sleep(Duration::from_secs(1)).await;

        assert_eq!(relay.status(), RelayStatus::Connecting);

        time::sleep(Duration::from_secs(2)).await;

        assert_eq!(relay.status(), RelayStatus::Connected);

        relay.disconnect();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_disconnect_unresponsive_relay_that_not_connect() {
        // Mock relay
        let opts = RelayTestOptions {
            unresponsive_connection: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect();

        time::sleep(Duration::from_secs(1)).await;

        assert_eq!(relay.status(), RelayStatus::Connecting);

        relay.disconnect();

        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_disconnect_unresponsive_during_try_connect() {
        // Mock relay
        let opts = RelayTestOptions {
            unresponsive_connection: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        // Terminate after 3 secs
        let r = relay.clone();
        tokio::spawn(async move {
            time::sleep(Duration::from_secs(3)).await;
            r.disconnect();
        });

        let res = relay.try_connect(Duration::from_secs(7)).await;
        assert!(matches!(res.unwrap_err(), Error::TerminationRequest));

        assert_eq!(relay.status(), RelayStatus::Terminated);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_ban_relay() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.try_connect(Duration::from_secs(2)).await.unwrap();

        assert_eq!(relay.status(), RelayStatus::Connected);

        relay.ban();

        assert_eq!(relay.status(), RelayStatus::Banned);
        assert!(!relay.inner.is_running());

        // Retry to connect
        let res = relay.try_connect(Duration::from_secs(2)).await;
        assert!(matches!(res.unwrap_err(), Error::Banned));

        assert_eq!(relay.status(), RelayStatus::Banned);

        // Try to call disconnect. The status mustn't change.
        relay.disconnect();

        assert_eq!(relay.status(), RelayStatus::Banned);

        // Health check
        let res = relay.inner.ensure_operational();
        assert!(matches!(res.unwrap_err(), Error::Banned));
    }

    #[tokio::test]
    async fn test_wait_for_connection() {
        // Mock relay
        let opts = RelayTestOptions {
            unresponsive_connection: Some(Duration::from_secs(2)),
            ..Default::default()
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect();

        relay.wait_for_connection(Duration::from_millis(500)).await; // This timeout

        assert_eq!(relay.status(), RelayStatus::Connecting);

        relay.wait_for_connection(Duration::from_secs(3)).await;

        assert_eq!(relay.status(), RelayStatus::Connected);
    }

    #[tokio::test]
    async fn test_fetch_events_ban_relay() {
        // Mock relay
        let opts = RelayTestOptions {
            unresponsive_connection: None,
            send_random_events: true,
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default().ban_relay_on_mismatch(true));

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.try_connect(Duration::from_secs(3)).await.unwrap();

        assert_eq!(relay.status(), RelayStatus::Connected);

        let filter = Filter::new().kind(Kind::Metadata);
        relay
            .fetch_events(filter, Duration::from_secs(3), ReqExitPolicy::ExitOnEOSE)
            .await
            .unwrap();

        assert_eq!(relay.status(), RelayStatus::Banned);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_subscribe_ban_relay() {
        // Mock relay
        let opts = RelayTestOptions {
            unresponsive_connection: None,
            send_random_events: true,
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay = new_relay(url, RelayOptions::default().ban_relay_on_mismatch(true));

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.try_connect(Duration::from_secs(3)).await.unwrap();

        assert_eq!(relay.status(), RelayStatus::Connected);

        let filter = Filter::new().kind(Kind::Metadata).limit(3);
        relay
            .subscribe(filter, SubscribeOptions::default())
            .await
            .unwrap();

        // Keep up the test
        time::timeout(
            Some(Duration::from_secs(10)),
            relay.handle_notifications(|_| async { Ok(false) }),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(relay.status(), RelayStatus::Banned);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_nip42_send_event() {
        // Mock relay
        let opts = RelayBuilderNip42 {
            mode: RelayBuilderNip42Mode::Write,
        };
        let builder = RelayBuilder::default().nip42(opts);
        let mock = LocalRelay::run(builder).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        relay.inner.state.automatic_authentication(true);

        relay.connect();

        // Signer
        let keys = Keys::generate();

        // Send as unauthenticated (MUST return error)
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        let err = relay.send_event(&event).await.unwrap_err();
        if let Error::RelayMessage(msg) = err {
            assert_eq!(
                MachineReadablePrefix::parse(&msg).unwrap(),
                MachineReadablePrefix::AuthRequired
            );
        } else {
            panic!("Unexpected error");
        }

        // Set a signer
        relay.inner.state.set_signer(keys.clone()).await;

        // Send as authenticated
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        assert!(relay.send_event(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_nip42_fetch_events() {
        // Mock relay
        let opts = RelayBuilderNip42 {
            mode: RelayBuilderNip42Mode::Read,
        };
        let builder = RelayBuilder::default().nip42(opts);
        let mock = LocalRelay::run(builder).await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let relay: Relay = new_relay(url, RelayOptions::default());

        relay.connect();

        // Signer
        let keys = Keys::generate();

        // Send an event
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        relay.send_event(&event).await.unwrap();

        let filter = Filter::new().kind(Kind::TextNote).limit(3);

        // Disable NIP42 auto auth
        relay.inner.state.automatic_authentication(false);

        // Unauthenticated fetch (MUST return error)
        let err = relay
            .fetch_events(
                filter.clone(),
                Duration::from_secs(5),
                ReqExitPolicy::ExitOnEOSE,
            )
            .await
            .unwrap_err();
        match err {
            Error::RelayMessage(msg) => {
                assert_eq!(
                    MachineReadablePrefix::parse(&msg).unwrap(),
                    MachineReadablePrefix::AuthRequired
                );
            }
            e => panic!("Unexpected error: {e}"),
        }

        // Enable NIP42 auto auth
        relay.inner.state.automatic_authentication(true);

        // Unauthenticated fetch (MUST return error)
        let err = relay
            .fetch_events(
                filter.clone(),
                Duration::from_secs(5),
                ReqExitPolicy::ExitOnEOSE,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::AuthenticationFailed));

        // Set a signer
        relay.inner.state.set_signer(keys).await;

        // Authenticated fetch
        let res = relay
            .fetch_events(filter, Duration::from_secs(5), ReqExitPolicy::ExitOnEOSE)
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_events_exit_on_eose() {
        let (relay, _mock) = setup_event_fetching_relay(5).await;

        // Exit on EOSE
        let events = relay
            .fetch_events(
                Filter::new().kind(Kind::TextNote),
                Duration::from_secs(5),
                ReqExitPolicy::ExitOnEOSE,
            )
            .await
            .unwrap();
        assert_eq!(events.len(), 5);

        // Exit on EOSE
        let events = relay
            .fetch_events(
                Filter::new().kind(Kind::TextNote).limit(3),
                Duration::from_secs(5),
                ReqExitPolicy::ExitOnEOSE,
            )
            .await
            .unwrap();
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn test_fetch_events_wait_for_events() {
        let (relay, _mock) = setup_event_fetching_relay(5).await;

        let events = relay
            .fetch_events(
                Filter::new().kind(Kind::TextNote),
                Duration::from_secs(15),
                ReqExitPolicy::WaitForEvents(2),
            )
            .await
            .unwrap();
        assert_eq!(events.len(), 2); // Requested all text notes but exit after receive 2

        // Task to send additional event
        let r = relay.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Signer
            let keys = Keys::generate();

            // Build and send event
            let event = EventBuilder::metadata(&Metadata::new().name("Test"))
                .sign_with_keys(&keys)
                .unwrap();
            r.send_event(&event).await.unwrap();
        });

        let events = relay
            .fetch_events(
                Filter::new().kind(Kind::Metadata),
                Duration::from_secs(5),
                ReqExitPolicy::WaitForEvents(1),
            )
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_fetch_events_wait_for_events_after_eose() {
        let (relay, _mock) = setup_event_fetching_relay(10).await;

        // Task to send additional events
        let r = relay.clone();
        tokio::spawn(async move {
            // Signer
            let keys = Keys::generate();

            // Send more events
            for _ in 0..2 {
                // Sleep
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Build and send event
                let event = EventBuilder::text_note("Additional")
                    .sign_with_keys(&keys)
                    .unwrap();
                r.send_event(&event).await.unwrap();
            }
        });

        let events = relay
            .fetch_events(
                Filter::new().kind(Kind::TextNote).limit(3),
                Duration::from_secs(15),
                ReqExitPolicy::WaitForEventsAfterEOSE(2),
            )
            .await
            .unwrap();
        assert_eq!(events.len(), 5); // 3 events received until EOSE + 2 new events
    }

    #[tokio::test]
    async fn test_fetch_events_wait_for_duration_after_eose() {
        let (relay, _mock) = setup_event_fetching_relay(5).await;

        // Task to send additional events
        let r = relay.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Signer
            let keys = Keys::generate();

            // Send more events
            for _ in 0..2 {
                // Build and send event
                let event = EventBuilder::text_note("Additional")
                    .sign_with_keys(&keys)
                    .unwrap();
                r.send_event(&event).await.unwrap();

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        let events = relay
            .fetch_events(
                Filter::new().kind(Kind::TextNote),
                Duration::from_secs(15),
                ReqExitPolicy::WaitDurationAfterEOSE(Duration::from_secs(3)),
            )
            .await
            .unwrap();
        assert_eq!(events.len(), 6); // 5 events received until EOSE + 1 new events
    }

    #[tokio::test]
    async fn test_subscribe_ephemeral_event() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        // Sender
        let relay1: Relay = new_relay(url.clone(), RelayOptions::default());
        relay1.connect();
        relay1
            .try_connect(Duration::from_millis(500))
            .await
            .unwrap();

        // Fetcher
        let relay2 = new_relay(url, RelayOptions::default());
        relay2
            .try_connect(Duration::from_millis(500))
            .await
            .unwrap();

        // Signer
        let keys = Keys::generate();

        // Event
        let kind = Kind::Custom(22_222); // Ephemeral kind
        let event: Event = EventBuilder::new(kind, "").sign_with_keys(&keys).unwrap();

        let event_id: EventId = event.id;

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            relay1.send_event(&event).await.unwrap();
        });

        // Subscribe
        let filter = Filter::new().kind(kind);
        let sub_id = relay2
            .subscribe(filter, SubscribeOptions::default())
            .await
            .unwrap();

        // Listen for notifications
        let fut = relay2.handle_notifications(|notification| async {
            if let RelayNotification::Event {
                subscription_id,
                event,
            } = notification
            {
                if subscription_id == sub_id {
                    if event.id == event_id {
                        return Ok(true);
                    } else {
                        panic!("Unexpected event");
                    }
                } else {
                    panic!("Unexpected subscription ID");
                }
            }
            Ok(false)
        });

        tokio::time::timeout(Duration::from_secs(5), fut)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let (id, relay, _mock) = setup_subscription_relay().await;

        time::sleep(Duration::from_secs(1)).await;

        assert!(relay.subscription(&id).await.is_some());

        relay.unsubscribe(&id).await.unwrap();

        assert!(relay.subscription(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_unsubscribe_all() {
        let (_id, relay, _mock) = setup_subscription_relay().await;

        time::sleep(Duration::from_secs(1)).await;

        relay.unsubscribe_all().await.unwrap();

        relay.subscriptions().await.is_empty();
    }

    #[tokio::test]
    async fn test_admit_connection() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        let mut relay = new_relay(url.clone(), RelayOptions::default());

        relay.inner.state.admit_policy = Some(Arc::new(CustomTestPolicy {
            banned_relays: HashSet::from([url]),
        }));

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay.connect();

        time::sleep(Duration::from_secs(2)).await;

        assert_eq!(relay.status(), RelayStatus::Terminated);
        assert!(!relay.inner.is_running());

        // Retry to connect
        let res = relay.try_connect(Duration::from_secs(2)).await;
        assert!(matches!(res.unwrap_err(), Error::ConnectionRejected { .. }));

        assert_eq!(relay.status(), RelayStatus::Terminated);
        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_negentropy_sync() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        // Database
        let database = MemoryDatabase::with_opts(MemoryDatabaseOptions {
            events: true,
            max_events: None,
        });

        // Build events to store in the local database
        let local_events = vec![
            EventBuilder::text_note("Local 1")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
            EventBuilder::text_note("Local 2")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
            EventBuilder::new(Kind::Custom(123), "Local 123")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
        ];

        // Save an event to the local database
        for event in local_events.iter() {
            database.save_event(event).await.unwrap();
        }
        assert_eq!(database.count(Filter::new()).await.unwrap(), 3);

        // Relay
        let relay =
            new_relay_with_database(url, Arc::new(database.clone()), RelayOptions::default());

        // Connect
        relay.try_connect(Duration::from_secs(2)).await.unwrap();

        // Build events to send to the relay
        let relays_events = vec![
            // Event in common with the local database
            local_events[0].clone(),
            EventBuilder::text_note("Test 2")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
            EventBuilder::text_note("Test 3")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
            EventBuilder::new(Kind::Custom(123), "Test 4")
                .sign_with_keys(&Keys::generate())
                .unwrap(),
        ];

        // Send events to the relays
        for event in relays_events.iter() {
            relay.send_event(event).await.unwrap();
        }

        // Sync
        let filter = Filter::new().kind(Kind::TextNote);
        let opts = SyncOptions::default().direction(SyncDirection::Both);
        let output = relay.sync(filter, &opts).await.unwrap();

        assert_eq!(
            output,
            Reconciliation {
                local: HashSet::from([local_events[1].id]),
                remote: HashSet::from([relays_events[1].id, relays_events[2].id]),
                sent: HashSet::from([local_events[1].id]),
                received: HashSet::from([relays_events[1].id, relays_events[2].id]),
                send_failures: HashMap::new(),
            }
        );
    }

    #[tokio::test]
    async fn test_sleep_when_idle() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        // Relay
        let opts = RelayOptions::default()
            .sleep_when_idle(true)
            .idle_timeout(Duration::from_secs(2));
        let relay = new_relay(url, opts);

        // Connect
        relay.try_connect(Duration::from_secs(1)).await.unwrap();

        // Check that is connected
        assert_eq!(relay.status(), RelayStatus::Connected);

        // Wait to make sure the relay go in sleep mode (see SLEEP_INTERVAL const)
        time::sleep(Duration::from_secs(3)).await;
        check_relay_is_sleeping(&relay);

        // Test wake up when sending an event
        let event = EventBuilder::text_note("text wake-up")
            .sign_with_keys(&Keys::generate())
            .unwrap();
        relay.send_event(&event).await.unwrap();
        assert_eq!(relay.status(), RelayStatus::Connected);

        // Check if relay is sleeping
        time::sleep(Duration::from_secs(3)).await;
        check_relay_is_sleeping(&relay);

        // Test wake up when fetch events
        let filter = Filter::new().kind(Kind::TextNote);
        let _ = relay
            .fetch_events(filter, Duration::from_secs(10), ReqExitPolicy::ExitOnEOSE)
            .await
            .unwrap();
        assert_eq!(relay.status(), RelayStatus::Connected);

        // Check if relay is sleeping
        time::sleep(Duration::from_secs(3)).await;
        check_relay_is_sleeping(&relay);

        // Test wake up when sync
        let filter = Filter::new().kind(Kind::TextNote);
        let _ = relay.sync(filter, &SyncOptions::new()).await.unwrap();
        assert_eq!(relay.status(), RelayStatus::Connected);

        // Check if relay is sleeping
        time::sleep(Duration::from_secs(3)).await;
        check_relay_is_sleeping(&relay);
    }

    #[tokio::test]
    async fn test_sleep_when_idle_with_long_lived_subscription() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = RelayUrl::parse(&mock.url()).unwrap();

        // Relay
        let opts = RelayOptions::default()
            .sleep_when_idle(true)
            .idle_timeout(Duration::from_secs(2));
        let relay = new_relay(url, opts);

        // Connect
        relay.try_connect(Duration::from_secs(1)).await.unwrap();

        // Check that is connected
        assert_eq!(relay.status(), RelayStatus::Connected);

        let filter = Filter::new().kind(Kind::TextNote);
        relay
            .subscribe(filter, SubscribeOptions::default())
            .await
            .unwrap();

        time::sleep(Duration::from_secs(5)).await;
        assert_eq!(relay.status(), RelayStatus::Connected);
    }
}
