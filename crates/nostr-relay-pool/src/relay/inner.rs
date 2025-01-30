// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::cmp;
use std::collections::{HashMap, HashSet};
#[cfg(feature = "nip11")]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::{task, time};
use async_wsocket::futures_util::{self, SinkExt, StreamExt};
use async_wsocket::{ConnectionMode, Message};
use atomic_destructor::AtomicDestroyer;
use negentropy::{Id, Negentropy, NegentropyStorageVector};
use negentropy_deprecated::{Bytes as BytesDeprecated, Negentropy as NegentropyDeprecated};
use nostr::event::raw::RawEvent;
use nostr::secp256k1::rand::{self, Rng};
use nostr_database::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, Mutex, MutexGuard, Notify, OnceCell, RwLock};

use super::constants::{
    BATCH_EVENT_ITERATION_TIMEOUT, DEFAULT_CONNECTION_TIMEOUT, JITTER_RANGE, MAX_RETRY_INTERVAL,
    MIN_ATTEMPTS, MIN_SUCCESS_RATE, NEGENTROPY_BATCH_SIZE_DOWN, NEGENTROPY_FRAME_SIZE_LIMIT,
    NEGENTROPY_HIGH_WATER_UP, NEGENTROPY_LOW_WATER_UP, PING_INTERVAL, WEBSOCKET_TX_TIMEOUT,
};
use super::filtering::CheckFiltering;
use super::flags::AtomicRelayServiceFlags;
use super::options::{RelayOptions, ReqExitPolicy, SubscribeAutoCloseOptions, SyncOptions};
use super::ping::PingTracker;
use super::stats::RelayConnectionStats;
use super::{Error, Reconciliation, RelayNotification, RelayStatus, SubscriptionAutoClosedReason};
use crate::pool::RelayPoolNotification;
use crate::relay::status::AtomicRelayStatus;
use crate::shared::SharedState;
use crate::transport::websocket::{BoxSink, BoxStream};

#[derive(Debug)]
struct RelayChannels {
    nostr: (
        Sender<Vec<ClientMessage>>,
        Mutex<Receiver<Vec<ClientMessage>>>,
    ),
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

    pub fn send_client_msgs(&self, msgs: Vec<ClientMessage>) -> Result<(), Error> {
        self.nostr
            .0
            .try_send(msgs)
            .map_err(|_| Error::CantSendChannelMessage {
                channel: String::from("nostr"),
            })
    }

    #[inline]
    pub async fn rx_nostr(&self) -> MutexGuard<'_, Receiver<Vec<ClientMessage>>> {
        self.nostr.1.lock().await
    }

    #[inline]
    pub fn nostr_queue(&self) -> usize {
        self.nostr.0.max_capacity() - self.nostr.0.capacity()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn ping(&self) {
        self.ping.notify_one()
    }

    pub fn terminate(&self) {
        self.terminate.notify_one()
    }
}

#[derive(Debug, Clone)]
struct SubscriptionData {
    pub filter: Filter,
    pub subscribed_at: Timestamp,
    /// Subscription closed by relay
    pub closed: bool,
}

impl Default for SubscriptionData {
    fn default() -> Self {
        Self {
            // TODO: use `Option<Filter>`?
            filter: Filter::new(),
            subscribed_at: Timestamp::zero(),
            closed: false,
        }
    }
}

// Instead of wrap every field in an `Arc<T>`, which increases the number of atomic operations,
// put all fields that require an `Arc` here.
#[derive(Debug)]
pub(super) struct AtomicPrivateData {
    status: AtomicRelayStatus,
    #[cfg(feature = "nip11")]
    pub(super) document: RwLock<RelayInformationDocument>,
    #[cfg(feature = "nip11")]
    last_document_fetch: AtomicU64,
    channels: RelayChannels,
    subscriptions: RwLock<HashMap<SubscriptionId, SubscriptionData>>,
    running: AtomicBool,
}

#[derive(Debug, Clone)]
pub(crate) struct InnerRelay {
    pub(super) url: RelayUrl,
    pub(super) atomic: Arc<AtomicPrivateData>,
    pub(super) opts: RelayOptions,
    pub(super) flags: AtomicRelayServiceFlags,
    pub(super) stats: RelayConnectionStats,
    pub(super) state: SharedState,
    pub(super) internal_notification_sender: broadcast::Sender<RelayNotification>,
    external_notification_sender: OnceCell<broadcast::Sender<RelayPoolNotification>>,
}

impl AtomicDestroyer for InnerRelay {
    fn on_destroy(&self) {
        self.disconnect();
    }
}

impl InnerRelay {
    pub(super) fn new(url: RelayUrl, state: SharedState, opts: RelayOptions) -> Self {
        let (relay_notification_sender, ..) = broadcast::channel::<RelayNotification>(2048);

        Self {
            url,
            atomic: Arc::new(AtomicPrivateData {
                status: AtomicRelayStatus::default(),
                #[cfg(feature = "nip11")]
                document: RwLock::new(RelayInformationDocument::new()),
                #[cfg(feature = "nip11")]
                last_document_fetch: AtomicU64::new(0),
                channels: RelayChannels::new(),
                subscriptions: RwLock::new(HashMap::new()),
                running: AtomicBool::new(false),
            }),
            flags: AtomicRelayServiceFlags::new(opts.flags),
            opts,
            stats: RelayConnectionStats::default(),
            state,
            internal_notification_sender: relay_notification_sender,
            external_notification_sender: OnceCell::new(),
        }
    }

    #[inline]
    pub fn connection_mode(&self) -> &ConnectionMode {
        &self.opts.connection_mode
    }

    /// Is connection task running?
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
            }
        }

        // Send notification
        self.send_notification(RelayNotification::RelayStatus { status }, true);
    }

    /// Perform health checks
    pub(super) fn health_check(&self) -> Result<(), Error> {
        let status: RelayStatus = self.status();

        // Relay not ready (never called connect method)
        if status.is_initialized() {
            return Err(Error::NotReady);
        }

        if !status.is_connected()
            && self.stats.attempts() > MIN_ATTEMPTS
            && self.stats.success_rate() < MIN_SUCCESS_RATE
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

    #[cfg(feature = "nip11")]
    fn request_nip11_document(&self) {
        let (allowed, proxy) = match self.opts.connection_mode {
            ConnectionMode::Direct => (true, None),
            #[cfg(not(target_arch = "wasm32"))]
            ConnectionMode::Proxy(proxy) => (true, Some(proxy)),
            #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
            ConnectionMode::Tor { .. } => (false, None),
        };

        if allowed {
            let now: u64 = Timestamp::now().as_u64();

            // Check last fetch
            if self.atomic.last_document_fetch.load(Ordering::SeqCst) + 3600 < now {
                // Update last fetch
                self.atomic.last_document_fetch.store(now, Ordering::SeqCst);

                // Fetch
                let url = self.url.clone();
                let atomic = self.atomic.clone();
                task::spawn(async move {
                    match RelayInformationDocument::get(url.clone().into(), proxy).await {
                        Ok(document) => {
                            let mut d = atomic.document.write().await;
                            *d = document
                        }
                        Err(e) => {
                            tracing::warn!(url = %url, error = %e, "Can't get information document.")
                        }
                    };
                });
            }
        }
    }

    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Filter> {
        let subscription = self.atomic.subscriptions.read().await;
        subscription
            .iter()
            .map(|(k, v)| (k.clone(), v.filter.clone()))
            .collect()
    }

    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Filter> {
        let subscription = self.atomic.subscriptions.read().await;
        subscription.get(id).map(|d| d.filter.clone())
    }

    pub(crate) async fn update_subscription(
        &self,
        id: SubscriptionId,
        filter: Filter,
        update_subscribed_at: bool,
    ) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        let data: &mut SubscriptionData = subscriptions.entry(id).or_default();
        data.filter = filter;

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

    /// Check if it should subscribe for current websocket session
    pub(crate) async fn should_resubscribe(&self, id: &SubscriptionId) -> bool {
        let subscriptions = self.atomic.subscriptions.read().await;
        match subscriptions.get(id) {
            Some(SubscriptionData {
                subscribed_at,
                closed,
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
            None => false,
        }
    }

    pub(crate) async fn remove_subscription(&self, id: &SubscriptionId) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        subscriptions.remove(id);
    }

    #[inline]
    pub fn queue(&self) -> usize {
        self.atomic.channels.nostr_queue()
    }

    pub(crate) fn set_notification_sender(
        &self,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
    ) -> Result<(), Error> {
        self.external_notification_sender.set(notification_sender)?;
        Ok(())
    }

    fn send_notification(&self, notification: RelayNotification, external: bool) {
        match (external, self.external_notification_sender.get()) {
            (true, Some(external_notification_sender)) => {
                // Clone and send internal notification
                let _ = self.internal_notification_sender.send(notification.clone());

                // Convert relay to notification to pool notification
                let notification: Option<RelayPoolNotification> = match notification {
                    RelayNotification::Event {
                        subscription_id,
                        event,
                    } => Some(RelayPoolNotification::Event {
                        relay_url: self.url.clone(),
                        subscription_id,
                        event,
                    }),
                    RelayNotification::Message { message } => {
                        Some(RelayPoolNotification::Message {
                            relay_url: self.url.clone(),
                            message,
                        })
                    }
                    RelayNotification::RelayStatus { .. } => None,
                    RelayNotification::Authenticated => None,
                    RelayNotification::AuthenticationFailed => None,
                    RelayNotification::SubscriptionAutoClosed { .. } => None,
                    RelayNotification::Shutdown => Some(RelayPoolNotification::Shutdown),
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

    pub(super) fn spawn_connection_task(&self, mut stream: Option<(BoxSink, BoxStream)>) {
        if self.is_running() {
            tracing::warn!(url = %self.url, "Connection task is already running.");
            return;
        }

        let relay = self.clone();
        task::spawn(async move {
            // Set that connection task is running
            relay.atomic.running.store(true, Ordering::SeqCst);

            // Lock receiver
            let mut rx_nostr = relay.atomic.channels.rx_nostr().await;

            // Last websocket error
            // Store it to avoid printing every time the same connection error
            let mut last_ws_error = None;

            // Auto-connect loop
            loop {
                // TODO: check in the relays state database if relay can connect (different from the previous check)
                // TODO: if the relay score is too low, immediately exit.
                // TODO: at every loop iteration check the score and if it's too low, exit

                // Connect and run message handler
                // The termination requests are handled inside this method!
                relay
                    .connect_and_run(stream, &mut rx_nostr, &mut last_ws_error)
                    .await;

                // Update stream to `None`, meaning that it was already used (if was some).
                stream = None;

                // Get status
                let status: RelayStatus = relay.status();

                // If the status is set to "terminated", break loop.
                if status.is_terminated() {
                    break;
                }

                // Check if reconnection is enabled
                if relay.opts.reconnect {
                    // Check if relay is marked as disconnected. If not, update status.
                    // Check if disconnected to avoid a possible double log
                    if !status.is_disconnected() {
                        relay.set_status(RelayStatus::Disconnected, true);
                    }

                    // Sleep before retry to connect
                    let interval: Duration = relay.calculate_retry_interval();
                    tracing::debug!(
                        "Reconnecting to '{}' relay in {} secs",
                        relay.url,
                        interval.as_secs()
                    );

                    // Sleep before retry to connect
                    // Handle termination to allow to exit immediately if request is received during the sleep.
                    tokio::select! {
                        // Sleep
                        _ = time::sleep(interval) => {},
                        // Handle termination notification
                        _ = relay.handle_terminate() => break,
                    }
                } else {
                    // Reconnection disabled, set status to "terminated"
                    relay.set_status(RelayStatus::Terminated, true);

                    // Break loop and exit
                    tracing::debug!(url = %relay.url, "Reconnection disabled, breaking loop.");
                    break;
                }
            }

            // Set that connection task is no longer running
            relay.atomic.running.store(false, Ordering::SeqCst);

            tracing::debug!(url = %relay.url, "Auto connect loop terminated.");
        });
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
            let jitter: i8 = rand::thread_rng().gen_range(JITTER_RANGE);

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

    async fn handle_terminate(&self) {
        // Wait to be notified
        self.atomic.channels.terminate.notified().await;

        // Update status
        self.set_status(RelayStatus::Terminated, true);
    }

    pub(super) async fn _try_connect(
        &self,
        timeout: Duration,
        status_on_failure: RelayStatus,
    ) -> Result<(BoxSink, BoxStream), Error> {
        // Update status
        self.set_status(RelayStatus::Connecting, true);

        // Add attempt
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
        stream: Option<(BoxSink, BoxStream)>,
        rx_nostr: &mut MutexGuard<'_, Receiver<Vec<ClientMessage>>>,
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
        mut ws_tx: BoxSink,
        ws_rx: BoxStream,
        rx_nostr: &mut MutexGuard<'_, Receiver<Vec<ClientMessage>>>,
    ) {
        // Request information document
        #[cfg(feature = "nip11")]
        self.request_nip11_document();

        // (Re)subscribe to relay
        if self.flags.can_read() {
            if let Err(e) = self.resubscribe().await {
                tracing::error!(url = %self.url, error = %e, "Impossible to subscribe.")
            }
        }

        let ping: PingTracker = PingTracker::default();

        // Wait that one of the futures terminates/completes
        // Also also termination here, to allow to close the connection in case of termination request.
        tokio::select! {
            // Message receiver handler
            res = self.receiver_message_handler(ws_rx, &ping) => match res {
                Ok(()) => tracing::trace!(url = %self.url, "Relay received exited."),
                Err(e) => tracing::error!(url = %self.url, error = %e, "Relay receiver exited with error.")
            },
            // Message sender handler
            res = self.sender_message_handler(&mut ws_tx, rx_nostr, &ping) => match res {
                Ok(()) => tracing::trace!(url = %self.url, "Relay sender exited."),
                Err(e) => tracing::error!(url = %self.url, error = %e, "Relay sender exited with error.")
            },
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
        ws_tx: &mut BoxSink,
        rx_nostr: &mut MutexGuard<'_, Receiver<Vec<ClientMessage>>>,
        ping: &PingTracker,
    ) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        let _ping = ping;

        loop {
            tokio::select! {
                // Nostr channel receiver
                Some(msgs) = rx_nostr.recv() => {
                    // Serialize messages to JSON and compose WebSocket text messages
                    let msgs: Vec<Message> = msgs
                        .into_iter()
                        .map(|msg| Message::Text(msg.as_json()))
                        .collect();

                    // Calculate messages size
                    let size: usize = msgs.iter().map(|msg| msg.len()).sum();
                    let len: usize = msgs.len();

                    // Log
                    if len == 1 {
                        let json = &msgs[0]; // SAFETY: len checked above (len == 1)
                        tracing::debug!("Sending '{json}' to '{}' (size: {size} bytes)", self.url);
                    } else {
                        tracing::debug!("Sending {len} messages to '{}' (size: {size} bytes)", self.url);
                    };

                    // Send WebSocket messages
                    send_ws_msgs(ws_tx, msgs).await?;

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
                        let nonce: u64 = rand::random();
                        ping.set_last_nonce(nonce);
                        ping.set_replied(false);

                        // Compose ping message
                        let msg = Message::Ping(nonce.to_be_bytes().to_vec());

                        // Send WebSocket message
                        send_ws_msgs(ws_tx, vec![msg]).await?;

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
        mut ws_rx: BoxStream,
        ping: &PingTracker,
    ) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        let _ping = ping;

        while let Some(msg) = ws_rx.next().await {
            match msg? {
                Message::Text(json) => self.handle_relay_message(&json).await,
                Message::Binary(_) => {
                    tracing::warn!(url = %self.url, "Binary messages aren't supported.");
                }
                #[cfg(not(target_arch = "wasm32"))]
                Message::Pong(bytes) => {
                    if self.flags.has_ping() && self.state.transport.support_ping() {
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

    /// Send a signal every [`PING_INTERVAL`] to the other tasks, asking to ping the relay.
    async fn pinger(&self) {
        loop {
            // Check if support ping
            #[cfg(not(target_arch = "wasm32"))]
            if self.flags.has_ping() && self.state.transport.support_ping() {
                // Ping supported, ping!
                self.atomic.channels.ping();
            }

            // Sleep
            time::sleep(PING_INTERVAL).await;
        }
    }

    async fn handle_relay_message(&self, msg: &str) {
        match self.handle_raw_relay_message(msg).await {
            Ok(Some(message)) => {
                match &message {
                    RelayMessage::Notice(message) => {
                        tracing::warn!(url = %self.url, msg = %message, "Received NOTICE.")
                    }
                    RelayMessage::Ok {
                        event_id,
                        status,
                        message,
                    } => {
                        tracing::debug!(
                            url = %self.url,
                            id = %event_id,
                            status = %status,
                            msg = %message,
                            "Received OK."
                        );
                    }
                    RelayMessage::EndOfStoredEvents(id) => {
                        tracing::debug!(
                            url = %self.url,
                            id = %id,
                            "Received EOSE."
                        );
                    }
                    RelayMessage::Closed {
                        subscription_id,
                        message,
                    } => {
                        tracing::debug!(
                            url = %self.url,
                            id = %subscription_id,
                            msg = %message,
                            "Subscription closed."
                        );
                        self.subscription_closed(subscription_id).await;
                    }
                    RelayMessage::Auth { challenge } => {
                        tracing::debug!(
                            url = %self.url,
                            challenge = %challenge,
                            "Received auth challenge."
                        );

                        // Check if NIP42 auto authentication is enabled
                        if self.state.is_auto_authentication_enabled() {
                            let relay = self.clone();
                            let challenge: String = challenge.clone();
                            task::spawn(async move {
                                // Authenticate to relay
                                match relay.auth(challenge).await {
                                    Ok(..) => {
                                        relay.send_notification(
                                            RelayNotification::Authenticated,
                                            false,
                                        );

                                        tracing::info!(url = %relay.url, "Authenticated to relay.");

                                        // TODO: ?
                                        if let Err(e) = relay.resubscribe().await {
                                            tracing::error!(
                                                url = %relay.url,
                                                error = %e,
                                                "Impossible to resubscribe."
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        relay.send_notification(
                                            RelayNotification::AuthenticationFailed,
                                            false,
                                        );

                                        tracing::error!(
                                            url = %relay.url,
                                            error = %e,
                                            "Can't authenticate to relay."
                                        );
                                    }
                                }
                            });
                        }
                    }
                    _ => (),
                }

                // Send notification
                self.send_notification(RelayNotification::Message { message }, true);
            }
            Ok(None) | Err(Error::MessageHandle(MessageHandleError::EmptyMsg)) => (),
            Err(e) => tracing::error!(
                url = %self.url,
                msg = %msg,
                error = %e,
                "Impossible to handle relay message."
            ),
        }
    }

    async fn handle_raw_relay_message(&self, msg: &str) -> Result<Option<RelayMessage>, Error> {
        let size: usize = msg.len();

        tracing::trace!(url = %self.url, size = %size, msg = %msg, "Received new relay message.");

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
        match RawRelayMessage::from_json(msg)? {
            RawRelayMessage::Event {
                subscription_id,
                event,
            } => self.handle_raw_event(subscription_id, event).await,
            m => Ok(Some(RelayMessage::try_from(m)?)),
        }
    }

    async fn handle_raw_event(
        &self,
        subscription_id: String,
        event: RawEvent,
    ) -> Result<Option<RelayMessage>, Error> {
        let kind: Kind = Kind::from(event.kind);

        // Check event size
        if let Some(max_size) = self.opts.limits.events.get_max_size(&kind) {
            let size: usize = event.as_json().len();
            let max_size: usize = max_size as usize;
            if size > max_size {
                return Err(Error::EventTooLarge { size, max_size });
            }
        }

        // Check tags limit
        if let Some(max_num_tags) = self.opts.limits.events.get_max_num_tags(&kind) {
            let size: usize = event.tags.len();
            let max_num_tags: usize = max_num_tags as usize;
            if size > max_num_tags {
                return Err(Error::TooManyTags {
                    size,
                    max_size: max_num_tags,
                });
            }
        }

        // Deserialize partial event (id, pubkey and sig)
        let partial_event: PartialEvent = PartialEvent::from_raw(&event)?;

        // Check filtering
        match self
            .state
            .filtering()
            .check_partial_event(&partial_event)
            .await
        {
            CheckFiltering::Allow => {
                // Nothing to do
            }
            CheckFiltering::EventIdBlacklisted(id) => {
                tracing::debug!("Received event with blacklisted ID: {id}");
                return Ok(None);
            }
            CheckFiltering::PublicKeyBlacklisted(pubkey) => {
                tracing::debug!("Received event authored by blacklisted public key: {pubkey}");
                return Ok(None);
            }
            CheckFiltering::PublicKeyNotInWhitelist(pubkey) => {
                tracing::debug!("Received event authored by non-whitelisted public key: {pubkey}");
                return Ok(None);
            }
        }

        // Check min POW
        let difficulty: u8 = self.state.minimum_pow_difficulty();
        if difficulty > 0 && !partial_event.id.check_pow(difficulty) {
            return Err(Error::PowDifficultyTooLow { min: difficulty });
        }

        // TODO: check if word/hashtag is blacklisted

        // Check if event status
        let status: DatabaseEventStatus = self.state.database().check_id(&partial_event.id).await?;

        // Event deleted
        if let DatabaseEventStatus::Deleted = status {
            return Ok(None);
        }

        // Deserialize missing fields
        let missing: MissingPartialEvent = MissingPartialEvent::from_raw(event)?;

        // Compose full event
        let event: Event = partial_event.merge(missing);

        // Check if it's expired
        if event.is_expired() {
            return Err(Error::EventExpired);
        }

        // Check if coordinate has been deleted
        if let Some(coordinate) = event.coordinate() {
            if self
                .state
                .database()
                .has_coordinate_been_deleted(&coordinate, &event.created_at)
                .await?
            {
                return Ok(None);
            }
        }

        let subscription_id: SubscriptionId = SubscriptionId::new(subscription_id);
        let event: Box<Event> = Box::new(event);

        // TODO: check if filter match

        // Check if event exists
        if let DatabaseEventStatus::NotExistent = status {
            // Verify event
            event.verify()?;

            // Save into database
            self.state.database().save_event(&event).await?;

            // Send notification
            self.send_notification(
                RelayNotification::Event {
                    subscription_id: subscription_id.clone(),
                    event: event.clone(),
                },
                true,
            );
        }

        Ok(Some(RelayMessage::Event {
            subscription_id,
            event,
        }))
    }

    pub fn disconnect(&self) {
        // Check if it's already terminated
        if self.status().is_terminated() {
            return;
        }

        self.atomic.channels.terminate();
        self.send_notification(RelayNotification::Shutdown, false);
    }

    #[inline]
    pub fn send_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        self.batch_msg(vec![msg])
    }

    pub fn batch_msg(&self, msgs: Vec<ClientMessage>) -> Result<(), Error> {
        // Perform health checks
        self.health_check()?;

        // Check if list is empty
        if msgs.is_empty() {
            return Err(Error::BatchMessagesEmpty);
        }

        // If it can't write, check if there are "write" messages
        if !self.flags.can_write() && msgs.iter().any(|msg| msg.is_event()) {
            return Err(Error::WriteDisabled);
        }

        // If it can't read, check if there are "read" messages
        if !self.flags.can_read() && msgs.iter().any(|msg| msg.is_req() || msg.is_close()) {
            return Err(Error::ReadDisabled);
        }

        // Send message
        self.atomic.channels.send_client_msgs(msgs)
    }

    fn send_neg_msg(&self, id: SubscriptionId, message: String) -> Result<(), Error> {
        self.send_msg(ClientMessage::NegMsg {
            subscription_id: id,
            message,
        })
    }

    fn send_neg_close(&self, id: SubscriptionId) -> Result<(), Error> {
        self.send_msg(ClientMessage::NegClose {
            subscription_id: id,
        })
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

        // Send AUTH message
        let id: EventId = event.id;
        self.send_msg(ClientMessage::auth(event))?;

        // Wait for OK
        // The event ID is already checked in `wait_for_ok` method
        let (_, status, message) = self
            .wait_for_ok(&mut notifications, id, BATCH_EVENT_ITERATION_TIMEOUT)
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
        id: EventId,
        timeout: Duration,
    ) -> Result<(EventId, bool, String), Error> {
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
                        if id == event_id {
                            return Ok((event_id, status, message));
                        }
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

    pub async fn resubscribe(&self) -> Result<(), Error> {
        let subscriptions = self.subscriptions().await;
        for (id, filter) in subscriptions.into_iter() {
            if !filter.is_empty() && self.should_resubscribe(&id).await {
                self.send_msg(ClientMessage::req(id, filter))?;
            } else {
                tracing::debug!("Skip re-subscription of '{id}'");
            }
        }

        Ok(())
    }

    pub(super) fn spawn_auto_closing_handler(
        &self,
        id: SubscriptionId,
        filter: Filter,
        opts: SubscribeAutoCloseOptions,
        notifications: broadcast::Receiver<RelayNotification>,
    ) {
        let relay = self.clone(); // <-- FULL RELAY CLONE HERE
        task::spawn(async move {
            // Check if CLOSE needed
            let to_close: bool = match relay
                .handle_auto_closing(&id, filter, opts, notifications)
                .await
            {
                Some((to_close, reason)) => {
                    // Send subscription auto-closed notification
                    if let Some(reason) = reason {
                        relay.send_notification(
                            RelayNotification::SubscriptionAutoClosed { reason },
                            false,
                        );
                    }

                    to_close
                }
                // Timeout
                None => {
                    tracing::warn!(id = %id, "Timeout reached for subscription, auto-closing.");
                    true
                }
            };

            // Close subscription
            if to_close {
                tracing::debug!(id = %id, "Auto-closing subscription.");
                relay.send_msg(ClientMessage::close(id))?;
            }

            Ok::<(), Error>(())
        });
    }

    async fn handle_auto_closing(
        &self,
        id: &SubscriptionId,
        filter: Filter,
        opts: SubscribeAutoCloseOptions,
        mut notifications: broadcast::Receiver<RelayNotification>,
    ) -> Option<(bool, Option<SubscriptionAutoClosedReason>)> {
        time::timeout(opts.timeout, async move {
            let mut counter: u16 = 0;
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
                        return Some((true, None)); // TODO: use SubscriptionAutoClosedReason::Timeout?
                    }
                }

                match notification {
                    RelayNotification::Message { message, .. } => match message {
                        RelayMessage::Event {
                            subscription_id, ..
                        } => {
                            if &subscription_id == id {
                                // If no-events timeout is enabled, update instant of last event received
                                if opts.idle_timeout.is_some() {
                                    last_event = Some(Instant::now());
                                }

                                if let ReqExitPolicy::WaitForEventsAfterEOSE(num) = opts.exit_policy
                                {
                                    if received_eose {
                                        counter += 1;
                                        if counter >= num {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        RelayMessage::EndOfStoredEvents(subscription_id) => {
                            if &subscription_id == id {
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
                            if &subscription_id == id {
                                // Check if auth required
                                match MachineReadablePrefix::parse(&message) {
                                    Some(MachineReadablePrefix::AuthRequired) => {
                                        if self.state.is_auto_authentication_enabled() {
                                            require_resubscription = true;
                                        } else {
                                            return Some((
                                                false,
                                                Some(SubscriptionAutoClosedReason::Closed(message)),
                                            )); // No need to send CLOSE msg
                                        }
                                    }
                                    _ => {
                                        return Some((
                                            false,
                                            Some(SubscriptionAutoClosedReason::Closed(message)),
                                        )); // No need to send CLOSE msg
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
                            let msg: ClientMessage = ClientMessage::req(id.clone(), filter.clone());
                            let _ = self.send_msg(msg);
                        }
                    }
                    RelayNotification::AuthenticationFailed => {
                        return Some((
                            false,
                            Some(SubscriptionAutoClosedReason::AuthenticationFailed),
                        )); // No need to send CLOSE msg
                    }
                    RelayNotification::RelayStatus { status } => {
                        if status.is_disconnected() {
                            return Some((false, None)); // No need to send CLOSE msg
                        }
                    }
                    RelayNotification::Shutdown => {
                        return Some((false, None)); // No need to send CLOSE msg
                    }
                    _ => (),
                }
            }

            if let ReqExitPolicy::WaitDurationAfterEOSE(duration) = opts.exit_policy {
                time::timeout(Some(duration), async {
                    while let Ok(notification) = notifications.recv().await {
                        match notification {
                            RelayNotification::RelayStatus { status } => {
                                if status.is_disconnected() {
                                    return Ok(());
                                }
                            }
                            RelayNotification::Shutdown => {
                                return Ok(());
                            }
                            _ => (),
                        }
                    }

                    Ok::<(), Error>(())
                })
                .await;
            }

            Some((true, Some(SubscriptionAutoClosedReason::Completed))) // Need to send CLOSE msg
        })
        .await?
    }

    pub async fn unsubscribe(&self, id: SubscriptionId) -> Result<(), Error> {
        // Remove subscription
        self.remove_subscription(&id).await;

        // Send CLOSE message
        self.send_msg(ClientMessage::close(id))
    }

    pub async fn unsubscribe_all(&self) -> Result<(), Error> {
        let subscriptions = self.subscriptions().await;

        for id in subscriptions.into_keys() {
            self.unsubscribe(id).await?;
        }

        Ok(())
    }

    #[inline(never)]
    fn handle_neg_msg<I>(
        &self,
        subscription_id: SubscriptionId,
        msg: Option<Vec<u8>>,
        curr_have_ids: I,
        curr_need_ids: I,
        opts: &SyncOptions,
        output: &mut Reconciliation,
        have_ids: &mut Vec<EventId>,
        need_ids: &mut Vec<EventId>,
        sync_done: &mut bool,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = EventId>,
    {
        let mut counter: u64 = 0;

        // If event ID wasn't already seen, add to the HAVE IDs
        // Add to HAVE IDs only if `do_up` is true
        for id in curr_have_ids.into_iter() {
            if output.local.insert(id) && opts.do_up() {
                have_ids.push(id);
                counter += 1;
            }
        }

        // If event ID wasn't already seen, add to the NEED IDs
        // Add to NEED IDs only if `do_down` is true
        for id in curr_need_ids.into_iter() {
            if output.remote.insert(id) && opts.do_down() {
                need_ids.push(id);
                counter += 1;
            }
        }

        if let Some(progress) = &opts.progress {
            progress.send_modify(|state| {
                state.total += counter;
            });
        }

        match msg {
            Some(query) => self.send_neg_msg(subscription_id, hex::encode(query)),
            None => {
                // Mark sync as done
                *sync_done = true;

                // Send NEG-CLOSE message
                self.send_neg_close(subscription_id)
            }
        }
    }

    #[inline(never)]
    async fn upload_neg_events(
        &self,
        have_ids: &mut Vec<EventId>,
        in_flight_up: &mut HashSet<EventId>,
        opts: &SyncOptions,
    ) -> Result<(), Error> {
        // Check if should skip the upload
        if !opts.do_up() || have_ids.is_empty() || in_flight_up.len() > NEGENTROPY_LOW_WATER_UP {
            return Ok(());
        }

        let mut num_sent = 0;

        while !have_ids.is_empty() && in_flight_up.len() < NEGENTROPY_HIGH_WATER_UP {
            if let Some(id) = have_ids.pop() {
                match self.state.database().event_by_id(&id).await {
                    Ok(Some(event)) => {
                        in_flight_up.insert(id);
                        self.send_msg(ClientMessage::event(event))?;
                        num_sent += 1;
                    }
                    Ok(None) => {
                        // Event not found
                    }
                    Err(e) => tracing::error!(
                        url = %self.url,
                        error = %e,
                        "Can't upload event."
                    ),
                }
            }
        }

        // Update progress
        if let Some(progress) = &opts.progress {
            progress.send_modify(|state| {
                state.current += num_sent;
            });
        }

        if num_sent > 0 {
            tracing::info!(
                "Negentropy UP for '{}': {} events ({} remaining)",
                self.url,
                num_sent,
                have_ids.len()
            );
        }

        Ok(())
    }

    #[inline(never)]
    fn req_neg_events(
        &self,
        need_ids: &mut Vec<EventId>,
        in_flight_down: &mut bool,
        down_sub_id: &SubscriptionId,
        opts: &SyncOptions,
    ) -> Result<(), Error> {
        // Check if should skip the download
        if !opts.do_down() || need_ids.is_empty() || *in_flight_down {
            return Ok(());
        }

        let capacity: usize = cmp::min(need_ids.len(), NEGENTROPY_BATCH_SIZE_DOWN);
        let mut ids: Vec<EventId> = Vec::with_capacity(capacity);

        while !need_ids.is_empty() && ids.len() < NEGENTROPY_BATCH_SIZE_DOWN {
            if let Some(id) = need_ids.pop() {
                ids.push(id);
            }
        }

        tracing::info!(
            "Negentropy DOWN for '{}': {} events ({} remaining)",
            self.url,
            ids.len(),
            need_ids.len()
        );

        // Update progress
        if let Some(progress) = &opts.progress {
            progress.send_modify(|state| {
                state.current += ids.len() as u64;
            });
        }

        let filter = Filter::new().ids(ids);
        self.send_msg(ClientMessage::req(down_sub_id.clone(), filter))?;

        *in_flight_down = true;

        Ok(())
    }

    #[inline(never)]
    fn handle_neg_ok(
        &self,
        in_flight_up: &mut HashSet<EventId>,
        event_id: EventId,
        status: bool,
        message: String,
        output: &mut Reconciliation,
    ) {
        if in_flight_up.remove(&event_id) {
            if status {
                output.sent.insert(event_id);
            } else {
                tracing::error!(
                    url = %self.url,
                    id = %event_id,
                    msg = %message,
                    "Can't upload event."
                );

                output
                    .send_failures
                    .entry(self.url.clone())
                    .and_modify(|map| {
                        map.insert(event_id, message.clone());
                    })
                    .or_default()
                    .insert(event_id, message);
            }
        }
    }

    /// New negentropy protocol
    #[inline(never)]
    pub(super) async fn sync_new(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: &SyncOptions,
        output: &mut Reconciliation,
    ) -> Result<(), Error> {
        // Prepare the negentropy client
        let storage: NegentropyStorageVector = prepare_negentropy_storage(items)?;
        let mut negentropy: Negentropy<NegentropyStorageVector> =
            Negentropy::borrowed(&storage, NEGENTROPY_FRAME_SIZE_LIMIT)?;

        // Initiate reconciliation
        let initial_message: Vec<u8> = negentropy.initiate()?;

        // Subscribe
        let mut notifications = self.internal_notification_sender.subscribe();
        let mut temp_notifications = self.internal_notification_sender.subscribe();

        // Send initial negentropy message
        let sub_id: SubscriptionId = SubscriptionId::generate();
        let open_msg: ClientMessage =
            ClientMessage::neg_open(sub_id.clone(), filter, hex::encode(initial_message));
        self.send_msg(open_msg)?;

        // Check if negentropy is supported
        check_negentropy_support(&sub_id, opts, &mut temp_notifications).await?;

        let mut in_flight_up: HashSet<EventId> = HashSet::new();
        let mut in_flight_down: bool = false;
        let mut sync_done: bool = false;
        let mut have_ids: Vec<EventId> = Vec::new();
        let mut need_ids: Vec<EventId> = Vec::new();
        let down_sub_id: SubscriptionId = SubscriptionId::generate();

        // Start reconciliation
        while let Ok(notification) = notifications.recv().await {
            match notification {
                RelayNotification::Message { message } => {
                    match message {
                        RelayMessage::NegMsg {
                            subscription_id,
                            message,
                        } => {
                            if subscription_id == sub_id {
                                let mut curr_have_ids: Vec<Id> = Vec::new();
                                let mut curr_need_ids: Vec<Id> = Vec::new();

                                // Parse message
                                let query: Vec<u8> = hex::decode(message)?;

                                // Reconcile
                                let msg: Option<Vec<u8>> = negentropy.reconcile_with_ids(
                                    &query,
                                    &mut curr_have_ids,
                                    &mut curr_need_ids,
                                )?;

                                // Handle message
                                self.handle_neg_msg(
                                    subscription_id,
                                    msg,
                                    curr_have_ids.into_iter().map(neg_id_to_event_id),
                                    curr_need_ids.into_iter().map(neg_id_to_event_id),
                                    opts,
                                    output,
                                    &mut have_ids,
                                    &mut need_ids,
                                    &mut sync_done,
                                )?;
                            }
                        }
                        RelayMessage::NegErr {
                            subscription_id,
                            message,
                        } => {
                            if subscription_id == sub_id {
                                return Err(Error::RelayMessage(message));
                            }
                        }
                        RelayMessage::Ok {
                            event_id,
                            status,
                            message,
                        } => {
                            self.handle_neg_ok(
                                &mut in_flight_up,
                                event_id,
                                status,
                                message,
                                output,
                            );
                        }
                        RelayMessage::Event {
                            subscription_id,
                            event,
                        } => {
                            if subscription_id == down_sub_id {
                                output.received.insert(event.id);
                            }
                        }
                        RelayMessage::EndOfStoredEvents(id) => {
                            if id == down_sub_id {
                                in_flight_down = false;
                            }
                        }
                        _ => (),
                    }

                    // Send events
                    self.upload_neg_events(&mut have_ids, &mut in_flight_up, opts)
                        .await?;

                    // Get events
                    self.req_neg_events(&mut need_ids, &mut in_flight_down, &down_sub_id, opts)?;
                }
                RelayNotification::RelayStatus { status } => {
                    if status.is_disconnected() {
                        return Err(Error::NotConnected);
                    }
                }
                RelayNotification::Shutdown => {
                    return Err(Error::ReceivedShutdown);
                }
                _ => (),
            };

            if sync_done
                && have_ids.is_empty()
                && need_ids.is_empty()
                && in_flight_up.is_empty()
                && !in_flight_down
            {
                break;
            }
        }

        tracing::info!(url = %self.url, "Negentropy reconciliation terminated.");

        Ok(())
    }

    /// Deprecated negentropy protocol
    #[inline(never)]
    pub(super) async fn sync_deprecated(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: &SyncOptions,
        output: &mut Reconciliation,
    ) -> Result<(), Error> {
        // Compose negentropy struct, add items and seal
        let mut negentropy = NegentropyDeprecated::new(32, Some(NEGENTROPY_FRAME_SIZE_LIMIT))?;
        for (id, timestamp) in items.into_iter() {
            let id = BytesDeprecated::from_slice(id.as_bytes());
            negentropy.add_item(timestamp.as_u64(), id)?;
        }
        negentropy.seal()?;

        // Initiate message
        let initial_message = negentropy.initiate()?;

        // Subscribe to notifications
        let mut notifications = self.internal_notification_sender.subscribe();
        let mut temp_notifications = self.internal_notification_sender.subscribe();

        // Send initial negentropy message
        let sub_id = SubscriptionId::generate();
        let open_msg = ClientMessage::NegOpen {
            subscription_id: sub_id.clone(),
            filter: Box::new(filter),
            id_size: Some(32),
            initial_message: hex::encode(initial_message),
        };
        self.send_msg(open_msg)?;

        // Check if negentropy is supported
        check_negentropy_support(&sub_id, opts, &mut temp_notifications).await?;

        let mut in_flight_up: HashSet<EventId> = HashSet::new();
        let mut in_flight_down: bool = false;
        let mut sync_done: bool = false;
        let mut have_ids: Vec<EventId> = Vec::new();
        let mut need_ids: Vec<EventId> = Vec::new();
        let down_sub_id: SubscriptionId = SubscriptionId::generate();

        // Start reconciliation
        while let Ok(notification) = notifications.recv().await {
            match notification {
                RelayNotification::Message { message } => {
                    match message {
                        RelayMessage::NegMsg {
                            subscription_id,
                            message,
                        } => {
                            if subscription_id == sub_id {
                                let mut curr_have_ids: Vec<BytesDeprecated> = Vec::new();
                                let mut curr_need_ids: Vec<BytesDeprecated> = Vec::new();

                                // Parse message
                                let query: BytesDeprecated = BytesDeprecated::from_hex(message)?;

                                // Reconcile
                                let msg: Option<BytesDeprecated> = negentropy.reconcile_with_ids(
                                    &query,
                                    &mut curr_have_ids,
                                    &mut curr_need_ids,
                                )?;

                                // Handle message
                                self.handle_neg_msg(
                                    subscription_id,
                                    msg.map(|m| m.to_bytes()),
                                    curr_have_ids.into_iter().filter_map(neg_depr_to_event_id),
                                    curr_need_ids.into_iter().filter_map(neg_depr_to_event_id),
                                    opts,
                                    output,
                                    &mut have_ids,
                                    &mut need_ids,
                                    &mut sync_done,
                                )?;
                            }
                        }
                        RelayMessage::NegErr {
                            subscription_id,
                            message,
                        } => {
                            if subscription_id == sub_id {
                                return Err(Error::RelayMessage(message));
                            }
                        }
                        RelayMessage::Ok {
                            event_id,
                            status,
                            message,
                        } => {
                            self.handle_neg_ok(
                                &mut in_flight_up,
                                event_id,
                                status,
                                message,
                                output,
                            );
                        }
                        RelayMessage::Event {
                            subscription_id,
                            event,
                        } => {
                            if subscription_id == down_sub_id {
                                output.received.insert(event.id);
                            }
                        }
                        RelayMessage::EndOfStoredEvents(id) => {
                            if id == down_sub_id {
                                in_flight_down = false;
                            }
                        }
                        _ => (),
                    }

                    // Send events
                    self.upload_neg_events(&mut have_ids, &mut in_flight_up, opts)
                        .await?;

                    // Get events
                    self.req_neg_events(&mut need_ids, &mut in_flight_down, &down_sub_id, opts)?;
                }
                RelayNotification::RelayStatus { status } => {
                    if status.is_disconnected() {
                        return Err(Error::NotConnected);
                    }
                }
                RelayNotification::Shutdown => return Err(Error::ReceivedShutdown),
                _ => (),
            };

            if sync_done
                && have_ids.is_empty()
                && need_ids.is_empty()
                && in_flight_up.is_empty()
                && !in_flight_down
            {
                break;
            }
        }

        tracing::info!(url = %self.url, "Deprecated negentropy reconciliation terminated.");

        Ok(())
    }
}

/// Send WebSocket messages with timeout set to [WEBSOCKET_TX_TIMEOUT].
async fn send_ws_msgs(tx: &mut BoxSink, msgs: Vec<Message>) -> Result<(), Error> {
    let mut stream = futures_util::stream::iter(msgs.into_iter().map(Ok));
    match time::timeout(Some(WEBSOCKET_TX_TIMEOUT), tx.send_all(&mut stream)).await {
        Some(res) => Ok(res?),
        None => Err(Error::Timeout),
    }
}

/// Send WebSocket messages with timeout set to [WEBSOCKET_TX_TIMEOUT].
async fn close_ws(tx: &mut BoxSink) -> Result<(), Error> {
    // TODO: remove timeout from here?
    match time::timeout(Some(WEBSOCKET_TX_TIMEOUT), tx.close()).await {
        Some(res) => Ok(res?),
        None => Err(Error::Timeout),
    }
}

#[inline]
fn neg_id_to_event_id(id: Id) -> EventId {
    EventId::from_byte_array(id.to_bytes())
}

#[inline]
fn neg_depr_to_event_id(id: BytesDeprecated) -> Option<EventId> {
    EventId::from_slice(id.as_bytes()).ok()
}

fn prepare_negentropy_storage(
    items: Vec<(EventId, Timestamp)>,
) -> Result<NegentropyStorageVector, Error> {
    // Compose negentropy storage
    let mut storage = NegentropyStorageVector::with_capacity(items.len());

    // Add items
    for (id, timestamp) in items.into_iter() {
        let id: Id = Id::from_byte_array(id.to_bytes());
        storage.insert(timestamp.as_u64(), id)?;
    }

    // Seal
    storage.seal()?;

    // Build negentropy client
    Ok(storage)
}

/// Check if negentropy is supported
#[inline(never)]
async fn check_negentropy_support(
    sub_id: &SubscriptionId,
    opts: &SyncOptions,
    temp_notifications: &mut broadcast::Receiver<RelayNotification>,
) -> Result<(), Error> {
    time::timeout(Some(opts.initial_timeout), async {
        while let Ok(notification) = temp_notifications.recv().await {
            if let RelayNotification::Message { message } = notification {
                match message {
                    RelayMessage::NegMsg {
                        subscription_id, ..
                    } => {
                        if &subscription_id == sub_id {
                            break;
                        }
                    }
                    RelayMessage::NegErr {
                        subscription_id,
                        message,
                    } => {
                        if &subscription_id == sub_id {
                            return Err(Error::RelayMessage(message));
                        }
                    }
                    RelayMessage::Notice(message) => {
                        if message == "ERROR: negentropy error: negentropy query missing elements" {
                            // The NEG-OPEN message is send with 4 elements instead of 5
                            // If the relay return this error means that is not support new
                            // negentropy protocol
                            return Err(Error::Negentropy(
                                negentropy::Error::UnsupportedProtocolVersion,
                            ));
                        } else if message.contains("bad msg")
                            && (message.contains("unknown cmd")
                                || message.contains("negentropy")
                                || message.contains("NEG-"))
                        {
                            return Err(Error::NegentropyNotSupported);
                        } else if message.contains("bad msg: invalid message")
                            && message.contains("NEG-OPEN")
                        {
                            return Err(Error::UnknownNegentropyError);
                        }
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    })
    .await
    .ok_or(Error::Timeout)?
}
