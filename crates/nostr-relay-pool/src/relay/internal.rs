// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Internal Relay

use std::cmp;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::{futures_util, thread, time};
use async_wsocket::futures_util::{Future, SinkExt, StreamExt};
use async_wsocket::{Sink, Stream, WsMessage};
use atomic_destructor::AtomicDestroyer;
use nostr::message::MessageHandleError;
use nostr::negentropy::{Bytes, Negentropy};
use nostr::nips::nip01::Coordinate;
#[cfg(feature = "nip11")]
use nostr::nips::nip11::RelayInformationDocument;
use nostr::secp256k1::rand::{self, Rng};
use nostr::{
    ClientMessage, Event, EventId, Filter, JsonUtil, Keys, Kind, MissingPartialEvent, PartialEvent,
    RawRelayMessage, RelayMessage, SubscriptionId, Timestamp, Url,
};
use nostr_database::{DynNostrDatabase, Order};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot, watch, Mutex, MutexGuard, RwLock};

use super::blacklist::RelayBlacklist;
#[cfg(not(target_arch = "wasm32"))]
use super::constants::HIGH_LATENCY;
use super::constants::{MIN_ATTEMPTS, MIN_UPTIME, PING_INTERVAL, WEBSOCKET_TX_TIMEOUT};
use super::flags::AtomicRelayServiceFlags;
use super::options::{
    FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions, SubscribeAutoCloseOptions,
    SubscribeOptions, MAX_ADJ_RETRY_SEC, MIN_RETRY_SEC, NEGENTROPY_BATCH_SIZE_DOWN,
    NEGENTROPY_HIGH_WATER_UP, NEGENTROPY_LOW_WATER_UP,
};
use super::stats::RelayConnectionStats;
use super::{Error, Reconciliation, RelayNotification, RelayStatus};
use crate::pool::RelayPoolNotification;

struct NostrMessage {
    msgs: Vec<ClientMessage>,
    shot: Option<oneshot::Sender<bool>>,
}

#[derive(Debug, Clone, Copy)]
enum RelayServiceEvent {
    /// None
    None,
    /// Completely disconnect
    Terminate,
}

#[derive(Debug, Clone)]
struct RelayChannels {
    nostr: (Sender<NostrMessage>, Arc<Mutex<Receiver<NostrMessage>>>),
    ping: (watch::Sender<u64>, Arc<Mutex<watch::Receiver<u64>>>),
    service: (
        watch::Sender<RelayServiceEvent>,
        Arc<Mutex<watch::Receiver<RelayServiceEvent>>>,
    ),
}

impl RelayChannels {
    pub fn new() -> Self {
        let (tx_nostr, rx_nostr) = mpsc::channel::<NostrMessage>(1024);
        let (tx_ping, rx_ping) = watch::channel::<u64>(0);
        let (tx_service, rx_service) = watch::channel::<RelayServiceEvent>(RelayServiceEvent::None);

        Self {
            nostr: (tx_nostr, Arc::new(Mutex::new(rx_nostr))),
            ping: (tx_ping, Arc::new(Mutex::new(rx_ping))),
            service: (tx_service, Arc::new(Mutex::new(rx_service))),
        }
    }

    #[inline]
    pub fn send_nostr_msg(&self, msg: NostrMessage) -> Result<(), Error> {
        self.nostr
            .0
            .try_send(msg)
            .map_err(|_| Error::CantSendChannelMessage {
                channel: String::from("nostr"),
            })
    }

    #[inline]
    pub async fn rx_nostr(&self) -> MutexGuard<'_, Receiver<NostrMessage>> {
        self.nostr.1.lock().await
    }

    #[inline]
    pub fn nostr_queue(&self) -> usize {
        self.nostr.0.max_capacity() - self.nostr.0.capacity()
    }

    #[inline]
    pub fn nostr_capacity(&self) -> usize {
        self.nostr.0.capacity()
    }

    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn ping(&self, nonce: u64) -> Result<(), Error> {
        self.ping
            .0
            .send(nonce)
            .map_err(|_| Error::CantSendChannelMessage {
                channel: String::from("ping"),
            })
    }

    #[inline]
    pub async fn rx_ping(&self) -> MutexGuard<'_, watch::Receiver<u64>> {
        self.ping.1.lock().await
    }

    #[inline]
    pub async fn rx_service(&self) -> MutexGuard<'_, watch::Receiver<RelayServiceEvent>> {
        self.service.1.lock().await
    }

    #[inline]
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
pub(crate) struct InternalRelay {
    pub(super) url: Url,
    status: Arc<RwLock<RelayStatus>>,
    #[cfg(feature = "nip11")]
    document: Arc<RwLock<RelayInformationDocument>>,
    opts: RelayOptions,
    stats: RelayConnectionStats,
    blacklist: RelayBlacklist,
    database: Arc<DynNostrDatabase>,
    channels: RelayChannels,
    scheduled_for_termination: Arc<AtomicBool>,
    pub(super) internal_notification_sender: broadcast::Sender<RelayNotification>,
    external_notification_sender: Arc<RwLock<Option<broadcast::Sender<RelayPoolNotification>>>>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, SubscriptionData>>>,
}

impl AtomicDestroyer for InternalRelay {
    fn name(&self) -> Option<String> {
        Some(format!("Relay {}", self.url))
    }

    fn on_destroy(&self) {
        let relay = self.clone();
        let _ = thread::spawn(async move {
            if let Err(e) = relay.disconnect().await {
                tracing::error!("Impossible to shutdown '{}': {e}", relay.url);
            }
        });
    }
}

impl InternalRelay {
    pub fn new(
        url: Url,
        database: Arc<DynNostrDatabase>,
        blacklist: RelayBlacklist,
        opts: RelayOptions,
    ) -> Self {
        let (relay_notification_sender, ..) = broadcast::channel::<RelayNotification>(2048);

        Self {
            url,
            status: Arc::new(RwLock::new(RelayStatus::Initialized)),
            #[cfg(feature = "nip11")]
            document: Arc::new(RwLock::new(RelayInformationDocument::new())),
            opts,
            stats: RelayConnectionStats::new(),
            blacklist,
            database,
            channels: RelayChannels::new(),
            scheduled_for_termination: Arc::new(AtomicBool::new(false)),
            internal_notification_sender: relay_notification_sender,
            external_notification_sender: Arc::new(RwLock::new(None)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[inline]
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn proxy(&self) -> Option<SocketAddr> {
        #[cfg(not(target_arch = "wasm32"))]
        let proxy = self.opts.proxy;

        #[cfg(target_arch = "wasm32")]
        let proxy = None;

        proxy
    }

    pub async fn status(&self) -> RelayStatus {
        let status = self.status.read().await;
        *status
    }

    async fn set_status(&self, status: RelayStatus, log: bool) {
        // Change status
        let mut s = self.status.write().await;
        *s = status;

        // Log
        if log {
            match status {
                RelayStatus::Initialized => tracing::trace!("'{}' relay initialized.", self.url),
                RelayStatus::Pending => tracing::trace!("'{}' relay is pending.", self.url),
                RelayStatus::Connecting => tracing::debug!("Connecting to '{}'", self.url),
                RelayStatus::Connected => tracing::info!("Connected to '{}'", self.url),
                RelayStatus::Disconnected => tracing::info!("Disconnected from '{}'", self.url),
                RelayStatus::Terminated => {
                    tracing::info!("Completely disconnected from '{}'", self.url)
                }
            }
        }

        // Send notification
        self.send_notification(RelayNotification::RelayStatus { status }, true)
            .await;
    }

    #[inline]
    pub fn flags(&self) -> AtomicRelayServiceFlags {
        self.opts.flags.clone()
    }

    #[inline]
    pub fn blacklist(&self) -> RelayBlacklist {
        self.blacklist.clone()
    }

    #[inline]
    pub async fn is_connected(&self) -> bool {
        self.status().await == RelayStatus::Connected
    }

    /// Check if is `disconnected`, `stopped` or `terminated`
    #[inline]
    pub async fn is_disconnected(&self) -> bool {
        self.status().await.is_disconnected()
    }

    #[cfg(feature = "nip11")]
    pub async fn document(&self) -> RelayInformationDocument {
        let document = self.document.read().await;
        document.clone()
    }

    #[cfg(feature = "nip11")]
    async fn set_document(&self, document: RelayInformationDocument) {
        let mut d = self.document.write().await;
        *d = document;
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

    /// Check if should subscribe for current websocket session
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
    pub fn opts(&self) -> RelayOptions {
        self.opts.clone()
    }

    #[inline]
    pub fn stats(&self) -> RelayConnectionStats {
        self.stats.clone()
    }

    #[inline]
    pub fn queue(&self) -> usize {
        self.channels.nostr_queue()
    }

    #[inline]
    fn is_scheduled_for_termination(&self) -> bool {
        self.scheduled_for_termination.load(Ordering::SeqCst)
    }

    #[inline]
    fn schedule_for_termination(&self, value: bool) {
        self.scheduled_for_termination
            .store(value, Ordering::SeqCst);
    }

    pub async fn set_notification_sender(
        &self,
        notification_sender: Option<broadcast::Sender<RelayPoolNotification>>,
    ) {
        let mut external_notification_sender = self.external_notification_sender.write().await;
        *external_notification_sender = notification_sender;
    }

    async fn send_notification(&self, notification: RelayNotification, external: bool) {
        // Send internal notification
        let _ = self.internal_notification_sender.send(notification.clone());

        // Send external notification
        if external {
            let external_notification_sender = self.external_notification_sender.read().await;
            if let Some(external_notification_sender) = external_notification_sender.as_ref() {
                // Convert relay to notification to pool notification
                let notification: RelayPoolNotification = match notification {
                    RelayNotification::Event {
                        subscription_id,
                        event,
                    } => RelayPoolNotification::Event {
                        relay_url: self.url(),
                        subscription_id,
                        event,
                    },
                    RelayNotification::Message { message } => RelayPoolNotification::Message {
                        relay_url: self.url(),
                        message,
                    },
                    RelayNotification::RelayStatus { status } => {
                        RelayPoolNotification::RelayStatus {
                            relay_url: self.url(),
                            status,
                        }
                    }
                    RelayNotification::Shutdown => RelayPoolNotification::Shutdown,
                };

                // Send notification
                let _ = external_notification_sender.send(notification);
            }
        }
    }

    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        self.schedule_for_termination(false); // TODO: remove?

        if let RelayStatus::Initialized | RelayStatus::Terminated = self.status().await {
            if self.opts.get_reconnect() {
                // If connection timeout is not null, try to connect
                match connection_timeout {
                    Some(..) => self.try_connect(connection_timeout).await,
                    None => {
                        // Set status to 'pending' so it'll connect in next step
                        self.set_status(RelayStatus::Pending, true).await;
                    }
                }

                tracing::debug!("Auto connect loop started for {}", self.url);

                let relay = self.clone();
                let _ = thread::spawn(async move {
                    loop {
                        let queue: usize = relay.queue();
                        if queue > 0 {
                            tracing::info!(
                                "{} messages queued for {} (capacity: {})",
                                queue,
                                relay.url(),
                                relay.channels.nostr_capacity()
                            );
                        }

                        // Log high latency
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(latency) = relay.stats.latency().await {
                            let reads: usize = relay.stats.latency_reads().await;

                            if latency >= HIGH_LATENCY && reads >= 3 {
                                tracing::warn!(
                                    "Latency of '{}' relay is high, averaging over {} ms!",
                                    relay.url(),
                                    latency.as_millis()
                                );
                            }
                        }

                        // Schedule relay for termination
                        // Needed to terminate the auto reconnect loop, also if the relay is not connected yet.
                        if relay.is_scheduled_for_termination() {
                            relay.set_status(RelayStatus::Terminated, true).await;
                            relay.schedule_for_termination(false);
                            tracing::debug!(
                                "Auto connect loop terminated for {} [schedule]",
                                relay.url
                            );
                            break;
                        }

                        // Check status
                        match relay.status().await {
                            RelayStatus::Initialized
                            | RelayStatus::Pending
                            | RelayStatus::Disconnected => {
                                relay.try_connect(connection_timeout).await
                            }
                            RelayStatus::Terminated => {
                                tracing::debug!("Auto connect loop terminated for {}", relay.url);
                                break;
                            }
                            _ => (),
                        };

                        // Sleep
                        let retry_sec: u64 = relay.calculate_retry_sec();
                        tracing::trace!("{} retry time set to {retry_sec} secs", relay.url);
                        thread::sleep(Duration::from_secs(retry_sec)).await;
                    }
                });
            } else if connection_timeout.is_some() {
                self.try_connect(connection_timeout).await
            } else {
                let relay = self.clone();
                let _ = thread::spawn(async move { relay.try_connect(connection_timeout).await });
            }
        }
    }

    /// Depending on attempts and success, use default or incremental retry time
    fn calculate_retry_sec(&self) -> u64 {
        if self.opts.get_adjust_retry_sec() {
            // diff = attempts - success
            let diff: u64 = self.stats.attempts().saturating_sub(self.stats.success()) as u64;

            // Use incremental retry time if diff >= 3
            if diff >= 3 {
                let retry_interval: i64 =
                    cmp::min(MIN_RETRY_SEC * (1 + diff), MAX_ADJ_RETRY_SEC) as i64;
                let jitter: i64 = rand::thread_rng().gen_range(-1..=1);
                return retry_interval.saturating_add(jitter) as u64;
            }
        }

        // Use default retry time
        self.opts.get_retry_sec()
    }

    #[cfg(feature = "nip11")]
    fn request_nip11_document(&self) {
        let relay = self.clone();
        let _ = thread::spawn(async move {
            match RelayInformationDocument::get(relay.url(), relay.proxy()).await {
                Ok(document) => relay.set_document(document).await,
                Err(e) => tracing::error!(
                    "Impossible to get information document from '{}': {}",
                    relay.url,
                    e
                ),
            };
        });
    }

    async fn sender_message_handler(&self, mut ws_tx: Sink) {
        // Lock receivers
        let mut rx_nostr = self.channels.rx_nostr().await;
        let mut rx_ping = self.channels.rx_ping().await;
        let mut rx_service = self.channels.rx_service().await;

        loop {
            tokio::select! {
                // Nostr channel receiver
                Some(NostrMessage { msgs, shot }) = rx_nostr.recv() => {
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

                    tracing::debug!("Sending {partial_log_msg} (size: {size} bytes)");

                    // Send WebSocket messages
                    let status: bool = match send_ws_msgs(&mut ws_tx, msgs).await {
                        Ok(()) => {
                            // TODO: tracing::debug!("Sent {partial_log_msg} (size: {size} bytes)");
                            self.stats.add_bytes_sent(size);
                            true
                        }
                        Err(e) => {
                            tracing::error!("Impossible to send {partial_log_msg}: {e}");
                            false
                        }
                    };

                    // Send oneshot message
                    if let Some(sender) = shot {
                        if sender.send(status).is_err() {
                            tracing::trace!(
                                "Impossible to send '{status}' oneshot msg for '{}",
                                self.url
                            );
                        }
                    }

                    // If error, break receiver loop
                    if !status {
                        break;
                    }
                }
                // Ping channel receiver
                Ok(()) = rx_ping.changed() => {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                         // Get nonce and mark as seen
                        let nonce: u64 = *rx_ping.borrow_and_update();

                        // Compose ping message
                        let msg = WsMessage::Ping(nonce.to_string().as_bytes().to_vec());

                        // Send WebSocket message
                        match send_ws_msgs(&mut ws_tx, [msg]).await {
                            Ok(()) => {
                                self.stats.ping.just_sent().await;
                                tracing::debug!("Ping '{}' (nonce: {nonce})", self.url);
                            }
                            Err(e) => {
                                tracing::error!("Impossible to ping '{}': {e}", self.url);
                                break;
                            }
                        }
                    }
                }
                // Service channel receiver (stop, shutdown, ..)
                Ok(()) = rx_service.changed() => {
                    // Get service and mark as seen
                    let service: RelayServiceEvent = *rx_service.borrow_and_update();

                    match service {
                        // Do nothing
                        RelayServiceEvent::None => {},
                        // Terminate
                        RelayServiceEvent::Terminate => {
                            if self.is_scheduled_for_termination() {
                                self.set_status(RelayStatus::Terminated, true).await;
                                self.schedule_for_termination(false);
                                break;
                            }
                        }
                    }
                }
                else => break
            }
        }

        // Close WebSocket
        match close_ws(&mut ws_tx).await {
            Ok(..) => {
                tracing::debug!("WebSocket closed for '{}'", self.url);
            }
            Err(e) => {
                tracing::error!("Impossible to close WebSocket for '{}': {e}", self.url);
            }
        }
    }

    async fn receiver_message_handler(&self, mut ws_rx: Stream) {
        while let Some(msg) = ws_rx.next().await {
            if let Ok(msg) = msg {
                match msg {
                    #[cfg(not(target_arch = "wasm32"))]
                    WsMessage::Pong(bytes) => {
                        if self.opts.flags.has_ping() {
                            match String::from_utf8(bytes) {
                                Ok(nonce) => match nonce.parse::<u64>() {
                                    Ok(nonce) => {
                                        if self.stats.ping.last_nonce() == nonce {
                                            tracing::debug!(
                                                "Pong from '{}' match nonce: {}",
                                                self.url,
                                                nonce
                                            );
                                            self.stats.ping.set_replied(true);
                                            let sent_at = self.stats.ping.sent_at().await;
                                            self.stats.save_latency(sent_at.elapsed()).await;
                                        } else {
                                            tracing::error!("Pong nonce not match: received={nonce}, expected={}", self.stats.ping.last_nonce());
                                        }
                                    }
                                    Err(e) => tracing::error!("{e}"),
                                },
                                Err(e) => tracing::error!("{e}"),
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

    fn spawn_message_handler(&self, ws_tx: Sink, ws_rx: Stream) -> Result<(), Error> {
        let relay = self.clone();
        thread::spawn(async move {
            tracing::debug!("Relay Message Handler started for '{}'", relay.url);

            #[cfg(not(target_arch = "wasm32"))]
            let pinger = async {
                if relay.opts.flags.has_ping() {
                    loop {
                        // If last nonce is NOT 0, check if relay replied
                        // Break loop if relay not replied
                        if relay.stats.ping.last_nonce() != 0 && !relay.stats.ping.replied() {
                            tracing::warn!("'{}' not replied to ping", relay.url);
                            relay.stats.ping.reset();
                            break;
                        }

                        // Generate and save nonce
                        let nonce: u64 = rand::random();
                        relay.stats.ping.set_last_nonce(nonce);
                        relay.stats.ping.set_replied(false);

                        // Ping
                        if let Err(e) = relay.channels.ping(nonce) {
                            tracing::error!("Impossible to ping '{}': {e}", relay.url);
                            break;
                        };

                        // Sleep
                        thread::sleep(PING_INTERVAL).await;
                    }
                } else {
                    loop {
                        thread::sleep(PING_INTERVAL).await;
                    }
                }
            };

            #[cfg(target_arch = "wasm32")]
            let pinger = async {
                loop {
                    thread::sleep(PING_INTERVAL).await;
                }
            };

            // Wait that one of the futures terminate/complete
            tokio::select! {
                _ = relay.receiver_message_handler(ws_rx) => {
                    tracing::trace!("Relay connection closed for '{}'", relay.url);
                }
                _ = relay.sender_message_handler(ws_tx) => {
                    tracing::trace!("Relay sender exited for '{}'", relay.url);
                }
                _ = pinger => {
                    tracing::trace!("Relay pinger exited for '{}'", relay.url);
                }
            }

            // Check if relay is marked as disconnected. If not, update status.
            if !relay.is_disconnected().await {
                relay.set_status(RelayStatus::Disconnected, true).await;
            }

            tracing::debug!("Exited from Message Handler for '{}'", relay.url);
        })?;
        Ok(())
    }

    async fn try_connect(&self, connection_timeout: Option<Duration>) {
        self.stats.new_attempt();

        let url: String = self.url.to_string();

        // Set RelayStatus to `Connecting`
        self.set_status(RelayStatus::Connecting, true).await;

        // Request `RelayInformationDocument`
        #[cfg(feature = "nip11")]
        self.request_nip11_document();

        // Compose timeout
        let timeout: Option<Duration> = if self.stats.attempts() > 1 {
            // Many attempts, use the default timeout
            Some(Duration::from_secs(60))
        } else {
            // First attempt, use external timeout
            connection_timeout
        };

        // Connect
        match async_wsocket::connect(&self.url, self.proxy(), timeout).await {
            Ok((ws_tx, ws_rx)) => {
                // Update status
                self.set_status(RelayStatus::Connected, true).await;

                // Increment success stats
                self.stats.new_success();

                // Spawn message handler
                match self.spawn_message_handler(ws_tx, ws_rx) {
                    Ok(()) => {
                        // Subscribe to relay
                        if self.opts.flags.has_read() {
                            let opts: RelaySendOptions =
                                RelaySendOptions::default().skip_send_confirmation(true);
                            if let Err(e) = self.resubscribe(opts).await {
                                tracing::error!("Impossible to subscribe to '{url}': {e}")
                            }
                        }
                    }
                    Err(e) => {
                        self.set_status(RelayStatus::Disconnected, false).await;
                        tracing::error!("Impossible to spawn message handler for '{url}': {e}");
                    }
                }
            }
            Err(e) => {
                self.set_status(RelayStatus::Disconnected, false).await;
                tracing::error!("Impossible to connect to '{url}': {e}");
            }
        };
    }

    #[inline(always)]
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
                        self.subscription_closed(subscription_id).await;
                    }
                    _ => (),
                }

                // Send notification
                self.send_notification(RelayNotification::Message { message }, true)
                    .await;
            }
            Ok(None) | Err(Error::MessageHandle(MessageHandleError::EmptyMsg)) => (),
            Err(e) => tracing::error!(
                "Impossible to handle relay message from '{}': {e}",
                self.url
            ),
        }
    }

    #[inline(always)]
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

                // Check blacklist (ID)
                if self.blacklist.has_id(&partial_event.id).await {
                    return Err(Error::EventIdBlacklisted(partial_event.id));
                }

                // Check blacklist (author public key)
                if self.blacklist.has_public_key(&partial_event.pubkey).await {
                    return Err(Error::PublicKeyBlacklisted(partial_event.pubkey));
                }

                // Check min POW
                let difficulty: u8 = self.opts.get_pow_difficulty();
                if difficulty > 0 && !partial_event.id.check_pow(difficulty) {
                    return Err(Error::PowDifficultyTooLow { min: difficulty });
                }

                // Check if event has been deleted
                if self
                    .database
                    .has_event_id_been_deleted(&partial_event.id)
                    .await?
                {
                    tracing::warn!(
                        "Received event {} that was deleted: type=id, relay_url={}",
                        partial_event.id,
                        self.url
                    );
                    return Ok(None);
                }

                // Deserialize missing event fields
                let missing: MissingPartialEvent = MissingPartialEvent::from_raw(event);

                // TODO: check if word/hashtag is blacklisted

                // Check if event is replaceable and has coordinate
                if missing.kind.is_replaceable() || missing.kind.is_parameterized_replaceable() {
                    let coordinate: Coordinate =
                        Coordinate::new(missing.kind, partial_event.pubkey)
                            .identifier(missing.identifier().unwrap_or_default());
                    // Check if event has been deleted
                    if self
                        .database
                        .has_coordinate_been_deleted(&coordinate, missing.created_at)
                        .await?
                    {
                        tracing::warn!(
                            "Received event {} that was deleted: type=coordinate, relay_url={}",
                            partial_event.id,
                            self.url
                        );
                        return Ok(None);
                    }
                }

                // Check if event id was already seen
                let seen: bool = self
                    .database
                    .has_event_already_been_seen(&partial_event.id)
                    .await?;

                // Set event as seen by relay
                if let Err(e) = self
                    .database
                    .event_id_seen(partial_event.id, self.url())
                    .await
                {
                    tracing::error!(
                        "Impossible to set event {} as seen by relay: {e}",
                        partial_event.id
                    );
                }

                // Check if event was already saved
                let saved: bool = self
                    .database
                    .has_event_already_been_saved(&partial_event.id)
                    .await?;

                // Compose full event
                let event: Event = partial_event.merge(missing)?;

                // Check if it's expired
                if event.is_expired() {
                    return Err(Error::EventExpired);
                }

                // Check if saved
                if !saved {
                    // Verify event
                    event.verify()?;

                    // Save event
                    self.database.save_event(&event).await?;
                }

                // Box event
                let event: Box<Event> = Box::new(event);

                // Check if seen
                if !seen {
                    // Send notification
                    self.send_notification(
                        RelayNotification::Event {
                            subscription_id: SubscriptionId::new(&subscription_id),
                            event: event.clone(),
                        },
                        true,
                    )
                    .await;
                }

                Ok(Some(RelayMessage::Event {
                    subscription_id: SubscriptionId::new(subscription_id),
                    event,
                }))
            }
            m => Ok(Some(RelayMessage::try_from(m)?)),
        }
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        self.schedule_for_termination(true); // TODO: remove?
        if !self.is_disconnected().await {
            self.channels
                .send_service_msg(RelayServiceEvent::Terminate)?;
        }
        self.send_notification(RelayNotification::Shutdown, false)
            .await;
        Ok(())
    }

    #[inline]
    pub async fn send_msg(&self, msg: ClientMessage, opts: RelaySendOptions) -> Result<(), Error> {
        self.batch_msg(vec![msg], opts).await
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        if !self.opts.flags.has_write() && msgs.iter().any(|msg| msg.is_event()) {
            return Err(Error::WriteDisabled);
        }

        if !self.opts.flags.has_read() && msgs.iter().any(|msg| msg.is_req() || msg.is_close()) {
            return Err(Error::ReadDisabled);
        }

        if opts.skip_disconnected
            && !self.is_connected().await
            && self.stats.attempts() > MIN_ATTEMPTS
            && self.stats.uptime() < MIN_UPTIME
        {
            return Err(Error::NotConnected);
        }

        if opts.skip_send_confirmation {
            self.channels
                .send_nostr_msg(NostrMessage { msgs, shot: None })
        } else {
            // Create new oneshot channel
            let (tx, rx) = oneshot::channel::<bool>();

            // Send message
            self.channels.send_nostr_msg(NostrMessage {
                msgs,
                shot: Some(tx),
            })?;

            // Wait for oneshot reply
            match time::timeout(Some(opts.timeout), rx).await {
                Some(result) => match result {
                    Ok(val) => {
                        if val {
                            Ok(())
                        } else {
                            Err(Error::MessageNotSent)
                        }
                    }
                    Err(_) => Err(Error::OneShotRecvError),
                },
                None => Err(Error::RecvTimeout),
            }
        }
    }

    #[inline]
    pub async fn send_event(&self, event: Event, opts: RelaySendOptions) -> Result<EventId, Error> {
        let id: EventId = event.id();
        self.batch_event(vec![event], opts).await?;
        Ok(id)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        if events.is_empty() {
            return Err(Error::BatchEventEmpty);
        }

        let events_len: usize = events.len();
        let mut msgs: Vec<ClientMessage> = Vec::with_capacity(events_len);
        let mut missing: HashSet<EventId> = HashSet::with_capacity(events_len);

        for event in events.into_iter() {
            missing.insert(event.id());
            msgs.push(ClientMessage::event(event));
        }

        let mut notifications = self.internal_notification_sender.subscribe();

        // Batch send messages
        self.batch_msg(msgs, opts).await?;

        // Handle responses
        time::timeout(Some(opts.timeout), async {
            let mut published: HashSet<EventId> = HashSet::new();
            let mut not_published: HashMap<EventId, String> = HashMap::new();
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
                        if missing.remove(&event_id) {
                            if events_len == 1 {
                                return if status {
                                    Ok(())
                                } else {
                                    Err(Error::EventNotPublished(message))
                                };
                            }

                            if status {
                                published.insert(event_id);
                            } else {
                                not_published.insert(event_id, message);
                            }
                        }
                    }
                    RelayNotification::RelayStatus { status } => {
                        if opts.skip_disconnected && status.is_disconnected() {
                            return Err(Error::EventNotPublished(String::from(
                                "relay not connected (status changed)",
                            )));
                        }
                    }
                    _ => (),
                }

                if missing.is_empty() {
                    break;
                }
            }

            if !published.is_empty() && not_published.is_empty() {
                Ok(())
            } else if !published.is_empty() && !not_published.is_empty() {
                Err(Error::PartialPublish {
                    published: published.into_iter().collect(),
                    not_published,
                })
            } else {
                Err(Error::EventsNotPublished(not_published))
            }
        })
        .await
        .ok_or(Error::Timeout)?
    }

    pub async fn auth(&self, event: Event, opts: RelaySendOptions) -> Result<(), Error> {
        // Check if NIP-42 event
        if event.kind != Kind::Authentication {
            return Err(Error::UnexpectedKind {
                expected: Kind::Authentication,
                found: event.kind(),
            });
        }

        let mut notifications = self.internal_notification_sender.subscribe();

        let id: EventId = event.id();

        // Send message
        let msg: ClientMessage = ClientMessage::auth(event);
        self.send_msg(msg, opts).await?;

        // Handle responses
        time::timeout(Some(opts.timeout), async {
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
                        if id == event_id {
                            return if status {
                                Ok(())
                            } else {
                                Err(Error::EventNotPublished(message))
                            };
                        }
                    }
                    RelayNotification::RelayStatus { status } => {
                        if opts.skip_disconnected && status.is_disconnected() {
                            return Err(Error::EventNotPublished(String::from(
                                "relay not connected (status changed)",
                            )));
                        }
                    }
                    _ => (),
                }
            }

            Err(Error::EventNotPublished(String::from("loop terminated")))
        })
        .await
        .ok_or(Error::Timeout)?
    }

    pub async fn resubscribe(&self, opts: RelaySendOptions) -> Result<(), Error> {
        if !self.opts.flags.has_read() {
            return Err(Error::ReadDisabled);
        }

        let subscriptions = self.subscriptions().await;
        for (id, filters) in subscriptions.into_iter() {
            if !filters.is_empty() && self.should_resubscribe(&id).await {
                self.send_msg(ClientMessage::req(id, filters), opts).await?;
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
        // Check if relay has READ flags disabled
        if !self.opts.flags.has_read() {
            return Err(Error::ReadDisabled);
        }

        // Check if filters are empty
        if filters.is_empty() {
            return Err(Error::FiltersEmpty);
        }

        // Compose and send REQ message
        let msg: ClientMessage = ClientMessage::req(id.clone(), filters.clone());
        self.send_msg(msg, opts.send_opts).await?;

        // TODO: check if relay send CLOSED message?

        // Check if auto-close condition is set
        match opts.auto_close {
            Some(opts) => {
                let this = self.clone();
                thread::spawn(async move {
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
                        this.send_msg(
                            ClientMessage::close(sub_id.clone()),
                            RelaySendOptions::default(),
                        )
                        .await?;

                        tracing::debug!("Subscription {sub_id} auto-closed");
                    }

                    Ok::<(), Error>(())
                })?;
            }
            None => {
                // No auto-close subscription: update subscription filters
                self.update_subscription(id.clone(), filters, true).await;
            }
        };

        Ok(())
    }

    pub async fn unsubscribe(
        &self,
        id: SubscriptionId,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        if !self.opts.flags.has_read() {
            return Err(Error::ReadDisabled);
        }

        // Remove subscription
        self.remove_subscription(&id).await;

        // Send CLOSE message
        let msg: ClientMessage = ClientMessage::close(id);
        self.send_msg(msg, opts).await
    }

    pub async fn unsubscribe_all(&self, opts: RelaySendOptions) -> Result<(), Error> {
        if !self.opts.flags.has_read() {
            return Err(Error::ReadDisabled);
        }

        let subscriptions = self.subscriptions().await;

        for id in subscriptions.into_keys() {
            // Remove subscription
            self.remove_subscription(&id).await;

            // Send CLOSE message
            let msg: ClientMessage = ClientMessage::close(id);
            self.send_msg(msg, opts).await?;
        }

        Ok(())
    }

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
        // Check if relay is connected
        if !self.is_connected().await
            && self.stats.attempts() > MIN_ATTEMPTS
            && self.stats.uptime() < MIN_UPTIME
        {
            return Err(Error::NotConnected);
        }

        // Compose options
        let auto_close_opts: SubscribeAutoCloseOptions = SubscribeAutoCloseOptions::default()
            .filter(opts)
            .timeout(Some(timeout));
        let send_opts: RelaySendOptions = RelaySendOptions::default().timeout(Some(timeout));
        let subscribe_opts: SubscribeOptions = SubscribeOptions::default()
            .send_opts(send_opts)
            .close_on(Some(auto_close_opts));

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
                            return Err(Error::NotConnectedStatusChanged);
                        }
                    }
                    RelayNotification::Shutdown => break,
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
                        RelayNotification::Shutdown => break,
                        _ => (),
                    }
                }

                Ok(())
            })
            .await;
        }

        Ok(())
    }

    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let stored_events: Vec<Event> = self
            .database
            .query(filters.clone(), Order::Desc)
            .await
            .unwrap_or_default();
        let events: Mutex<BTreeSet<Event>> = Mutex::new(stored_events.into_iter().collect());
        self.get_events_of_with_callback(filters, timeout, opts, |event| async {
            let mut events = events.lock().await;
            events.insert(event);
        })
        .await?;
        Ok(events.into_inner().into_iter().rev().collect())
    }

    pub async fn count_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<usize, Error> {
        let id = SubscriptionId::generate();
        let send_opts = RelaySendOptions::default().skip_send_confirmation(true);
        self.send_msg(ClientMessage::count(id.clone(), filters), send_opts)
            .await?;

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
        self.send_msg(ClientMessage::close(id), send_opts).await?;

        Ok(count)
    }

    pub async fn reconcile(
        &self,
        filter: Filter,
        opts: NegentropyOptions,
    ) -> Result<Reconciliation, Error> {
        let items = self.database.negentropy_items(filter.clone()).await?;
        self.reconcile_with_items(filter, items, opts).await
    }

    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<Reconciliation, Error> {
        // Check if read option is disabled
        if !self.opts.flags.has_read() {
            return Err(Error::ReadDisabled);
        }

        // Check if relay is connected
        if !self.is_connected().await
            && self.stats.attempts() > MIN_ATTEMPTS
            && self.stats.uptime() < MIN_UPTIME
        {
            return Err(Error::NotConnected);
        }

        // Compose negentropy struct, add items and seal
        let mut negentropy = Negentropy::new(32, Some(20_000))?;
        for (id, timestamp) in items.into_iter() {
            let id = Bytes::from_slice(id.as_bytes());
            negentropy.add_item(timestamp.as_u64(), id)?;
        }
        negentropy.seal()?;

        // Send initial negentropy message
        let sub_id = SubscriptionId::generate();
        let send_opts = RelaySendOptions::default().skip_send_confirmation(true);
        let open_msg = ClientMessage::neg_open(&mut negentropy, &sub_id, filter)?;
        self.send_msg(open_msg, send_opts).await?;

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

            Ok::<(), Error>(())
        })
        .await
        .ok_or(Error::Timeout)??;

        let do_up: bool = opts.do_up();
        let do_down: bool = opts.do_down();
        let mut in_flight_up: HashSet<EventId> = HashSet::new();
        let mut in_flight_down: bool = false;
        let mut sync_done: bool = false;
        let mut have_ids: Vec<Bytes> = Vec::new();
        let mut need_ids: Vec<Bytes> = Vec::new();
        let down_sub_id: SubscriptionId = SubscriptionId::generate();

        let mut output: Reconciliation = Reconciliation::default();

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
                                let query: Bytes = Bytes::from_hex(message)?;
                                let msg: Option<Bytes> = negentropy.reconcile_with_ids(
                                    &query,
                                    &mut have_ids,
                                    &mut need_ids,
                                )?;

                                output.local.extend(
                                    have_ids.iter().filter_map(|b| EventId::from_slice(b).ok()),
                                );
                                output.remote.extend(
                                    need_ids.iter().filter_map(|b| EventId::from_slice(b).ok()),
                                );

                                if !do_up {
                                    have_ids.clear();
                                }

                                if !do_down {
                                    need_ids.clear();
                                }

                                match msg {
                                    Some(query) => {
                                        tracing::info!(
                                            "Continue negentropy reconciliation with {}",
                                            self.url
                                        );
                                        self.send_msg(
                                            ClientMessage::NegMsg {
                                                subscription_id: sub_id.clone(),
                                                message: query.to_hex(),
                                            },
                                            send_opts,
                                        )
                                        .await?;
                                    }
                                    None => sync_done = true,
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
                                        "Unable to upload event {event_id} to {}: {message}",
                                        self.url
                                    );

                                    output
                                        .send_failures
                                        .entry(self.url())
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
                                if let Ok(event_id) = EventId::from_slice(&id) {
                                    match self.database.event_by_id(event_id).await {
                                        Ok(event) => {
                                            in_flight_up.insert(event_id);
                                            self.send_msg(ClientMessage::event(event), send_opts)
                                                .await?;
                                            num_sent += 1;
                                        }
                                        Err(e) => tracing::error!(
                                            "Couldn't upload event to {}: {e}",
                                            self.url
                                        ),
                                    }
                                }
                            }
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
                        let mut ids: Vec<EventId> = Vec::with_capacity(NEGENTROPY_BATCH_SIZE_DOWN);

                        while !need_ids.is_empty() && ids.len() < NEGENTROPY_BATCH_SIZE_DOWN {
                            if let Some(id) = need_ids.pop() {
                                if let Ok(event_id) = EventId::from_slice(&id) {
                                    ids.push(event_id);
                                }
                            }
                        }

                        tracing::info!(
                            "Negentropy DOWN for '{}': {} events ({} remaining)",
                            self.url,
                            ids.len(),
                            need_ids.len()
                        );

                        let filter = Filter::new().ids(ids);
                        self.send_msg(
                            ClientMessage::req(down_sub_id.clone(), vec![filter]),
                            send_opts,
                        )
                        .await?;

                        in_flight_down = true
                    }
                }
                RelayNotification::RelayStatus { status } => {
                    if status.is_disconnected() {
                        return Err(Error::NotConnectedStatusChanged);
                    }
                }
                RelayNotification::Shutdown => break,
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

        // Close negentropy
        let close_msg = ClientMessage::NegClose {
            subscription_id: sub_id,
        };
        self.send_msg(close_msg, send_opts).await?;

        Ok(output)
    }

    pub async fn support_negentropy(&self) -> Result<bool, Error> {
        let pk = Keys::generate();
        let filter = Filter::new().author(pk.public_key());
        match self
            .reconcile_with_items(
                filter,
                Vec::new(),
                NegentropyOptions::new().initial_timeout(Duration::from_secs(5)),
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(Error::NegentropyNotSupported) => Ok(false),
            Err(e) => Err(e),
        }
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
