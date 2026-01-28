use std::borrow::Cow;
use std::cmp;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::{task, time};
use async_wsocket::{ConnectionMode, Message};
use futures::{self, SinkExt, StreamExt};
use nostr::rand::rngs::OsRng;
use nostr::rand::{Rng, RngCore, TryRngCore};
use nostr_database::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot, Mutex, MutexGuard, Notify, RwLock, RwLockWriteGuard};

use super::capabilities::{AtomicRelayCapabilities, RelayCapabilities};
use super::constants::{
    DEFAULT_CONNECTION_TIMEOUT, JITTER_RANGE, MAX_RETRY_INTERVAL, MIN_ATTEMPTS, MIN_SUCCESS_RATE,
    PING_INTERVAL, SLEEP_INTERVAL, WEBSOCKET_TX_TIMEOUT,
};
use super::options::{RelayOptions, ReqExitPolicy, SubscribeAutoCloseOptions};
use super::ping::PingTracker;
use super::stats::RelayConnectionStats;
use super::{
    Error, RelayNotification, RelayStatus, SubscriptionActivity, SubscriptionAutoClosedReason,
};
use crate::client::ClientNotification;
use crate::policy::AdmitStatus;
use crate::relay::status::AtomicRelayStatus;
use crate::shared::SharedState;
use crate::transport::websocket::{WebSocketSink, WebSocketStream};

type ClientMessageJson = String;

// Skip NIP-50 matches since they may create issues and ban non-malicious relays.
const MATCH_EVENT_OPTS: MatchEventOptions = MatchEventOptions::new().nip50(false);

enum IngesterCommand {
    Authenticate { challenge: String },
}

enum HandleClosedMsg {
    MarkAsClosed,
    Remove,
}

struct HandleAutoClosing {
    to_close: bool,
    reason: Option<SubscriptionAutoClosedReason>,
}

struct JsonMessageItem {
    json: ClientMessageJson,
    confirmation: Option<oneshot::Sender<()>>,
}

#[derive(Debug)]
struct RelayChannels {
    nostr: (Sender<JsonMessageItem>, Mutex<Receiver<JsonMessageItem>>),
    ping: Notify,
    terminate: Notify,
}

impl RelayChannels {
    pub fn new() -> Self {
        let (tx_nostr, rx_nostr) = mpsc::channel(1024);

        Self {
            nostr: (tx_nostr, Mutex::new(rx_nostr)),
            ping: Notify::new(),
            terminate: Notify::new(),
        }
    }

    #[inline]
    fn send_client_msg(&self, msg: JsonMessageItem) -> Result<(), Error> {
        self.nostr
            .0
            .try_send(msg)
            .map_err(|_| Error::CantSendMessageToDispatcher)
    }

    #[inline]
    pub async fn rx_nostr(&self) -> MutexGuard<'_, Receiver<JsonMessageItem>> {
        self.nostr.1.lock().await
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn ping(&self) {
        self.ping.notify_one()
    }

    pub fn terminate(&self) {
        self.terminate.notify_one()
    }
}

#[derive(Debug)]
struct SubscriptionData {
    pub filters: Vec<Filter>,
    pub subscribed_at: Timestamp,
    pub is_auto_closing: bool,
    /// Received EOSE msg
    pub received_eose: bool,
    /// Number of received events
    pub received_events: AtomicUsize,
    /// Subscription closed by relay
    pub closed: bool,
}

impl Default for SubscriptionData {
    fn default() -> Self {
        Self {
            filters: Vec::new(),
            subscribed_at: Timestamp::zero(),
            is_auto_closing: false,
            received_eose: false,
            received_events: AtomicUsize::new(0),
            closed: false,
        }
    }
}

// Instead of wrap every field in an `Arc<T>`, which increases the number of atomic operations,
// put all fields that require an `Arc` here.
#[derive(Debug)]
pub(super) struct AtomicPrivateData {
    status: AtomicRelayStatus,
    channels: RelayChannels,
    subscriptions: RwLock<HashMap<SubscriptionId, SubscriptionData>>,
    running: AtomicBool,
}

#[derive(Debug, Clone)]
pub(crate) struct InnerRelay {
    pub(super) url: RelayUrl,
    pub(super) atomic: Arc<AtomicPrivateData>,
    pub(super) opts: RelayOptions,
    pub(super) capabilities: Arc<AtomicRelayCapabilities>,
    pub(super) stats: RelayConnectionStats,
    pub(super) state: SharedState,
    pub(super) internal_notification_sender: broadcast::Sender<RelayNotification>,
    external_notification_sender: Option<broadcast::Sender<ClientNotification>>,
}

impl InnerRelay {
    pub(super) fn new(
        url: RelayUrl,
        state: SharedState,
        capabilities: RelayCapabilities,
        opts: RelayOptions,
    ) -> Self {
        let (relay_notification_sender, ..) =
            broadcast::channel::<RelayNotification>(opts.notification_channel_size);

        Self {
            url,
            atomic: Arc::new(AtomicPrivateData {
                status: AtomicRelayStatus::default(),
                channels: RelayChannels::new(),
                subscriptions: RwLock::new(HashMap::new()),
                running: AtomicBool::new(false),
            }),
            capabilities: Arc::new(AtomicRelayCapabilities::new(capabilities)),
            opts,
            stats: RelayConnectionStats::default(),
            state,
            internal_notification_sender: relay_notification_sender,
            external_notification_sender: None,
        }
    }

    #[inline]
    pub fn connection_mode(&self) -> &ConnectionMode {
        &self.opts.connection_mode
    }

    /// Check if the connection task is running
    #[inline]
    pub(super) fn is_running(&self) -> bool {
        self.atomic.running.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn status(&self) -> RelayStatus {
        self.atomic.status.load()
    }

    pub(super) fn set_status(&self, status: RelayStatus, log: bool) {
        // Change status
        self.atomic.status.set(status);

        // Log
        if log {
            match status {
                RelayStatus::Initialized => tracing::trace!(url = %self.url, "Relay initialized."),
                RelayStatus::Pending => tracing::trace!(url = %self.url, "Relay is pending."),
                RelayStatus::Connecting => tracing::debug!("Connecting to '{}'", self.url),
                RelayStatus::Connected => tracing::info!("Connected to '{}'", self.url),
                RelayStatus::Disconnected => tracing::info!("Disconnected from '{}'", self.url),
                RelayStatus::Terminated => {
                    tracing::info!("Completely disconnected from '{}'", self.url)
                }
                RelayStatus::Banned => tracing::info!(url = %self.url, "Relay banned."),
                RelayStatus::Sleeping => tracing::info!("Relay '{}' went to sleep.", self.url),
                RelayStatus::Shutdown => tracing::info!("Relay '{}' has been shutdown.", self.url),
            }
        }

        // Send notification
        self.send_notification(RelayNotification::RelayStatus { status }, false);

        // If monitor is enabled, notify status change.
        if let Some(monitor) = &self.state.monitor {
            monitor.notify_status_change(self.url.clone(), status);
        }
    }

    /// Perform checks to ensure that the relay is ready for use.
    pub(super) fn ensure_operational(&self) -> Result<(), Error> {
        // Ensures that the relay is awake.
        self.ensure_awake_for_activity();

        // Get current status
        let status: RelayStatus = self.status();

        // Relay is not ready (never called connect method)
        if status.is_initialized() {
            return Err(Error::NotReady);
        }

        // The relay has been banned
        if status.is_banned() {
            return Err(Error::Banned);
        }

        // Sanity-check, to ensure that the relay is not sleeping.
        if status.is_sleeping() {
            return Err(Error::Sleeping);
        }

        // This is needed to allow giving the time to the relay to connect,
        // instead of just checking the status.
        //
        // A relay is considered not connected if all the following conditions are met:
        // - the status is different from `RelayStatus::Connected`
        // - the relay has already exceeded the minimum number of attempts
        // - the connection success rate is lower than the minimum success rate
        // - the relay woke up from sleep from more than `DEFAULT_CONNECTION_TIMEOUT` (needed if the relay has just waked up!)
        if !status.is_connected()
            && self.stats.attempts() > MIN_ATTEMPTS
            && self.stats.success_rate() < MIN_SUCCESS_RATE
            && self.stats.woke_up_at() + DEFAULT_CONNECTION_TIMEOUT < Timestamp::now()
        {
            return Err(Error::NotConnected);
        }

        // Check avg. latency
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Check if max avg latency is set
            if let Some(max) = self.opts.max_avg_latency {
                // ONLY LATER get the latency, to avoid unnecessary calculation
                if let Some(current) = self.stats.latency() {
                    if current > max {
                        return Err(Error::MaximumLatencyExceeded { max, current });
                    }
                }
            }
        }

        Ok(())
    }

    /// Returns all long-lived (non-auto-closing) subscriptions
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        let subscription = self.atomic.subscriptions.read().await;
        subscription
            .iter()
            .filter_map(|(k, v)| (!v.is_auto_closing).then_some((k.clone(), v.filters.clone())))
            .collect()
    }

    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        let subscription = self.atomic.subscriptions.read().await;
        subscription.get(id).map(|d| d.filters.clone())
    }

    pub(super) async fn remove_subscription(&self, id: &SubscriptionId) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        subscriptions.remove(id);
    }

    /// Register an auto-closing subscription
    pub(crate) async fn add_auto_closing_subscription(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
    ) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        let data: &mut SubscriptionData = subscriptions.entry(id).or_default();
        data.filters = filters;
        data.is_auto_closing = true;
    }

    pub(crate) async fn update_subscription(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        update_subscribed_at: bool,
    ) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        let data: &mut SubscriptionData = subscriptions.entry(id).or_default();
        data.filters = filters;

        if update_subscribed_at {
            data.subscribed_at = Timestamp::now();
        }
    }

    /// Mark subscription as closed
    async fn subscription_closed(&self, id: &SubscriptionId) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        if let Some(data) = subscriptions.get_mut(id) {
            data.closed = true;
        }
    }

    /// Received eose for subscription
    async fn received_eose(&self, id: &SubscriptionId) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        if let Some(data) = subscriptions.get_mut(id) {
            data.received_eose = true;
        }
    }

    /// Check if it should subscribe for current websocket session
    pub(crate) async fn should_resubscribe(&self, id: &SubscriptionId) -> bool {
        let subscriptions = self.atomic.subscriptions.read().await;
        match subscriptions.get(id) {
            Some(SubscriptionData {
                subscribed_at,
                closed,
                is_auto_closing: false,
                ..
            }) => {
                // Never subscribed -> SHOULD subscribe
                // Subscription closed by relay -> SHOULD subscribe
                if subscribed_at.is_zero() || *closed {
                    return true;
                }

                // First connection and subscribed_at != 0 -> SHOULD NOT re-subscribe
                // Many connections and subscription NOT done in current websocket session -> SHOULD re-subscribe
                self.stats.connected_at() > *subscribed_at && self.stats.success() > 1
            }
            // NOT subscribe if auto-closing subscription or subscription not found
            Some(SubscriptionData {
                is_auto_closing: true,
                ..
            })
            | None => false,
        }
    }

    #[inline]
    pub(crate) fn set_notification_sender(
        &mut self,
        notification_sender: broadcast::Sender<ClientNotification>,
    ) {
        self.external_notification_sender = Some(notification_sender);
    }

    fn send_notification(&self, notification: RelayNotification, external: bool) {
        match (external, &self.external_notification_sender) {
            (true, Some(external_notification_sender)) => {
                // Clone and send internal notification
                let _ = self.internal_notification_sender.send(notification.clone());

                // Convert relay to notification to pool notification
                let notification: Option<ClientNotification> = match notification {
                    RelayNotification::Event {
                        subscription_id,
                        event,
                    } => Some(ClientNotification::Event {
                        relay_url: self.url.clone(),
                        subscription_id,
                        event,
                    }),
                    RelayNotification::Message { message } => Some(ClientNotification::Message {
                        relay_url: self.url.clone(),
                        message,
                    }),
                    RelayNotification::RelayStatus { .. } => None,
                    RelayNotification::Authenticated => None,
                    RelayNotification::AuthenticationFailed => None,
                };

                // Send external notification
                if let Some(notification) = notification {
                    let _ = external_notification_sender.send(notification);
                }
            }
            _ => {
                // Send internal notification
                let _ = self.internal_notification_sender.send(notification);
            }
        }
    }

    pub(super) async fn check_connection_policy(&self) -> Result<AdmitStatus, Error> {
        match &self.state.admit_policy {
            Some(policy) => Ok(policy.admit_connection(&self.url).await?),
            None => Ok(AdmitStatus::Success),
        }
    }

    /// Check if relay should sleep
    async fn should_sleep(&self) -> bool {
        // Check if sleeping is disabled
        if !self.opts.sleep_when_idle {
            return false;
        }

        // Get current subscriptions
        let subscriptions = self.atomic.subscriptions.read().await;

        // No sleep if there are active subscriptions
        if !subscriptions.is_empty() {
            return false;
        }

        // See if enough time elapsed since the last activity
        let last_activity: Timestamp = self.stats.last_activity_at();

        // If no activity has been recorded yet, use connection time
        let reference_time: Timestamp = if last_activity == Timestamp::zero() {
            self.stats.connected_at()
        } else {
            last_activity
        };

        // If reference time is still 0, do not sleep; relay has just started.
        if reference_time == Timestamp::zero() {
            return false;
        }

        let idle_duration_secs: u64 = Timestamp::now().as_secs() - reference_time.as_secs();
        let idle_duration: Duration = Duration::from_secs(idle_duration_secs);
        idle_duration >= self.opts.idle_timeout
    }

    /// Wake up the relay if it's sleeping and update the last activity timestamp.
    #[inline]
    fn ensure_awake_for_activity(&self) {
        // If the sleeping is disabled, immediately return.
        if !self.opts.sleep_when_idle {
            return;
        }

        // Update last activity timestamp
        self.stats.update_activity();

        // If it isn't sleeping, immediately return.
        if !self.status().is_sleeping() {
            return;
        }

        tracing::debug!(url = %self.url, "Waking up sleeping relay.");

        // Update status to pending
        // TODO: is this needed here?
        self.set_status(RelayStatus::Pending, false);

        // Spawn a new connection task
        self.spawn_connection_task(None);

        // Update the wake-up timestamp
        self.stats.just_woke_up();
    }

    pub(super) fn spawn_connection_task(&self, stream: Option<(WebSocketSink, WebSocketStream)>) {
        // Check if the connection task is already running
        // This is checked also later, but it's checked also here to avoid a full-clone if we know that is already running.
        if self.is_running() {
            tracing::warn!(url = %self.url, "Connection task is already running.");
            return;
        }

        // Full-clone
        let relay: InnerRelay = self.clone();

        // Spawn task
        task::spawn(relay.connection_task(stream));
    }

    /// This **MUST** be called only by the [`InnerRelay::spawn_connection_task`] method!
    async fn connection_task(self, mut stream: Option<(WebSocketSink, WebSocketStream)>) {
        // Set the connection task as running and get the previous value.
        let is_running: bool = self.atomic.running.swap(true, Ordering::SeqCst);

        // Re-check if the connection task is already running.
        // This is required because may happen that two tasks are spawned at the exact same moment.
        // Not use the "assert" macro since will cause the task to panic.
        if is_running {
            tracing::warn!(url = %self.url, "Connection task is already running.");
            return;
        }

        // Lock receiver
        let mut rx_nostr = self.atomic.channels.rx_nostr().await;

        // Last websocket error
        // Store it to avoid printing every time the same connection error
        let mut last_ws_error = None;

        // Auto-connect loop
        loop {
            // Check if the connection is allowed
            match self.check_connection_policy().await {
                Ok(status) => {
                    // Connection rejected, update status and break the loop.
                    if let AdmitStatus::Rejected { reason } = status {
                        if let Some(reason) = reason {
                            tracing::warn!(reason = %reason, "Connection rejected by admission policy.");
                        }

                        // Set the status to "terminated" and break loop.
                        self.set_status(RelayStatus::Terminated, false);
                        break;
                    }
                }
                Err(e) => tracing::error!(error = %e, "Impossible to check connection policy."),
            }

            // Connect and run message handler
            // The termination requests are handled inside this method!
            self.connect_and_run(stream.take(), &mut rx_nostr, &mut last_ws_error)
                .await;

            // Get status
            let status: RelayStatus = self.status();

            // If the relay is terminated, banned or sleeping, break the loop.
            if status.is_terminated()
                || status.is_banned()
                || status.is_sleeping()
                || status.is_shutdown()
            {
                break;
            }

            // Check if reconnection is enabled
            if self.opts.reconnect {
                // Check if the relay is marked as disconnected. If not, update status.
                // Check if disconnected to avoid a possible double log
                if !status.is_disconnected() {
                    self.set_status(RelayStatus::Disconnected, true);
                }

                // Sleep before retry to connect
                let interval: Duration = self.calculate_retry_interval();
                tracing::debug!(
                    "Reconnecting to '{}' relay in {} secs",
                    self.url,
                    interval.as_secs()
                );

                // Sleep before retry to connect
                // Handle termination to allow exiting immediately if request is received during the sleep.
                tokio::select! {
                    // Sleep
                    _ = time::sleep(interval) => {},
                    // Handle termination notification
                    _ = self.handle_terminate() => break,
                }
            } else {
                // Reconnection disabled, set status to "terminated"
                self.set_status(RelayStatus::Terminated, true);

                // Break loop and exit
                tracing::debug!(url = %self.url, "Reconnection disabled, breaking loop.");
                break;
            }
        }

        // Mark the connection task as stopped.
        self.atomic.running.store(false, Ordering::SeqCst);

        tracing::debug!(url = %self.url, "Auto connect loop terminated.");
    }

    /// Depending on attempts and success, use default or incremental retry interval
    fn calculate_retry_interval(&self) -> Duration {
        // Check if the incremental interval is enabled
        if self.opts.adjust_retry_interval {
            // Calculate the difference between attempts and success
            let diff: u32 = self.stats.attempts().saturating_sub(self.stats.success()) as u32;

            // Calculate multiplier
            let multiplier: u32 = 1 + (diff / 2);

            // Compute the adaptive retry interval
            let adaptive_interval: Duration = self.opts.retry_interval * multiplier;

            // If the interval is too big, use the min one.
            // If the interval is checked after the jitter, the interval may be the same for all relays!
            let mut interval: Duration = cmp::min(adaptive_interval, MAX_RETRY_INTERVAL);

            // The jitter is added to avoid situations where multiple relays reconnect simultaneously after a failure.
            // This helps prevent synchronized retry storms.
            let jitter: i8 = OsRng.unwrap_err().random_range(JITTER_RANGE);

            // Apply jitter
            if jitter >= 0 {
                // Positive jitter, add it to the interval.
                interval = interval.saturating_add(Duration::from_secs(jitter as u64));
            } else {
                // Negative jitter, compute `|jitter|` and saturating subtract it from the interval.
                let jitter: u64 = jitter.unsigned_abs() as u64;
                interval = interval.saturating_sub(Duration::from_secs(jitter));
            }

            // Return interval
            return interval;
        }

        // Use default internal
        self.opts.retry_interval
    }

    #[inline]
    async fn handle_terminate(&self) {
        // Wait to be notified
        self.atomic.channels.terminate.notified().await;
    }

    pub(super) async fn _try_connect(
        &self,
        timeout: Duration,
        status_on_failure: RelayStatus,
    ) -> Result<(WebSocketSink, WebSocketStream), Error> {
        // Update status
        self.set_status(RelayStatus::Connecting, true);

        // Increase the attempts
        self.stats.new_attempt();

        // Try to connect
        // If during connection the termination request is received, abort the connection and return error.
        // At this stem is NOT required to close the WebSocket connection.
        tokio::select! {
            // Connect
            res = self.state.transport.connect((&self.url).into(), &self.opts.connection_mode, timeout) => match res {
                Ok((ws_tx, ws_rx)) => {
                    // Update status
                    self.set_status(RelayStatus::Connected, true);

                    // Increment success stats
                    self.stats.new_success();

                    Ok((ws_tx, ws_rx))
                }
                Err(e) => {
                    // Update status
                    self.set_status(status_on_failure, false);

                    // Return error
                    Err(Error::Transport(e))
                }
            },
            // Handle termination notification
            _ = self.handle_terminate() => Err(Error::TerminationRequest),
        }
    }

    /// Connect and run message handler
    ///
    /// If `stream` arg is passed, no connection attempt will be done.
    async fn connect_and_run(
        &self,
        stream: Option<(WebSocketSink, WebSocketStream)>,
        rx_nostr: &mut MutexGuard<'_, Receiver<JsonMessageItem>>,
        last_ws_error: &mut Option<String>,
    ) {
        match stream {
            // Already have a stream, go to post-connection stage
            Some((ws_tx, ws_rx)) => self.post_connection(ws_tx, ws_rx, rx_nostr).await,
            // No stream is passed, try to connect
            // Set the status to "disconnected" to allow to automatic retries
            None => match self
                ._try_connect(DEFAULT_CONNECTION_TIMEOUT, RelayStatus::Disconnected)
                .await
            {
                // Connection success, go to post-connection stage
                Ok((ws_tx, ws_rx)) => self.post_connection(ws_tx, ws_rx, rx_nostr).await,
                // Error during connection
                Err(e) => {
                    // TODO: avoid string allocation. The error is converted to string only to perform the `!=` binary operation.
                    // Check if error should be logged
                    let e: String = e.to_string();
                    let to_log: bool = match &last_ws_error {
                        Some(prev_err) => {
                            // Log only if different from the last one
                            prev_err != &e
                        }
                        None => true,
                    };

                    // Log error and update the last error
                    if to_log {
                        tracing::error!(url = %self.url, error= %e, "Connection failed.");
                        *last_ws_error = Some(e);
                    }
                }
            },
        }
    }

    /// To run after websocket connection.
    /// Run message handlers, pinger and other services
    async fn post_connection(
        &self,
        mut ws_tx: WebSocketSink,
        ws_rx: WebSocketStream,
        rx_nostr: &mut MutexGuard<'_, Receiver<JsonMessageItem>>,
    ) {
        // (Re)subscribe to relay
        if self.capabilities.can_read() {
            if let Err(e) = self.resubscribe().await {
                tracing::error!(url = %self.url, error = %e, "Impossible to subscribe.")
            }
        }

        let ping: PingTracker = PingTracker::default();

        let (ingester_tx, ingester_rx) = mpsc::unbounded_channel();

        // Wait that one of the futures terminates/completes
        // Add also termination here, to allow closing the connection in case of termination request.
        tokio::select! {
            // Message sender handler
            res = self.sender_message_handler(&mut ws_tx, rx_nostr, &ping) => match res {
                Ok(()) => tracing::trace!(url = %self.url, "Relay sender exited."),
                Err(e) => tracing::error!(url = %self.url, error = %e, "Relay sender exited with error.")
            },
            // Message receiver handler
            res = self.receiver_message_handler(ws_rx, &ping, ingester_tx) => match res {
                Ok(()) => tracing::trace!(url = %self.url, "Relay receiver exited."),
                Err(e) => tracing::error!(url = %self.url, error = %e, "Relay receiver exited with error.")
            },
            // Ingester: perform actions
            res = self.ingester(ingester_rx) => match res {
                Ok(()) => tracing::trace!(url = %self.url, "Relay ingester exited."),
                Err(e) => tracing::error!(url = %self.url, error = %e, "Relay ingester exited with error.")
            },
            // Monitor when the relay can go to sleep
            _ = self.sleep_when_idle_monitor() => {},
            // Termination handler
            _ = self.handle_terminate() => {},
            // Pinger
            _ = self.pinger() => {}
        }

        // Always try to close the WebSocket connection
        match close_ws(&mut ws_tx).await {
            Ok(..) => tracing::debug!("WebSocket connection closed."),
            Err(e) => tracing::error!(error = %e, "Can't close WebSocket connection."),
        }
    }

    async fn sender_message_handler(
        &self,
        ws_tx: &mut WebSocketSink,
        rx_nostr: &mut MutexGuard<'_, Receiver<JsonMessageItem>>,
        ping: &PingTracker,
    ) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        let _ping = ping;

        loop {
            tokio::select! {
                // Nostr channel receiver
                Some(JsonMessageItem { json, confirmation }) = rx_nostr.recv() => {
                    // Get messages size
                    let size: usize = json.len();

                    // Log
                    tracing::debug!("Sending '{json}' to '{}' (size: {size} bytes)", self.url);

                    // Compose WebSocket text messages
                    let msg: Message = Message::Text(json);

                    // Send WebSocket messages
                    send_ws_msg(ws_tx, msg).await?;

                    // Send the confirmation
                    if let Some(confirmation) = confirmation {
                        if confirmation.send(()).is_err() {
                            tracing::error!(url = %self.url, "Can't send msg confirmation.");
                        }
                    }

                    // Increase sent bytes
                    self.stats.add_bytes_sent(size);
                }
                // Ping channel receiver
                _ = self.atomic.channels.ping.notified() => {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // If the last nonce is NOT 0, check if relay replied.
                        // Return error if relay not replied
                        if ping.last_nonce() != 0 && !ping.replied() {
                            return Err(Error::NotRepliedToPing);
                        }

                        // Generate and save nonce
                        let mut rng = OsRng.unwrap_err();
                        let nonce: u64 = rng.next_u64();
                        ping.set_last_nonce(nonce);
                        ping.set_replied(false);

                        // Compose ping message
                        let msg = Message::Ping(nonce.to_be_bytes().to_vec());

                        // Send WebSocket message
                        send_ws_msg(ws_tx, msg).await?;

                        // Set ping as just sent
                        ping.just_sent().await;

                        #[cfg(debug_assertions)]
                        tracing::debug!(url = %self.url, nonce = %nonce, "Ping sent.");
                    }
                }
                else => break
            }
        }

        Ok(())
    }

    async fn receiver_message_handler(
        &self,
        mut ws_rx: WebSocketStream,
        ping: &PingTracker,
        ingester_tx: mpsc::UnboundedSender<IngesterCommand>,
    ) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        let _ping = ping;

        while let Some(msg) = ws_rx.next().await {
            match msg? {
                Message::Text(json) => self.handle_relay_message(&json, &ingester_tx).await,
                Message::Binary(_) => {
                    tracing::warn!(url = %self.url, "Binary messages aren't supported.");
                }
                #[cfg(not(target_arch = "wasm32"))]
                Message::Pong(bytes) => {
                    if self.opts.ping && self.state.transport.support_ping() {
                        match bytes.try_into() {
                            Ok(nonce) => {
                                // Nonce from big-endian bytes
                                let nonce: u64 = u64::from_be_bytes(nonce);

                                // Get last nonce
                                let last_nonce: u64 = ping.last_nonce();

                                // Check if last nonce not matches the received one
                                if last_nonce != nonce {
                                    return Err(Error::PongNotMatch {
                                        expected: last_nonce,
                                        received: nonce,
                                    });
                                }

                                // Set ping as replied
                                ping.set_replied(true);

                                // Save latency
                                let sent_at = ping.sent_at().await;
                                self.stats.save_latency(sent_at.elapsed());
                            }
                            Err(..) => {
                                return Err(Error::CantParsePong);
                            }
                        }
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                Message::Close(None) => break,
                #[cfg(not(target_arch = "wasm32"))]
                Message::Close(Some(frame)) => {
                    tracing::info!(code = %frame.code, reason = %frame.reason, "Connection closed by peer.");
                    break;
                }
                #[cfg(not(target_arch = "wasm32"))]
                _ => {}
            }
        }

        Ok(())
    }

    async fn ingester(
        &self,
        mut rx: mpsc::UnboundedReceiver<IngesterCommand>,
    ) -> Result<(), Error> {
        while let Some(command) = rx.recv().await {
            match command {
                // Authenticate to relay
                IngesterCommand::Authenticate { challenge } => {
                    match self.auth(challenge).await {
                        Ok(..) => {
                            self.send_notification(RelayNotification::Authenticated, false);

                            tracing::info!(url = %self.url, "Authenticated to relay.");

                            // TODO: ?
                            if let Err(e) = self.resubscribe().await {
                                tracing::error!(
                                    url = %self.url,
                                    error = %e,
                                    "Impossible to resubscribe."
                                );
                            }
                        }
                        Err(e) => {
                            self.send_notification(RelayNotification::AuthenticationFailed, false);

                            tracing::error!(
                                url = %self.url,
                                error = %e,
                                "Can't authenticate to relay."
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Monitor if it's time to put the relay in sleep mode.
    async fn sleep_when_idle_monitor(&self) {
        loop {
            // Sleep
            time::sleep(SLEEP_INTERVAL).await;

            // Check if should go to sleep
            if self.should_sleep().await {
                // Update status
                self.set_status(RelayStatus::Sleeping, true);

                // Break the loop
                break;
            }
        }
    }

    /// Send a signal every [`PING_INTERVAL`] to the other tasks, asking to ping the relay.
    async fn pinger(&self) {
        loop {
            // Check if support ping
            #[cfg(not(target_arch = "wasm32"))]
            if self.opts.ping && self.state.transport.support_ping() {
                // Ping supported, ping!
                self.atomic.channels.ping();
            }

            // Sleep
            time::sleep(PING_INTERVAL).await;
        }
    }

    async fn handle_relay_message(
        &self,
        msg: &str,
        ingester_tx: &mpsc::UnboundedSender<IngesterCommand>,
    ) {
        match self.handle_raw_relay_message(msg).await {
            Ok(Some(message)) => {
                match &message {
                    RelayMessage::Closed {
                        subscription_id,
                        message,
                    } => {
                        // Check machine-readable prefix
                        let res: HandleClosedMsg = match MachineReadablePrefix::parse(message) {
                            Some(MachineReadablePrefix::Duplicate) => HandleClosedMsg::Remove,
                            Some(MachineReadablePrefix::Pow) => HandleClosedMsg::Remove,
                            Some(MachineReadablePrefix::Blocked) => HandleClosedMsg::Remove,
                            Some(MachineReadablePrefix::RateLimited) => {
                                // TODO: add something like MarkAsRateLimited?
                                // TODO: And retry after some time to re-subscribe
                                HandleClosedMsg::MarkAsClosed
                            }
                            Some(MachineReadablePrefix::Invalid) => HandleClosedMsg::Remove,
                            Some(MachineReadablePrefix::Error) => HandleClosedMsg::Remove,
                            Some(MachineReadablePrefix::Unsupported) => HandleClosedMsg::Remove,
                            Some(MachineReadablePrefix::AuthRequired) => {
                                // Authentication is handled in other parts of code,
                                // so here just mark as closed.
                                HandleClosedMsg::MarkAsClosed
                            }
                            Some(MachineReadablePrefix::Restricted) => HandleClosedMsg::Remove,
                            None => {
                                // Doesn't mach any prefix,
                                // meaning that it probably closed without errors,
                                // so remove it.
                                HandleClosedMsg::Remove
                            }
                        };

                        // TODO: if auto-closing subscription, just remove it.

                        match res {
                            HandleClosedMsg::MarkAsClosed => {
                                self.subscription_closed(subscription_id).await;
                            }
                            HandleClosedMsg::Remove => {
                                tracing::debug!(
                                    url = %self.url,
                                    id = %subscription_id,
                                    "Removing subscription."
                                );

                                self.remove_subscription(subscription_id).await;
                            }
                        }
                    }
                    RelayMessage::EndOfStoredEvents(id) => {
                        self.received_eose(id).await;
                    }
                    RelayMessage::Auth { challenge } => {
                        // Check if NIP42 auto authentication is enabled
                        if self.state.is_auto_authentication_enabled() {
                            // Forward action to ingester
                            let _ = ingester_tx.send(IngesterCommand::Authenticate {
                                challenge: challenge.to_string(),
                            });
                        }
                    }
                    _ => (),
                }

                // Send notification
                self.send_notification(RelayNotification::Message { message }, true);
            }
            Ok(None) => (),
            Err(e) => tracing::error!(
                url = %self.url,
                msg = %msg,
                error = %e,
                "Impossible to handle relay message."
            ),
        }
    }

    async fn handle_raw_relay_message(
        &self,
        msg: &str,
    ) -> Result<Option<RelayMessage<'static>>, Error> {
        // Trim the message (removes leading and trailing whitespaces and line breaks).
        let msg: &str = msg.trim();

        // Get message size
        let size: usize = msg.len();

        tracing::debug!("Received '{msg}' from '{}' (size: {size} bytes)", self.url);

        // Update bytes received
        self.stats.add_bytes_received(size);

        // Check message size
        if let Some(max_size) = self.opts.limits.messages.max_size {
            let max_size: usize = max_size as usize;
            if size > max_size {
                return Err(Error::RelayMessageTooLarge { size, max_size });
            }
        }

        // Handle msg
        match RelayMessage::from_json(msg)? {
            RelayMessage::Event {
                subscription_id,
                event,
            } => {
                self.handle_event_msg(subscription_id.into_owned(), event.into_owned())
                    .await
            }
            m => Ok(Some(m)),
        }
    }

    async fn handle_event_msg(
        &self,
        subscription_id: SubscriptionId,
        event: Event,
    ) -> Result<Option<RelayMessage<'static>>, Error> {
        // Check event size
        if let Some(max_size) = self.opts.limits.events.get_max_size(&event.kind) {
            let size: usize = event.as_json().len();
            let max_size: usize = max_size as usize;
            if size > max_size {
                return Err(Error::EventTooLarge { size, max_size });
            }
        }

        // Check tags limit
        if let Some(max_num_tags) = self.opts.limits.events.get_max_num_tags(&event.kind) {
            let size: usize = event.tags.len();
            let max_num_tags: usize = max_num_tags as usize;
            if size > max_num_tags {
                return Err(Error::TooManyTags {
                    size,
                    max_size: max_num_tags,
                });
            }
        }

        // Check if subscription must be verified
        if self.opts.verify_subscriptions || self.opts.ban_relay_on_mismatch {
            // NOTE: here we don't use the `self.subscription(id)` to avoid an unnecessary clone of the filter!

            // Acquire read lock
            let subscriptions = self.atomic.subscriptions.read().await;

            // Check if the subscription id exist and verify if the event matches the subscription filter.
            let SubscriptionData {
                filters,
                received_eose,
                received_events,
                ..
            } = subscriptions
                .get(&subscription_id)
                .ok_or(Error::SubscriptionNotFound)?;

            // Check filter limit ONLY if EOSE is not received yet and if there is only ONE filter.
            // We can't ensure that limit is respected if there is more than one filter.
            if !received_eose && filters.len() == 1 {
                // SAFETY: we've checked above that exists one filter.
                let filter: &Filter = &filters[0];

                // Check if the filter has a limit
                if let Some(limit) = filter.limit {
                    // Update number of received events
                    let prev: usize = received_events.fetch_add(1, Ordering::SeqCst);
                    let received_events: usize = prev.saturating_add(1);

                    // Check if received more that requested
                    if received_events > limit {
                        // Ban the relay
                        if self.opts.ban_relay_on_mismatch {
                            self.ban();
                        }

                        return Err(Error::TooManyEvents);
                    }
                }
            }

            // Check if the filter matches the event
            for filter in filters.iter() {
                if !filter.match_event(&event, MATCH_EVENT_OPTS) {
                    // Ban the relay
                    if self.opts.ban_relay_on_mismatch {
                        self.ban();
                    }

                    return Err(Error::EventNotMatchFilter);
                }
            }
        }

        // Check if the event is expired
        if event.is_expired() {
            return Err(Error::EventExpired);
        }

        // Check event admission policy
        if let Some(policy) = &self.state.admit_policy {
            if let AdmitStatus::Rejected { .. } = policy
                .admit_event(&self.url, &subscription_id, &event)
                .await?
            {
                return Ok(None);
            }
        }

        // Check the event status
        match self.state.database().check_id(&event.id).await? {
            // Already saved, continue with code execution
            DatabaseEventStatus::Saved => {}
            // Deleted, immediately return
            DatabaseEventStatus::Deleted => return Ok(None),
            // Not existent, verify the event and try to save it to the database
            DatabaseEventStatus::NotExistent => {
                // Check if the event was already verified.
                //
                // This is useful if someone continues to send the same invalid event:
                // since invalid events aren't stored in the database,
                // skipping this check would result in the re-verification of the event.
                // This may also be useful to avoid double verification if the event is received at the exact same time by many different Relay instances.
                //
                // This is important since event signature verification is a heavy job!
                if !self.state.verified(&event.id).await {
                    event.verify()?;
                }

                // Save into the database
                let send_notification: bool = match self.state.database().save_event(&event).await?
                {
                    SaveEventStatus::Success => true,
                    SaveEventStatus::Rejected(reason) => match reason {
                        RejectedReason::Ephemeral => true,
                        RejectedReason::Duplicate => true,
                        RejectedReason::Deleted => false,
                        RejectedReason::Expired => false,
                        RejectedReason::Replaced => false,
                        RejectedReason::InvalidDelete => false,
                        RejectedReason::Vanished => false,
                        RejectedReason::Other => true,
                    },
                };

                // If the notification should NOT be sent, immediately return.
                if !send_notification {
                    return Ok(None);
                }

                // Send notification
                self.send_notification(
                    RelayNotification::Event {
                        subscription_id: subscription_id.clone(),
                        event: Box::new(event.clone()),
                    },
                    true,
                );
            }
        }

        Ok(Some(RelayMessage::Event {
            subscription_id: Cow::Owned(subscription_id),
            event: Cow::Owned(event),
        }))
    }

    pub fn disconnect(&self) {
        let status = self.status();

        // Check if it's already terminated, banned or shutdown
        if status.is_terminated() || status.is_banned() || status.is_shutdown() {
            return;
        }

        // Notify termination
        self.atomic.channels.terminate();

        // Update status
        self.set_status(RelayStatus::Terminated, true);
    }

    pub fn ban(&self) {
        let status = self.status();

        // Check if it's already terminated, banned or shutdown
        if status.is_terminated() || status.is_banned() || status.is_shutdown() {
            return;
        }

        // Notify termination
        self.atomic.channels.terminate();

        // Update status
        self.set_status(RelayStatus::Banned, true);
    }

    pub(super) fn shutdown(&self) {
        let status = self.status();

        // Check if it's already terminated, banned or shutdown
        if status.is_terminated() || status.is_banned() || status.is_shutdown() {
            return;
        }

        // Notify termination
        self.atomic.channels.terminate();

        // Update status
        self.set_status(RelayStatus::Shutdown, true);
    }

    #[inline]
    pub(super) async fn send_msg(
        &self,
        msg: ClientMessage<'_>,
        wait_until_sent: Option<Duration>,
    ) -> Result<(), Error> {
        // Check if relay is operational
        self.ensure_operational()?;

        // If it can't write, check if there are "write" messages
        if !self.capabilities.can_write() && msg.is_event() {
            return Err(Error::WriteDisabled);
        }

        // If it can't read, check if there are "read" messages
        if !self.capabilities.can_read() && (msg.is_req() || msg.is_close()) {
            return Err(Error::ReadDisabled);
        }

        match wait_until_sent {
            Some(timeout) => {
                // Create a channel
                let (tx, rx) = oneshot::channel();

                // Send the item
                self.atomic.channels.send_client_msg(JsonMessageItem {
                    json: msg.as_json(),
                    confirmation: Some(tx),
                })?;

                // Wait for confirmation
                Ok(time::timeout(Some(timeout), rx)
                    .await
                    .ok_or(Error::Timeout)??)
            }
            None => self.atomic.channels.send_client_msg(JsonMessageItem {
                json: msg.as_json(),
                confirmation: None,
            }),
        }
    }

    async fn auth(&self, challenge: String) -> Result<(), Error> {
        // Get signer
        let signer = self.state.signer().await?;

        // Construct event
        let event: Event = EventBuilder::auth(challenge, self.url.clone())
            .sign(&signer)
            .await?;

        // Subscribe to notifications
        let mut notifications = self.internal_notification_sender.subscribe();

        // Send the AUTH message
        self.send_msg(ClientMessage::Auth(Cow::Borrowed(&event)), None)
            .await?;

        // Wait for OK
        // The event ID is already checked in `wait_for_ok` method
        let (status, message) = self
            .wait_for_ok(&mut notifications, &event.id, Duration::from_secs(10))
            .await?;

        // Check status
        if status {
            Ok(())
        } else {
            Err(Error::RelayMessage(message))
        }
    }

    pub(super) async fn wait_for_ok(
        &self,
        notifications: &mut broadcast::Receiver<RelayNotification>,
        id: &EventId,
        timeout: Duration,
    ) -> Result<(bool, String), Error> {
        time::timeout(Some(timeout), async {
            while let Ok(notification) = notifications.recv().await {
                match notification {
                    RelayNotification::Message {
                        message:
                            RelayMessage::Ok {
                                event_id,
                                status,
                                message,
                            },
                    } => {
                        // Check if it can return
                        if id == &event_id {
                            return Ok((status, message.into_owned()));
                        }
                    }
                    RelayNotification::RelayStatus { status } => {
                        if status.is_disconnected() {
                            return Err(Error::NotConnected);
                        }
                    }
                    _ => (),
                }
            }

            Err(Error::PrematureExit)
        })
        .await
        .ok_or(Error::Timeout)?
    }

    pub async fn resubscribe(&self) -> Result<(), Error> {
        // TODO: avoid subscriptions clone
        let subscriptions = self.subscriptions().await;
        for (id, filters) in subscriptions.into_iter() {
            if !filters.is_empty() && self.should_resubscribe(&id).await {
                self.send_msg(ClientMessage::req(id, filters), None).await?;
            } else {
                tracing::debug!("Skip re-subscription of '{id}'");
            }
        }

        Ok(())
    }

    pub(super) fn spawn_auto_closing_handler(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeAutoCloseOptions,
        notifications: broadcast::Receiver<RelayNotification>,
        activity: Option<Sender<SubscriptionActivity>>,
    ) {
        let relay = self.clone(); // <-- FULL RELAY CLONE HERE
        task::spawn(async move {
            // Check if CLOSE needed
            let to_close: bool = match relay
                .handle_auto_closing(&id, &filters, opts, notifications, &activity)
                .await
            {
                Some(HandleAutoClosing { to_close, reason }) => {
                    // Send activity
                    if let Some(reason) = reason {
                        if let Some(activity) = &activity {
                            // TODO: handle error?
                            let _ = activity.send(SubscriptionActivity::Closed(reason)).await;
                        }
                    }

                    to_close
                }
                // Timeout
                None => {
                    tracing::warn!(id = %id, "Timeout reached for subscription, auto-closing.");
                    true
                }
            };

            // Drop activity sender to terminate the receiver activity loop
            drop(activity);

            // Close subscription
            let send_result = if to_close {
                tracing::debug!(id = %id, "Auto-closing subscription.");
                relay
                    .send_msg(ClientMessage::Close(Cow::Borrowed(&id)), None)
                    .await
            } else {
                Ok(())
            };

            // Remove subscription
            relay.remove_subscription(&id).await;

            send_result
        });
    }

    async fn handle_auto_closing(
        &self,
        id: &SubscriptionId,
        filters: &[Filter],
        opts: SubscribeAutoCloseOptions,
        mut notifications: broadcast::Receiver<RelayNotification>,
        activity: &Option<Sender<SubscriptionActivity>>,
    ) -> Option<HandleAutoClosing> {
        time::timeout(opts.timeout, async move {
            let mut wait_for_events_counter: u16 = 0;
            let mut wait_for_events_after_eose_counter: u16 = 0;
            let mut received_eose: bool = false;
            let mut require_resubscription: bool = false;
            let mut last_event: Option<Instant> = None;

            // Listen to notifications with timeout
            // If no notification is received within no-events timeout, `None` is returned.
            while let Ok(notification) =
                time::timeout(opts.idle_timeout, notifications.recv()).await?
            {
                // Check if no-events timeout is reached
                if let (Some(idle_timeout), Some(last_event)) = (opts.idle_timeout, last_event) {
                    if last_event.elapsed() > idle_timeout {
                        // Close the subscription
                        return Some(HandleAutoClosing {
                            to_close: true,
                            reason: None,
                        });
                    }
                }

                match notification {
                    RelayNotification::Message { message, .. } => match message {
                        RelayMessage::Event {
                            subscription_id,
                            event,
                        } => {
                            if subscription_id.as_ref() == id {
                                // Send activity
                                if let Some(activity) = activity {
                                    // TODO: handle error?
                                    let _ = activity
                                        .send(SubscriptionActivity::ReceivedEvent(
                                            event.into_owned(),
                                        ))
                                        .await;
                                }

                                // If no-events timeout is enabled, update instant of last event received
                                if opts.idle_timeout.is_some() {
                                    last_event = Some(Instant::now());
                                }

                                // Check exit policy
                                match opts.exit_policy {
                                    ReqExitPolicy::WaitForEvents(num) => {
                                        wait_for_events_counter += 1;
                                        if wait_for_events_counter >= num {
                                            break;
                                        }
                                    }
                                    ReqExitPolicy::WaitForEventsAfterEOSE(num) => {
                                        if received_eose {
                                            wait_for_events_after_eose_counter += 1;
                                            if wait_for_events_after_eose_counter >= num {
                                                break;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        RelayMessage::EndOfStoredEvents(subscription_id) => {
                            if subscription_id.as_ref() == id {
                                received_eose = true;
                                if let ReqExitPolicy::ExitOnEOSE
                                | ReqExitPolicy::WaitDurationAfterEOSE(_) = opts.exit_policy
                                {
                                    break;
                                }
                            }
                        }
                        RelayMessage::Closed {
                            subscription_id,
                            message,
                        } => {
                            if subscription_id.as_ref() == id {
                                // Check machine-readable prefix
                                match MachineReadablePrefix::parse(&message) {
                                    Some(MachineReadablePrefix::AuthRequired) => {
                                        // Authentication is not enabled, return.
                                        if !self.state.is_auto_authentication_enabled() {
                                            return Some(HandleAutoClosing {
                                                to_close: false, // No need to send CLOSE msg
                                                reason: Some(SubscriptionAutoClosedReason::Closed(
                                                    message.into_owned(),
                                                )),
                                            });
                                        }

                                        // Needs to re-subscribe
                                        require_resubscription = true;
                                    }
                                    Some(_) => {
                                        return Some(HandleAutoClosing {
                                            to_close: false, // No need to send CLOSE msg
                                            reason: Some(SubscriptionAutoClosedReason::Closed(
                                                message.into_owned(),
                                            )),
                                        });
                                    }
                                    // Mark subscription as completed.
                                    //
                                    // If we are arrived at this point,
                                    // means that no error should be occurred,
                                    // so the subscription can be marked as completed.
                                    //
                                    // # Example
                                    //
                                    // Send a request with `{"ids":["<id>"]}` filter.
                                    // In this case, when the relay sends the matching event,
                                    // it no longer makes sense to keep the subscription open,
                                    // as no more events will ever be served.
                                    // Discussion: https://github.com/nostrability/nostrability/issues/167
                                    None => {
                                        return Some(HandleAutoClosing {
                                            to_close: false,
                                            reason: Some(SubscriptionAutoClosedReason::Completed),
                                        });
                                    }
                                }
                            }
                        }
                        _ => (),
                    },
                    RelayNotification::Authenticated => {
                        // Resend REQ
                        if require_resubscription {
                            require_resubscription = false;
                            let msg = ClientMessage::Req {
                                subscription_id: Cow::Borrowed(id),
                                filters: filters.iter().map(Cow::Borrowed).collect(),
                            };
                            let _ = self.send_msg(msg, None).await;
                        }
                    }
                    RelayNotification::AuthenticationFailed => {
                        return Some(HandleAutoClosing {
                            to_close: false, // No need to send CLOSE msg
                            reason: Some(SubscriptionAutoClosedReason::AuthenticationFailed),
                        });
                    }
                    RelayNotification::RelayStatus { status } => {
                        if status.is_disconnected() {
                            return Some(HandleAutoClosing {
                                to_close: false, // No need to send CLOSE msg
                                reason: None,
                            });
                        }
                    }
                    _ => (),
                }
            }

            if let ReqExitPolicy::WaitDurationAfterEOSE(duration) = opts.exit_policy {
                time::timeout(Some(duration), async {
                    while let Ok(notification) = notifications.recv().await {
                        match notification {
                            RelayNotification::Message {
                                message:
                                    RelayMessage::Event {
                                        subscription_id,
                                        event,
                                    },
                            } => {
                                if subscription_id.as_ref() == id {
                                    // Send activity
                                    if let Some(activity) = activity {
                                        // TODO: handle error?
                                        let _ = activity
                                            .send(SubscriptionActivity::ReceivedEvent(
                                                event.into_owned(),
                                            ))
                                            .await;
                                    }
                                }
                            }
                            RelayNotification::RelayStatus { status } => {
                                if status.is_disconnected() {
                                    return Ok(());
                                }
                            }
                            _ => (),
                        }
                    }

                    Ok::<(), Error>(())
                })
                .await;
            }

            Some(HandleAutoClosing {
                to_close: true, // Need to send CLOSE msg
                reason: Some(SubscriptionAutoClosedReason::Completed),
            })
        })
        .await?
    }

    // Returns `true` if the subscription has been unsubscribed
    async fn _unsubscribe_long_lived_subscription(
        &self,
        subscriptions: &mut RwLockWriteGuard<'_, HashMap<SubscriptionId, SubscriptionData>>,
        id: Cow<'_, SubscriptionId>,
    ) -> Result<bool, Error> {
        match subscriptions.remove(&id) {
            Some(sub) => {
                // Re-insert if auto-closing
                if sub.is_auto_closing {
                    subscriptions.insert(id.into_owned(), sub);
                    return Ok(false);
                }

                // Send CLOSE message
                self.send_msg(ClientMessage::Close(id), None).await?;

                Ok(true)
            }
            // Not existent subscription
            None => Ok(false),
        }
    }

    pub async fn unsubscribe(&self, id: &SubscriptionId) -> Result<bool, Error> {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        self._unsubscribe_long_lived_subscription(&mut subscriptions, Cow::Borrowed(id))
            .await
    }

    pub async fn unsubscribe_all(&self) -> Result<(), Error> {
        let mut subscriptions = self.atomic.subscriptions.write().await;

        // All IDs
        let ids: Vec<SubscriptionId> = subscriptions.keys().cloned().collect();

        // Unsubscribe
        for id in ids.into_iter() {
            self._unsubscribe_long_lived_subscription(&mut subscriptions, Cow::Owned(id))
                .await?;
        }

        Ok(())
    }
}

/// Send a WebSocket message with timeout set to [WEBSOCKET_TX_TIMEOUT].
async fn send_ws_msg(tx: &mut WebSocketSink, msg: Message) -> Result<(), Error> {
    match time::timeout(Some(WEBSOCKET_TX_TIMEOUT), tx.send(msg)).await {
        Some(res) => Ok(res?),
        None => Err(Error::Timeout),
    }
}

/// Send the close message with timeout set to [WEBSOCKET_TX_TIMEOUT].
async fn close_ws(tx: &mut WebSocketSink) -> Result<(), Error> {
    // TODO: remove timeout from here?
    match time::timeout(Some(WEBSOCKET_TX_TIMEOUT), tx.close()).await {
        Some(res) => Ok(res?),
        None => Err(Error::Timeout),
    }
}

#[cfg(bench)]
mod benches {
    use std::sync::LazyLock;

    use test::Bencher;
    use tokio::runtime::Runtime;

    use super::*;

    static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

    fn relay() -> InnerRelay {
        let url = RelayUrl::parse("ws://localhost:8080").unwrap();
        let state = SharedState::default();
        let opts = RelayOptions::default();
        InnerRelay::new(url, state, opts)
    }

    #[bench]
    fn bench_handle_relay_msg_event(bh: &mut Bencher) {
        let relay = relay();

        let msg = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

        bh.iter(|| {
            RUNTIME.block_on(async {
                relay.handle_raw_relay_message(msg).await.unwrap();
            });
        });
    }

    #[bench]
    fn bench_handle_relay_msg_invalid_event(bh: &mut Bencher) {
        let relay = relay();

        let msg = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"fa163f5cfb75d77d9b6269011872ee22b34fb48d23251e9879bb1e4ccbdd8aaaf4b6dc5f5084a65ef42c52fbcde8f3178bac3ba207de827ec513a6aa39fa684c"}]"#;

        bh.iter(|| {
            RUNTIME.block_on(async {
                let _ = relay.handle_raw_relay_message(msg).await;
            });
        });
    }
}
