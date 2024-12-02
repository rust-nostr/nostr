// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::cmp;
use std::collections::{HashMap, HashSet};
#[cfg(feature = "nip11")]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::{task, time};
use async_wsocket::futures_util::{self, Future, SinkExt, StreamExt};
use async_wsocket::{connect as wsocket_connect, ConnectionMode, Sink, Stream, WsMessage};
use atomic_destructor::AtomicDestroyer;
use negentropy::{Bytes, Id, Negentropy, NegentropyStorageVector};
use negentropy_deprecated::{Bytes as BytesDeprecated, Negentropy as NegentropyDeprecated};
#[cfg(not(target_arch = "wasm32"))]
use nostr::secp256k1::rand;
use nostr_database::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, watch, Mutex, MutexGuard, OnceCell, RwLock};

use super::constants::{
    BATCH_EVENT_ITERATION_TIMEOUT, MAX_RETRY_INTERVAL, MIN_ATTEMPTS, MIN_SUCCESS_RATE,
    NEGENTROPY_BATCH_SIZE_DOWN, NEGENTROPY_FRAME_SIZE_LIMIT, NEGENTROPY_HIGH_WATER_UP,
    NEGENTROPY_LOW_WATER_UP, PING_INTERVAL, WEBSOCKET_TX_TIMEOUT,
};
use super::filtering::{CheckFiltering, RelayFiltering};
use super::flags::AtomicRelayServiceFlags;
use super::options::{
    FilterOptions, RelayOptions, SubscribeAutoCloseOptions, SubscribeOptions, SyncOptions,
};
use super::ping::PingTracker;
use super::stats::RelayConnectionStats;
use super::{Error, Reconciliation, RelayNotification, RelayStatus};
use crate::pool::RelayPoolNotification;
use crate::relay::status::AtomicRelayStatus;

struct NostrMessage {
    msgs: Vec<ClientMessage>,
}

#[derive(Debug, Clone, Copy)]
enum RelayServiceEvent {
    /// None
    None,
    /// Completely disconnect
    Terminate,
}

#[derive(Debug)]
struct RelayChannels {
    nostr: (Sender<NostrMessage>, Mutex<Receiver<NostrMessage>>),
    ping: (watch::Sender<u64>, Mutex<watch::Receiver<u64>>),
    service: (
        watch::Sender<RelayServiceEvent>,
        Mutex<watch::Receiver<RelayServiceEvent>>,
    ),
}

impl RelayChannels {
    pub fn new() -> Self {
        let (tx_nostr, rx_nostr) = mpsc::channel::<NostrMessage>(1024);
        let (tx_ping, rx_ping) = watch::channel::<u64>(0);
        let (tx_service, rx_service) = watch::channel::<RelayServiceEvent>(RelayServiceEvent::None);

        Self {
            nostr: (tx_nostr, Mutex::new(rx_nostr)),
            ping: (tx_ping, Mutex::new(rx_ping)),
            service: (tx_service, Mutex::new(rx_service)),
        }
    }

    pub fn send_nostr_msg(&self, msg: NostrMessage) -> Result<(), Error> {
        self.nostr
            .0
            .try_send(msg)
            .map_err(|_| Error::CantSendChannelMessage {
                channel: String::from("nostr"),
            })
    }

    pub async fn rx_nostr(&self) -> MutexGuard<'_, Receiver<NostrMessage>> {
        self.nostr.1.lock().await
    }

    pub fn nostr_queue(&self) -> usize {
        self.nostr.0.max_capacity() - self.nostr.0.capacity()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn ping(&self, nonce: u64) -> Result<(), Error> {
        self.ping
            .0
            .send(nonce)
            .map_err(|_| Error::CantSendChannelMessage {
                channel: String::from("ping"),
            })
    }

    pub async fn rx_ping(&self) -> MutexGuard<'_, watch::Receiver<u64>> {
        self.ping.1.lock().await
    }

    pub async fn rx_service(&self) -> MutexGuard<'_, watch::Receiver<RelayServiceEvent>> {
        self.service.1.lock().await
    }

    pub fn send_service_msg(&self, event: RelayServiceEvent) -> Result<(), Error> {
        self.service
            .0
            .send(event)
            .map_err(|_| Error::CantSendChannelMessage {
                channel: String::from("service"),
            })
    }
}

#[derive(Debug, Clone)]
struct SubscriptionData {
    pub filters: Vec<Filter>,
    pub subscribed_at: Timestamp,
    /// Subscription closed by relay
    pub closed: bool,
}

impl Default for SubscriptionData {
    fn default() -> Self {
        Self {
            filters: Vec::new(),
            subscribed_at: Timestamp::zero(),
            closed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InnerRelay {
    pub(super) url: RelayUrl,
    status: Arc<AtomicRelayStatus>,
    #[cfg(feature = "nip11")]
    document: Arc<RwLock<RelayInformationDocument>>,
    #[cfg(feature = "nip11")]
    last_document_fetch: Arc<AtomicU64>,
    pub(super) opts: RelayOptions,
    pub(super) flags: AtomicRelayServiceFlags,
    pub(super) stats: RelayConnectionStats,
    pub(super) filtering: RelayFiltering,
    database: Arc<dyn NostrDatabase>,
    channels: Arc<RelayChannels>,
    pub(super) internal_notification_sender: broadcast::Sender<RelayNotification>,
    external_notification_sender: OnceCell<broadcast::Sender<RelayPoolNotification>>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, SubscriptionData>>>,
    running: Arc<AtomicBool>,
}

impl AtomicDestroyer for InnerRelay {
    fn on_destroy(&self) {
        if let Err(e) = self.disconnect() {
            tracing::error!("Impossible to shutdown '{}': {e}", self.url);
        }
    }
}

impl InnerRelay {
    pub fn new(
        url: RelayUrl,
        database: Arc<dyn NostrDatabase>,
        filtering: RelayFiltering,
        opts: RelayOptions,
    ) -> Self {
        let (relay_notification_sender, ..) = broadcast::channel::<RelayNotification>(2048);

        Self {
            url,
            status: Arc::new(AtomicRelayStatus::default()),
            #[cfg(feature = "nip11")]
            document: Arc::new(RwLock::new(RelayInformationDocument::new())),
            #[cfg(feature = "nip11")]
            last_document_fetch: Arc::new(AtomicU64::new(0)),
            flags: AtomicRelayServiceFlags::new(opts.flags),
            opts,
            stats: RelayConnectionStats::default(),
            filtering,
            database,
            channels: Arc::new(RelayChannels::new()),
            internal_notification_sender: relay_notification_sender,
            external_notification_sender: OnceCell::new(),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    #[inline]
    pub fn connection_mode(&self) -> &ConnectionMode {
        &self.opts.connection_mode
    }

    /// Is connection task running?
    #[inline]
    pub(super) fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn status(&self) -> RelayStatus {
        self.status.load()
    }

    fn set_status(&self, status: RelayStatus, log: bool) {
        // Change status
        self.status.set(status);

        // Log
        if log {
            match status {
                RelayStatus::Initialized => tracing::trace!("'{}' initialized.", self.url),
                RelayStatus::Pending => tracing::trace!("'{}' is pending.", self.url),
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

    #[inline]
    pub fn is_connected(&self) -> bool {
        self.status().is_connected()
    }

    /// Perform health checks
    fn health_check(&self) -> Result<(), Error> {
        let status: RelayStatus = self.status();

        // Relay initialized (never called connect method)
        if status.is_initialized() {
            return Err(Error::Initialized);
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
    pub async fn document(&self) -> RelayInformationDocument {
        let document = self.document.read().await;
        document.clone()
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
            if self.last_document_fetch.load(Ordering::SeqCst) + 3600 < now {
                // Update last fetch
                self.last_document_fetch.store(now, Ordering::SeqCst);

                // Fetch
                let url = self.url.clone();
                let d = self.document.clone();
                task::spawn(async move {
                    match RelayInformationDocument::get(url.clone().into(), proxy).await {
                        Ok(document) => {
                            let mut d = d.write().await;
                            *d = document
                        }
                        Err(e) => {
                            tracing::warn!("Can't get information document from '{url}': {e}")
                        }
                    };
                });
            }
        }
    }

    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        let subscription = self.subscriptions.read().await;
        subscription
            .iter()
            .map(|(k, v)| (k.clone(), v.filters.clone()))
            .collect()
    }

    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        let subscription = self.subscriptions.read().await;
        subscription.get(id).map(|d| d.filters.clone())
    }

    pub(crate) async fn update_subscription(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        update_subscribed_at: bool,
    ) {
        let mut subscriptions = self.subscriptions.write().await;
        let data: &mut SubscriptionData = subscriptions.entry(id).or_default();
        data.filters = filters;

        if update_subscribed_at {
            data.subscribed_at = Timestamp::now();
        }
    }

    /// Mark subscription as closed
    async fn subscription_closed(&self, id: &SubscriptionId) {
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(data) = subscriptions.get_mut(id) {
            data.closed = true;
        }
    }

    /// Check if it should subscribe for current websocket session
    pub(crate) async fn should_resubscribe(&self, id: &SubscriptionId) -> bool {
        let subscriptions = self.subscriptions.read().await;
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
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(id);
    }

    #[inline]
    pub fn queue(&self) -> usize {
        self.channels.nostr_queue()
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
                    RelayNotification::Authenticated => {
                        Some(RelayPoolNotification::Authenticated {
                            relay_url: self.url.clone(),
                        })
                    }
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

    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        // Return if relay can't connect
        if !self.status().can_connect() {
            return;
        }

        // Update status
        // Change it to pending to avoid issues with the health check (initialized check)
        self.set_status(RelayStatus::Pending, false);

        // If connection timeout is `Some`, try to connect waiting for connection
        match connection_timeout {
            Some(timeout) => {
                let mut notifications = self.internal_notification_sender.subscribe();

                // Spawn and try connect
                self.spawn_and_try_connect(timeout);

                // Wait for status change (connected or disconnected)
                tracing::debug!(
                    "Waiting for status change for '{}' relay before continue",
                    self.url
                );
                while let Ok(notification) = notifications.recv().await {
                    if let RelayNotification::RelayStatus {
                        status: RelayStatus::Connected | RelayStatus::Disconnected,
                    } = notification
                    {
                        break;
                    }
                }
            }
            None => {
                self.spawn_and_try_connect(Duration::from_secs(60));
            }
        }
    }

    fn spawn_and_try_connect(&self, connection_timeout: Duration) {
        if self.is_running() {
            tracing::warn!(url = %self.url, "Connection task is already running.");
            return;
        }

        let relay = self.clone();
        task::spawn(async move {
            // Set that connection task is running
            relay.running.store(true, Ordering::SeqCst);

            // Auto-connect loop
            loop {
                // TODO: check in the relays state database if relay can connect (different from the previous check)
                // TODO: if the relay score is too low, immediately exit.
                // TODO: at every loop iteration check the score and if it's too low, exit

                // Acquire service watcher
                let mut rx_service = relay.channels.rx_service().await;

                tokio::select! {
                    // Connect and run message handler
                    _ = relay.connect_and_run(connection_timeout) => {},
                    // Handle terminate
                    _ = relay.handle_terminate(&mut rx_service) => {
                        // Update status
                        relay.set_status(RelayStatus::Terminated, true);

                        // Break loop
                        break;
                    }
                }

                // Get status
                let status: RelayStatus = relay.status();

                // If status is set to terminated, break loop.
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

                    tokio::select! {
                        // Sleep
                        _ = time::sleep(interval) => {},
                        // Handle terminate
                        _ = relay.handle_terminate(&mut rx_service) => {
                            // Update status
                            relay.set_status(RelayStatus::Terminated, true);
                            break;
                        }
                    }
                } else {
                    // Reconnection disabled, set status to terminated
                    relay.set_status(RelayStatus::Terminated, true);

                    // Break loop and exit
                    tracing::debug!(url = %relay.url, "Reconnection disabled, breaking loop.");
                    break;
                }
            }

            // Set that connection task is no longer running
            relay.running.store(false, Ordering::SeqCst);

            tracing::debug!(url = %relay.url, "Auto connect loop terminated.");
        });
    }

    /// Depending on attempts and success, use default or incremental retry interval
    fn calculate_retry_interval(&self) -> Duration {
        // Check if incremental interval is enabled
        if self.opts.adjust_retry_interval {
            // Calculate difference between attempts and success
            // diff = attempts - success
            let diff: u32 = self.stats.attempts().saturating_sub(self.stats.success()) as u32;

            // Diff must be at least 2
            if diff >= 2 {
                // Calculate multiplier
                let multiplier: u32 = diff / 2;

                // Calculate interval
                let interval: Duration = self.opts.retry_interval * multiplier;

                // If interval is too big, use the max one.
                return cmp::min(interval, MAX_RETRY_INTERVAL);
            }
        }

        // Use default internal
        self.opts.retry_interval
    }

    async fn handle_terminate(&self, rx_service: &mut watch::Receiver<RelayServiceEvent>) {
        loop {
            if rx_service.changed().await.is_ok() {
                // Get service and mark as seen
                match *rx_service.borrow_and_update() {
                    // Do nothing
                    RelayServiceEvent::None => {}
                    // Terminate
                    RelayServiceEvent::Terminate => break,
                }
            }
        }
    }

    /// Connect and run message handler
    async fn connect_and_run(&self, connection_timeout: Duration) {
        // Update status
        self.set_status(RelayStatus::Connecting, true);

        // Add attempt
        self.stats.new_attempt();

        // Compose timeout
        let timeout: Duration = if self.stats.attempts() > 1 {
            // Many attempts, use the default timeout
            #[cfg(feature = "tor")]
            if let ConnectionMode::Tor { .. } = &self.opts.connection_mode {
                Duration::from_secs(120)
            } else {
                Duration::from_secs(60)
            }

            #[cfg(not(feature = "tor"))]
            Duration::from_secs(60)
        } else {
            // First attempt, use external timeout
            connection_timeout
        };

        // Connect
        match wsocket_connect((&self.url).into(), &self.opts.connection_mode, timeout).await {
            Ok((ws_tx, ws_rx)) => {
                // Update status
                self.set_status(RelayStatus::Connected, true);

                // Increment success stats
                self.stats.new_success();

                // Request information document
                #[cfg(feature = "nip11")]
                self.request_nip11_document();

                // Run message handler
                self.run_message_handler(ws_tx, ws_rx).await;
            }
            Err(e) => {
                // Update status
                self.set_status(RelayStatus::Disconnected, false);

                // Log error
                tracing::error!("Impossible to connect to '{}': {e}", self.url);
            }
        }
    }

    async fn run_message_handler(&self, ws_tx: Sink, ws_rx: Stream) {
        // (Re)subscribe to relay
        if self.flags.can_read() {
            if let Err(e) = self.resubscribe().await {
                tracing::error!("Impossible to subscribe to '{}': {e}", self.url)
            }
        }

        let ping: PingTracker = PingTracker::default();

        // Wait that one of the futures terminate/complete
        tokio::select! {
            _ = self.receiver_message_handler(ws_rx, &ping) => {
                tracing::trace!(url = %self.url, "Relay received exited.");
            },
            res = self.sender_message_handler(ws_tx, &ping) => match res {
                Ok(()) => tracing::trace!(url = %self.url, "Relay sender exited."),
                Err(e) => tracing::error!(url = %self.url, error = %e, "Relay sender exited with error.")
            },
            res = self.ping_handler(&ping) => match res {
                Ok(()) => tracing::trace!(url = %self.url, "Relay pinger exited."),
                Err(e) => tracing::error!(url = %self.url, error = %e, "Relay pinger exited with error.")
            }
        }
    }

    async fn sender_message_handler(
        &self,
        mut ws_tx: Sink,
        ping: &PingTracker,
    ) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        let _ping = ping;

        // Lock receivers
        let mut rx_nostr = self.channels.rx_nostr().await;
        let mut rx_ping = self.channels.rx_ping().await;

        loop {
            tokio::select! {
                // Nostr channel receiver
                Some(NostrMessage { msgs }) = rx_nostr.recv() => {
                    // Serialize messages to JSON and compose WebSocket text message
                    let msgs: Vec<WsMessage> = msgs
                        .into_iter()
                        .map(|msg| WsMessage::Text(msg.as_json()))
                        .collect();

                    // Calculate messages size
                    let size: usize = msgs.iter().map(|msg| msg.len()).sum();
                    let len: usize = msgs.len();

                    // Compose log msg without prefix ("Sending" or "Sent")
                    let partial_log_msg: String = if len == 1 {
                        let json = &msgs[0]; // SAFETY: len checked above (len == 1)
                        format!("'{json}' to '{}'", self.url)
                    } else {
                        format!("{len} messages to '{}'", self.url)
                    };

                    tracing::trace!("Sending {partial_log_msg} (size: {size} bytes)");

                    // Send WebSocket messages
                    send_ws_msgs(&mut ws_tx, msgs).await?;

                    // Increase sent bytes
                    self.stats.add_bytes_sent(size);

                    tracing::debug!("Sent {partial_log_msg} (size: {size} bytes)");
                }
                // Ping channel receiver
                Ok(()) = rx_ping.changed() => {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                         // Get nonce and mark as seen
                        let nonce: u64 = *rx_ping.borrow_and_update();

                        // Compose ping message
                        let msg = WsMessage::Ping(nonce.to_be_bytes().to_vec());

                        // Send WebSocket message
                        send_ws_msgs(&mut ws_tx, [msg]).await?;

                        // Set ping as just sent
                        ping.just_sent().await;

                        tracing::debug!("Ping '{}' (nonce: {nonce})", self.url);
                    }
                }
                else => break
            }
        }

        // Close WebSocket
        close_ws(&mut ws_tx).await
    }

    async fn receiver_message_handler(&self, mut ws_rx: Stream, ping: &PingTracker) {
        #[cfg(target_arch = "wasm32")]
        let _ping = ping;

        while let Some(msg) = ws_rx.next().await {
            if let Ok(msg) = msg {
                match msg {
                    #[cfg(not(target_arch = "wasm32"))]
                    WsMessage::Pong(bytes) => {
                        if self.flags.has_ping() {
                            match bytes.try_into() {
                                Ok(nonce) => {
                                    // Nonce from big-endian bytes
                                    let nonce: u64 = u64::from_be_bytes(nonce);

                                    // Get last nonce
                                    let last_nonce: u64 = ping.last_nonce();

                                    // Check if last nonce not match the current one
                                    if last_nonce != nonce {
                                        tracing::error!("Pong nonce not match: received={nonce}, expected={last_nonce}");
                                        break;
                                    }

                                    tracing::debug!(
                                        "Pong from '{}' match nonce: {nonce}",
                                        self.url
                                    );

                                    // Set ping as replied
                                    ping.set_replied(true);

                                    // Save latency
                                    let sent_at = ping.sent_at().await;
                                    self.stats.save_latency(sent_at.elapsed());
                                }
                                Err(e) => {
                                    tracing::error!("Can't parse pong nonce: {e:?}");
                                    break;
                                }
                            }
                        }
                    }
                    _ => {
                        let data: Vec<u8> = msg.into_data();
                        self.handle_relay_message_infallible(&data).await;
                    }
                }
            }
        }
    }

    async fn ping_handler(&self, ping: &PingTracker) -> Result<(), Error> {
        #[cfg(not(target_arch = "wasm32"))]
        if self.flags.has_ping() {
            loop {
                // If last nonce is NOT 0, check if relay replied
                // Return error if relay not replied
                if ping.last_nonce() != 0 && !ping.replied() {
                    // Reset ping status
                    ping.reset();

                    // Return error
                    return Err(Error::NotRepliedToPing);
                }

                // Generate and save nonce
                let nonce: u64 = rand::random();
                ping.set_last_nonce(nonce);
                ping.set_replied(false);

                // Try to ping
                self.channels.ping(nonce)?;

                // Sleep
                time::sleep(PING_INTERVAL).await;
            }
        } else {
            loop {
                time::sleep(PING_INTERVAL).await;
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ping = ping;
            loop {
                time::sleep(PING_INTERVAL).await;
            }
        }
    }

    async fn handle_relay_message_infallible(&self, msg: &[u8]) {
        match self.handle_relay_message(msg).await {
            Ok(Some(message)) => {
                match &message {
                    RelayMessage::Notice { message } => {
                        tracing::warn!("Notice from '{}': {message}", self.url)
                    }
                    RelayMessage::Ok {
                        event_id,
                        status,
                        message,
                    } => {
                        tracing::debug!("Received OK from '{}' for event {event_id}: status={status}, message={message}", self.url);
                    }
                    RelayMessage::Closed {
                        subscription_id, ..
                    } => {
                        tracing::debug!(
                            "Subscription '{subscription_id}' closed by '{}'",
                            self.url
                        );
                        self.subscription_closed(subscription_id).await;
                    }
                    RelayMessage::Auth { challenge } => {
                        tracing::debug!(
                            "Received '{challenge}' authentication challenge from '{}'",
                            self.url
                        );
                    }
                    _ => (),
                }

                // Send notification
                self.send_notification(RelayNotification::Message { message }, true);
            }
            Ok(None) | Err(Error::MessageHandle(MessageHandleError::EmptyMsg)) => (),
            Err(e) => tracing::warn!(
                "Impossible to handle relay message from '{}': {e}",
                self.url
            ),
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn handle_relay_message(&self, msg: &[u8]) -> Result<Option<RelayMessage>, Error> {
        let size: usize = msg.len();

        // Update bytes received
        self.stats.add_bytes_received(size);

        // Check message size
        if let Some(max_size) = self.opts.limits.messages.max_size {
            let max_size: usize = max_size as usize;
            if size > max_size {
                return Err(Error::RelayMessageTooLarge { size, max_size });
            }
        }

        // Deserialize message
        let msg = RawRelayMessage::from_json(msg)?;
        tracing::trace!("Received message from '{}': {:?}", self.url, msg);

        // Handle msg
        match msg {
            RawRelayMessage::Event {
                subscription_id,
                event,
            } => {
                let kind: Kind = Kind::from(event.kind);

                // Check event size
                if let Some(max_size) = self.opts.limits.events.get_max_size(&kind) {
                    let size: usize = event.as_json().as_bytes().len();
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
                match self.filtering.check_partial_event(&partial_event).await {
                    CheckFiltering::Allow => {
                        // Nothing to do
                    }
                    CheckFiltering::EventIdBlacklisted(id) => {
                        tracing::debug!("Received event with blacklisted ID: {id}");
                        return Ok(None);
                    }
                    CheckFiltering::PublicKeyBlacklisted(pubkey) => {
                        tracing::debug!(
                            "Received event authored by blacklisted public key: {pubkey}"
                        );
                        return Ok(None);
                    }
                    CheckFiltering::PublicKeyNotInWhitelist(pubkey) => {
                        tracing::debug!(
                            "Received event authored by non-whitelisted public key: {pubkey}"
                        );
                        return Ok(None);
                    }
                }

                // Check min POW
                let difficulty: u8 = self.opts.get_pow_difficulty();
                if difficulty > 0 && !partial_event.id.check_pow(difficulty) {
                    return Err(Error::PowDifficultyTooLow { min: difficulty });
                }

                // TODO: check if word/hashtag is blacklisted

                // Check if event status
                let status: DatabaseEventStatus = self.database.check_id(&partial_event.id).await?;

                // Event deleted
                if let DatabaseEventStatus::Deleted = status {
                    return Ok(None);
                }

                // Deserialize missing fields
                let missing: MissingPartialEvent = MissingPartialEvent::from_raw(event)?;

                // Check if event is replaceable and has coordinate
                if missing.kind.is_replaceable() || missing.kind.is_parameterized_replaceable() {
                    let coordinate: Coordinate =
                        Coordinate::new(missing.kind, partial_event.pubkey)
                            .identifier(missing.identifier().unwrap_or_default());

                    // Check if coordinate has been deleted
                    if self
                        .database
                        .has_coordinate_been_deleted(&coordinate, &missing.created_at)
                        .await?
                    {
                        return Ok(None);
                    }
                }

                // Set event as seen by relay
                if let Err(e) = self
                    .database
                    .event_id_seen(partial_event.id, self.url.clone())
                    .await
                {
                    tracing::error!(
                        "Impossible to set event {} as seen by relay: {e}",
                        partial_event.id
                    );
                }

                // Compose full event
                let event: Event = partial_event.merge(missing)?;

                // Check if it's expired
                if event.is_expired() {
                    return Err(Error::EventExpired);
                }

                if let DatabaseEventStatus::NotExistent = status {
                    // Verify event
                    event.verify()?;

                    // Save into database
                    self.database.save_event(&event).await?;

                    // Send notification
                    self.send_notification(
                        RelayNotification::Event {
                            subscription_id: SubscriptionId::new(&subscription_id),
                            event: Box::new(event.clone()),
                        },
                        true,
                    );
                }

                Ok(Some(RelayMessage::Event {
                    subscription_id: SubscriptionId::new(subscription_id),
                    event: Box::new(event),
                }))
            }
            m => Ok(Some(RelayMessage::try_from(m)?)),
        }
    }

    pub fn disconnect(&self) -> Result<(), Error> {
        // Check if it's NOT terminated
        if !self.status().is_terminated() {
            self.channels
                .send_service_msg(RelayServiceEvent::Terminate)?;
            self.send_notification(RelayNotification::Shutdown, false);
        }

        Ok(())
    }

    #[inline]
    pub fn send_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        self.batch_msg(vec![msg])
    }

    #[tracing::instrument(skip_all, level = "trace")]
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
        self.channels.send_nostr_msg(NostrMessage { msgs })
    }

    #[inline]
    fn send_neg_msg(&self, id: SubscriptionId, message: String) -> Result<(), Error> {
        self.send_msg(ClientMessage::NegMsg {
            subscription_id: id,
            message,
        })
    }

    #[inline]
    fn send_neg_close(&self, id: SubscriptionId) -> Result<(), Error> {
        self.send_msg(ClientMessage::NegClose {
            subscription_id: id,
        })
    }

    #[inline]
    pub async fn send_event(&self, event: Event) -> Result<EventId, Error> {
        // Health, write permission and number of messages checks are executed in `batch_msg` method.

        // Subscribe to notifications
        let mut notifications = self.internal_notification_sender.subscribe();

        // Send message
        self.send_msg(ClientMessage::event(event))?;

        // Wait for OK
        let (event_id, status, message) = self
            .wait_for_ok(&mut notifications, None, BATCH_EVENT_ITERATION_TIMEOUT)
            .await?;

        if status {
            Ok(event_id)
        } else {
            Err(Error::RelayMessage { message })
        }
    }

    pub async fn auth(&self, event: Event) -> Result<(), Error> {
        // Check if NIP42 event
        if event.kind != Kind::Authentication {
            return Err(Error::UnexpectedKind {
                expected: Kind::Authentication,
                found: event.kind,
            });
        }

        let mut notifications = self.internal_notification_sender.subscribe();

        // Send message
        let id: EventId = event.id;
        self.send_msg(ClientMessage::auth(event))?;

        // Wait for OK
        // The event ID is already checked in `wait_for_ok` method
        let (_, status, message) = self
            .wait_for_ok(&mut notifications, Some(id), BATCH_EVENT_ITERATION_TIMEOUT)
            .await?;

        // Check status
        if status {
            // Send notification
            self.send_notification(RelayNotification::Authenticated, true);
            Ok(())
        } else {
            Err(Error::RelayMessage { message })
        }
    }

    async fn wait_for_ok(
        &self,
        notifications: &mut broadcast::Receiver<RelayNotification>,
        id: Option<EventId>,
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
                        let can_return: bool = match id {
                            // It's specified an ID, check if match the received one.
                            Some(id) => id == event_id,
                            // Nothing to check, return.
                            None => true,
                        };

                        if can_return {
                            return Ok((event_id, status, message));
                        }
                    }
                    RelayNotification::RelayStatus { status } => {
                        if status.is_disconnected() {
                            // TODO: use another error?
                            return Err(Error::EventNotPublished(String::from(
                                "relay not connected (status changed)",
                            )));
                        }
                    }
                    RelayNotification::Shutdown => break,
                    _ => (),
                }
            }

            Err(Error::EventNotPublished(String::from(
                "loop prematurely terminated",
            )))
        })
        .await
        .ok_or(Error::Timeout)?
    }

    pub async fn resubscribe(&self) -> Result<(), Error> {
        let subscriptions = self.subscriptions().await;
        for (id, filters) in subscriptions.into_iter() {
            if !filters.is_empty() && self.should_resubscribe(&id).await {
                self.send_msg(ClientMessage::req(id, filters))?;
            } else {
                tracing::debug!("Skip re-subscription of '{id}'");
            }
        }

        Ok(())
    }

    #[inline]
    pub async fn subscribe(
        &self,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<SubscriptionId, Error> {
        let id: SubscriptionId = SubscriptionId::generate();
        self.subscribe_with_id(id.clone(), filters, opts).await?;
        Ok(id)
    }

    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<(), Error> {
        // Check if filters are empty
        if filters.is_empty() {
            return Err(Error::FiltersEmpty);
        }

        // Compose and send REQ message
        let msg: ClientMessage = ClientMessage::req(id.clone(), filters.clone());
        self.send_msg(msg)?;

        // Check if auto-close condition is set
        match opts.auto_close {
            Some(opts) => {
                let this = self.clone();
                task::spawn(async move {
                    let sub_id: SubscriptionId = id.clone();
                    let relay = this.clone();
                    let res: Option<bool> = time::timeout(opts.timeout, async move {
                        let mut counter = 0;
                        let mut received_eose: bool = false;

                        let mut notifications = relay.internal_notification_sender.subscribe();
                        while let Ok(notification) = notifications.recv().await {
                            match notification {
                                RelayNotification::Message { message, .. } => match message {
                                    RelayMessage::Event {
                                        subscription_id, ..
                                    } => {
                                        if subscription_id.eq(&id) {
                                            if let FilterOptions::WaitForEventsAfterEOSE(num) =
                                                opts.filter
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
                                        if subscription_id.eq(&id) {
                                            tracing::debug!(
                                                "Received EOSE for subscription {id} from {}",
                                                relay.url
                                            );
                                            received_eose = true;
                                            if let FilterOptions::ExitOnEOSE
                                            | FilterOptions::WaitDurationAfterEOSE(_) =
                                                opts.filter
                                            {
                                                break;
                                            }
                                        }
                                    }
                                    _ => (),
                                },
                                RelayNotification::RelayStatus { status } => {
                                    if status.is_disconnected() {
                                        return false; // No need to send CLOSE msg
                                    }
                                }
                                RelayNotification::Shutdown => {
                                    return false; // No need to send CLOSE msg
                                }
                                _ => (),
                            }
                        }

                        if let FilterOptions::WaitDurationAfterEOSE(duration) = opts.filter {
                            time::timeout(Some(duration), async {
                                while let Ok(notification) = notifications.recv().await {
                                    match notification {
                                        RelayNotification::RelayStatus { status } => {
                                            if status.is_disconnected() {
                                                return Ok(()); // No need to send CLOSE msg
                                            }
                                        }
                                        RelayNotification::Shutdown => {
                                            return Ok(()); // No need to send CLOSE msg
                                        }
                                        _ => (),
                                    }
                                }

                                Ok::<(), Error>(())
                            })
                            .await;
                        }

                        true // Need to send CLOSE msg
                    })
                    .await;

                    // Check if CLOSE needed
                    let to_close: bool = res.unwrap_or_else(|| {
                        tracing::warn!("Timeout reached for REQ {sub_id}, auto-closing.");
                        true
                    });

                    if to_close {
                        // Unsubscribe
                        this.send_msg(ClientMessage::close(sub_id.clone()))?;

                        tracing::debug!("Subscription {sub_id} auto-closed");
                    }

                    Ok::<(), Error>(())
                });
            }
            None => {
                // No auto-close subscription: update subscription filters
                self.update_subscription(id.clone(), filters, true).await;
            }
        };

        Ok(())
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
            // Remove subscription
            self.remove_subscription(&id).await;

            // Send CLOSE message
            self.send_msg(ClientMessage::close(id))?;
        }

        Ok(())
    }

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
        // Perform health checks
        self.health_check()?;

        // Compose options
        let auto_close_opts: SubscribeAutoCloseOptions = SubscribeAutoCloseOptions::default()
            .filter(opts)
            .timeout(Some(timeout));
        let subscribe_opts: SubscribeOptions =
            SubscribeOptions::default().close_on(Some(auto_close_opts));

        // Subscribe to channel
        let mut notifications = self.internal_notification_sender.subscribe();

        // Subscribe with auto-close
        let id: SubscriptionId = self.subscribe(filters, subscribe_opts).await?;

        let mut counter: u16 = 0;
        let mut received_eose: bool = false;

        time::timeout(Some(timeout), async {
            while let Ok(notification) = notifications.recv().await {
                match notification {
                    RelayNotification::Message { message, .. } => match message {
                        RelayMessage::Event {
                            subscription_id,
                            event,
                        } => {
                            if subscription_id.eq(&id) {
                                callback(*event).await;
                                if let FilterOptions::WaitForEventsAfterEOSE(num) = opts {
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
                            if subscription_id.eq(&id) {
                                tracing::debug!(
                                    "Received EOSE for subscription {id} from {}",
                                    self.url
                                );
                                received_eose = true;
                                if let FilterOptions::ExitOnEOSE
                                | FilterOptions::WaitDurationAfterEOSE(_) = opts
                                {
                                    break;
                                }
                            }
                        }
                        RelayMessage::Ok { .. } => (),
                        _ => (),
                    },
                    RelayNotification::RelayStatus { status } => {
                        if status.is_disconnected() {
                            return Err(Error::NotConnected);
                        }
                    }
                    RelayNotification::Shutdown => return Err(Error::Shutdown),
                    _ => (),
                }
            }

            Ok(())
        })
        .await
        .ok_or(Error::Timeout)??;

        if let FilterOptions::WaitDurationAfterEOSE(duration) = opts {
            time::timeout(Some(duration), async {
                while let Ok(notification) = notifications.recv().await {
                    match notification {
                        RelayNotification::Message {
                            message:
                                RelayMessage::Event {
                                    subscription_id,
                                    event,
                                },
                            ..
                        } => {
                            if subscription_id.eq(&id) {
                                callback(*event).await;
                            }
                        }
                        RelayNotification::RelayStatus { status } => {
                            if status.is_disconnected() {
                                return Err(Error::NotConnected);
                            }
                        }
                        RelayNotification::Shutdown => return Err(Error::Shutdown),
                        _ => (),
                    }
                }

                Ok(())
            })
            .await;
        }

        Ok(())
    }

    pub async fn fetch_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Events, Error> {
        let events: Mutex<Events> = Mutex::new(Events::new(&filters));
        self.fetch_events_with_callback(filters, timeout, opts, |event| async {
            let mut events = events.lock().await;
            events.insert(event);
        })
        .await?;
        Ok(events.into_inner())
    }

    pub async fn count_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<usize, Error> {
        let id = SubscriptionId::generate();
        self.send_msg(ClientMessage::count(id.clone(), filters))?;

        let mut count = 0;

        let mut notifications = self.internal_notification_sender.subscribe();
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
                    if subscription_id == id {
                        count = c;
                        break;
                    }
                }
            }
        })
        .await
        .ok_or(Error::Timeout)?;

        // Unsubscribe
        self.send_msg(ClientMessage::close(id))?;

        Ok(count)
    }

    pub async fn sync(&self, filter: Filter, opts: &SyncOptions) -> Result<Reconciliation, Error> {
        let items = self.database.negentropy_items(filter.clone()).await?;
        self.sync_with_items(filter, items, opts).await
    }

    pub async fn sync_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: &SyncOptions,
    ) -> Result<Reconciliation, Error> {
        // Compose map
        let mut map = HashMap::with_capacity(1);
        map.insert(filter, items);

        // Reconcile
        self.sync_multi(map, opts).await
    }

    pub async fn sync_multi(
        &self,
        map: HashMap<Filter, Vec<(EventId, Timestamp)>>,
        opts: &SyncOptions,
    ) -> Result<Reconciliation, Error> {
        // Perform health checks
        self.health_check()?;

        // Check if relay can read
        if !self.flags.can_read() {
            return Err(Error::ReadDisabled);
        }

        let mut output: Reconciliation = Reconciliation::default();

        for (filter, items) in map.into_iter() {
            match self
                .sync_new(filter.clone(), items.clone(), opts, &mut output)
                .await
            {
                Ok(..) => {}
                Err(e) => match e {
                    Error::NegentropyMaybeNotSupported
                    | Error::Negentropy(negentropy::Error::UnsupportedProtocolVersion) => {
                        tracing::warn!("Negentropy protocol '{}' (maybe) not supported, trying the deprecated one.", negentropy::PROTOCOL_VERSION);
                        self.sync_deprecated(filter, items, opts, &mut output)
                            .await?;
                    }
                    e => return Err(e),
                },
            }
        }

        Ok(output)
    }

    /// New negentropy protocol
    async fn sync_new(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: &SyncOptions,
        output: &mut Reconciliation,
    ) -> Result<(), Error> {
        // Compose negentropy storage, add items and seal
        let mut storage = NegentropyStorageVector::with_capacity(items.len());
        for (id, timestamp) in items.into_iter() {
            let id: Id = Id::new(id.to_bytes());
            storage.insert(timestamp.as_u64(), id)?;
        }
        storage.seal()?;

        let mut negentropy = Negentropy::new(storage, NEGENTROPY_FRAME_SIZE_LIMIT)?;

        // Send initial negentropy message
        let sub_id = SubscriptionId::generate();
        let open_msg = ClientMessage::neg_open(&mut negentropy, sub_id.clone(), filter)?;
        self.send_msg(open_msg)?;

        let mut notifications = self.internal_notification_sender.subscribe();
        let mut temp_notifications = self.internal_notification_sender.subscribe();

        // Check if negentropy it's supported
        time::timeout(Some(opts.initial_timeout), async {
            while let Ok(notification) = temp_notifications.recv().await {
                if let RelayNotification::Message { message } = notification {
                    match message {
                        RelayMessage::NegMsg {
                            subscription_id, ..
                        } => {
                            if subscription_id == sub_id {
                                break;
                            }
                        }
                        RelayMessage::NegErr {
                            subscription_id,
                            code,
                        } => {
                            if subscription_id == sub_id {
                                return Err(Error::NegentropyReconciliation(code));
                            }
                        }
                        RelayMessage::Notice { message } => {
                            if message
                                == "ERROR: negentropy error: negentropy query missing elements"
                            {
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
                                return Err(Error::NegentropyMaybeNotSupported);
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

            Ok::<(), Error>(())
        })
        .await
        .ok_or(Error::Timeout)??;

        let do_up: bool = opts.do_up();
        let do_down: bool = opts.do_down();
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
                                let query: Bytes = Bytes::from_hex(message)?;

                                // Reconcile
                                let msg: Option<Bytes> = negentropy.reconcile_with_ids(
                                    &query,
                                    &mut curr_have_ids,
                                    &mut curr_need_ids,
                                )?;

                                let mut counter: u64 = 0;

                                // If event ID wasn't already seen, add to the HAVE IDs
                                // Add to HAVE IDs only if `do_up` is true
                                for id in curr_have_ids.into_iter() {
                                    let event_id: EventId = EventId::from_byte_array(id.to_bytes());
                                    if output.local.insert(event_id) && do_up {
                                        have_ids.push(event_id);
                                        counter += 1;
                                    }
                                }

                                // If event ID wasn't already seen, add to the NEED IDs
                                // Add to NEED IDs only if `do_down` is true
                                for id in curr_need_ids.into_iter() {
                                    let event_id: EventId = EventId::from_byte_array(id.to_bytes());
                                    if output.remote.insert(event_id) && do_down {
                                        need_ids.push(event_id);
                                        counter += 1;
                                    }
                                }

                                if let Some(progress) = &opts.progress {
                                    progress.send_modify(|state| {
                                        state.total += counter;
                                    });
                                }

                                match msg {
                                    Some(query) => {
                                        tracing::debug!(
                                            "Continue negentropy reconciliation with '{}'",
                                            self.url
                                        );
                                        self.send_neg_msg(subscription_id, query.to_hex())?;
                                    }
                                    None => {
                                        // Mark sync as done
                                        sync_done = true;

                                        // Send NEG-CLOSE message
                                        self.send_neg_close(subscription_id)?;
                                    }
                                }
                            }
                        }
                        RelayMessage::NegErr {
                            subscription_id,
                            code,
                        } => {
                            if subscription_id == sub_id {
                                return Err(Error::NegentropyReconciliation(code));
                            }
                        }
                        RelayMessage::Ok {
                            event_id,
                            status,
                            message,
                        } => {
                            if in_flight_up.remove(&event_id) {
                                if status {
                                    output.sent.insert(event_id);
                                } else {
                                    tracing::error!(
                                        "Unable to upload event {event_id} to '{}': {message}",
                                        self.url
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
                    if do_up
                        && !have_ids.is_empty()
                        && in_flight_up.len() <= NEGENTROPY_LOW_WATER_UP
                    {
                        let mut num_sent = 0;

                        while !have_ids.is_empty() && in_flight_up.len() < NEGENTROPY_HIGH_WATER_UP
                        {
                            if let Some(id) = have_ids.pop() {
                                match self.database.event_by_id(&id).await {
                                    Ok(Some(event)) => {
                                        in_flight_up.insert(id);
                                        self.send_msg(ClientMessage::event(event))?;
                                        num_sent += 1;
                                    }
                                    Ok(None) => {
                                        // Event not found
                                    }
                                    Err(e) => tracing::error!(
                                        "Couldn't upload event to '{}': {e}",
                                        self.url
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
                    }

                    // Get events
                    if do_down && !need_ids.is_empty() && !in_flight_down {
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
                        self.send_msg(ClientMessage::req(down_sub_id.clone(), vec![filter]))?;

                        in_flight_down = true
                    }
                }
                RelayNotification::RelayStatus { status } => {
                    if status.is_disconnected() {
                        return Err(Error::NotConnected);
                    }
                }
                RelayNotification::Shutdown => {
                    return Err(Error::Shutdown);
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

        tracing::info!("Negentropy reconciliation terminated for {}", self.url);

        Ok(())
    }

    /// Deprecated negentropy protocol
    async fn sync_deprecated(
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

        // Send initial negentropy message
        let sub_id = SubscriptionId::generate();
        let open_msg = ClientMessage::neg_open_deprecated(&mut negentropy, sub_id.clone(), filter)?;
        self.send_msg(open_msg)?;

        let mut notifications = self.internal_notification_sender.subscribe();
        let mut temp_notifications = self.internal_notification_sender.subscribe();

        // Check if negentropy it's supported
        time::timeout(Some(opts.initial_timeout), async {
            while let Ok(notification) = temp_notifications.recv().await {
                if let RelayNotification::Message { message } = notification {
                    match message {
                        RelayMessage::NegMsg {
                            subscription_id, ..
                        } => {
                            if subscription_id == sub_id {
                                break;
                            }
                        }
                        RelayMessage::NegErr {
                            subscription_id,
                            code,
                        } => {
                            if subscription_id == sub_id {
                                return Err(Error::NegentropyReconciliation(code));
                            }
                        }
                        RelayMessage::Notice { message } => {
                            if message.contains("bad msg")
                                && (message.contains("unknown cmd")
                                    || message.contains("negentropy")
                                    || message.contains("NEG-"))
                            {
                                return Err(Error::NegentropyMaybeNotSupported);
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

            Ok::<(), Error>(())
        })
        .await
        .ok_or(Error::Timeout)??;

        let do_up: bool = opts.do_up();
        let do_down: bool = opts.do_down();
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

                                let mut counter: u64 = 0;

                                // If event ID wasn't already seen, add to the HAVE IDs
                                // Add to HAVE IDs only if `do_up` is true
                                for id in curr_have_ids
                                    .into_iter()
                                    .filter_map(|id| EventId::from_slice(id.as_bytes()).ok())
                                {
                                    if output.local.insert(id) && do_up {
                                        have_ids.push(id);
                                        counter += 1;
                                    }
                                }

                                // If event ID wasn't already seen, add to the NEED IDs
                                // Add to NEED IDs only if `do_down` is true
                                for id in curr_need_ids
                                    .into_iter()
                                    .filter_map(|id| EventId::from_slice(id.as_bytes()).ok())
                                {
                                    if output.remote.insert(id) && do_down {
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
                                    Some(query) => {
                                        tracing::debug!(
                                            "Continue negentropy reconciliation with '{}'",
                                            self.url
                                        );
                                        self.send_neg_msg(subscription_id, query.to_hex())?;
                                    }
                                    None => {
                                        // Mark sync as done
                                        sync_done = true;

                                        // Send NEG-CLOSE message
                                        self.send_neg_close(subscription_id)?;
                                    }
                                }
                            }
                        }
                        RelayMessage::NegErr {
                            subscription_id,
                            code,
                        } => {
                            if subscription_id == sub_id {
                                return Err(Error::NegentropyReconciliation(code));
                            }
                        }
                        RelayMessage::Ok {
                            event_id,
                            status,
                            message,
                        } => {
                            if in_flight_up.remove(&event_id) {
                                if status {
                                    output.sent.insert(event_id);
                                } else {
                                    tracing::error!(
                                        "Unable to upload event {event_id} to '{}': {message}",
                                        self.url
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

                    // Get/Send events
                    if do_up
                        && !have_ids.is_empty()
                        && in_flight_up.len() <= NEGENTROPY_LOW_WATER_UP
                    {
                        let mut num_sent = 0;

                        while !have_ids.is_empty() && in_flight_up.len() < NEGENTROPY_HIGH_WATER_UP
                        {
                            if let Some(id) = have_ids.pop() {
                                match self.database.event_by_id(&id).await {
                                    Ok(Some(event)) => {
                                        in_flight_up.insert(id);
                                        self.send_msg(ClientMessage::event(event))?;
                                        num_sent += 1;
                                    }
                                    Ok(None) => {
                                        // Event not found
                                    }
                                    Err(e) => tracing::error!(
                                        "Couldn't upload event to {}: {e}",
                                        self.url
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
                    }

                    if do_down && !need_ids.is_empty() && !in_flight_down {
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
                        self.send_msg(ClientMessage::req(down_sub_id.clone(), vec![filter]))?;

                        in_flight_down = true
                    }
                }
                RelayNotification::RelayStatus { status } => {
                    if status.is_disconnected() {
                        return Err(Error::NotConnected);
                    }
                }
                RelayNotification::Shutdown => return Err(Error::Shutdown),
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

        tracing::info!("Negentropy reconciliation terminated for {}", self.url);

        Ok(())
    }
}

/// Send WebSocket messages with timeout set to [WEBSOCKET_TX_TIMEOUT].
async fn send_ws_msgs<I>(tx: &mut Sink, msgs: I) -> Result<(), Error>
where
    I: IntoIterator<Item = WsMessage>,
{
    let mut stream = futures_util::stream::iter(msgs.into_iter().map(Ok));
    match time::timeout(Some(WEBSOCKET_TX_TIMEOUT), tx.send_all(&mut stream)).await {
        Some(res) => res.map_err(Error::websocket),
        None => Err(Error::WebSocketTimeout),
    }
}

/// Send WebSocket messages with timeout set to [WEBSOCKET_TX_TIMEOUT].
async fn close_ws(tx: &mut Sink) -> Result<(), Error> {
    match time::timeout(Some(WEBSOCKET_TX_TIMEOUT), tx.close()).await {
        Some(res) => res.map_err(Error::websocket),
        None => Err(Error::WebSocketTimeout),
    }
}
