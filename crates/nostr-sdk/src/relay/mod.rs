// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Relay

#[cfg(not(target_arch = "wasm32"))]
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::fmt;
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use async_utility::futures_util::stream::AbortHandle;
use async_utility::{futures_util, thread, time};
use nostr::message::MessageHandleError;
#[cfg(feature = "nip11")]
use nostr::nips::nip11::RelayInformationDocument;
use nostr::{ClientMessage, Event, EventId, Filter, RelayMessage, SubscriptionId, Timestamp, Url};
use nostr_sdk_net::futures_util::{Future, SinkExt, StreamExt};
use nostr_sdk_net::{self as net, WsMessage};
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot, Mutex, RwLock};

mod options;
pub mod pool;

pub use self::options::{FilterOptions, RelayOptions, RelayPoolOptions, RelaySendOptions};
pub use self::pool::{RelayPoolMessage, RelayPoolNotification};
#[cfg(feature = "blocking")]
use crate::RUNTIME;

type Message = (RelayEvent, Option<oneshot::Sender<bool>>);

/// [`Relay`] error
#[derive(Debug, Error)]
pub enum Error {
    /// Channel timeout
    #[error("channel timeout")]
    ChannelTimeout,
    /// Message response timeout
    #[error("recv message response timeout")]
    RecvTimeout,
    /// Generic timeout
    #[error("timeout")]
    Timeout,
    /// Message not sent
    #[error("message not sent")]
    MessageNotSent,
    /// Event not published
    #[error("event not published: {0}")]
    EventNotPublished(String),
    /// No event is published
    #[error("events not published: {0:?}")]
    EventsNotPublished(HashMap<EventId, String>),
    /// Only some events
    #[error("partial publish: published={}, missing={}", published.len(), not_published.len())]
    PartialPublish {
        /// Published events
        published: Vec<EventId>,
        /// Not published events
        not_published: HashMap<EventId, String>,
    },
    /// Loop terminated
    #[error("loop terminated")]
    LoopTerminated,
    /// Batch event empty
    #[error("batch event cannot be empty")]
    BatchEventEmpty,
    /// Impossible to receive oneshot message
    #[error("impossible to recv msg")]
    OneShotRecvError,
    /// Read actions disabled
    #[error("read actions are disabled for this relay")]
    ReadDisabled,
    /// Write actions disabled
    #[error("write actions are disabled for this relay")]
    WriteDisabled,
    /// Subscription internal ID not found
    #[error("internal ID not found")]
    InternalIdNotFound,
    /// Filters empty
    #[error("filters empty")]
    FiltersEmpty,
}

/// Relay connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayStatus {
    /// Relay initialized
    Initialized,
    /// Connecting
    Connecting,
    /// Relay connected
    Connected,
    /// Relay disconnected, will retry to connect again
    Disconnected,
    /// Stop
    Stopped,
    /// Relay completely disconnected
    Terminated,
}

impl fmt::Display for RelayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialized => write!(f, "Initialized"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

/// Relay event
#[derive(Debug)]
pub enum RelayEvent {
    /// Send [`ClientMessage`]
    SendMsg(Box<ClientMessage>),
    /// Send multiple messages at once
    Batch(Vec<ClientMessage>),
    /// Ping
    #[cfg(not(target_arch = "wasm32"))]
    Ping {
        /// Nonce
        nonce: u64,
    },
    /// Close
    Close,
    /// Stop
    Stop,
    /// Completely disconnect
    Terminate,
}

/// Ping Stats
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct PingStats {
    sent_at: Arc<RwLock<Instant>>,
    last_nonce: Arc<AtomicU64>,
    replied: Arc<AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for PingStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PingStats {
    /// New default ping stats
    pub fn new() -> Self {
        Self {
            sent_at: Arc::new(RwLock::new(Instant::now())),
            last_nonce: Arc::new(AtomicU64::new(0)),
            replied: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get sent at
    pub async fn sent_at(&self) -> Instant {
        *self.sent_at.read().await
    }

    /// Last nonce
    pub fn last_nonce(&self) -> u64 {
        self.last_nonce.load(Ordering::SeqCst)
    }

    /// Replied
    pub fn replied(&self) -> bool {
        self.replied.load(Ordering::SeqCst)
    }

    pub(crate) fn reset(&self) {
        self.set_last_nonce(0);
        self.set_replied(false);
    }

    pub(crate) async fn just_sent(&self) {
        let mut sent_at = self.sent_at.write().await;
        *sent_at = Instant::now();
    }

    pub(crate) fn set_last_nonce(&self, nonce: u64) {
        let _ = self
            .last_nonce
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(nonce));
    }

    pub(crate) fn set_replied(&self, replied: bool) {
        let _ = self
            .replied
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(replied));
    }
}

/// [`Relay`] connection stats
#[derive(Debug, Clone)]
pub struct RelayConnectionStats {
    attempts: Arc<AtomicUsize>,
    success: Arc<AtomicUsize>,
    bytes_sent: Arc<AtomicUsize>,
    bytes_received: Arc<AtomicUsize>,
    connected_at: Arc<AtomicU64>,
    #[cfg(not(target_arch = "wasm32"))]
    latencies: Arc<RwLock<VecDeque<Duration>>>,
    #[cfg(not(target_arch = "wasm32"))]
    ping: PingStats,
}

impl Default for RelayConnectionStats {
    fn default() -> Self {
        Self::new()
    }
}

impl RelayConnectionStats {
    /// New connections stats
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(AtomicUsize::new(0)),
            success: Arc::new(AtomicUsize::new(0)),
            bytes_sent: Arc::new(AtomicUsize::new(0)),
            bytes_received: Arc::new(AtomicUsize::new(0)),
            connected_at: Arc::new(AtomicU64::new(0)),
            #[cfg(not(target_arch = "wasm32"))]
            latencies: Arc::new(RwLock::new(VecDeque::new())),
            #[cfg(not(target_arch = "wasm32"))]
            ping: PingStats::default(),
        }
    }

    /// The number of times a connection has been attempted
    pub fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    /// The number of times a connection has been successfully established
    pub fn success(&self) -> usize {
        self.success.load(Ordering::SeqCst)
    }

    /// Bytes sent
    pub fn bytes_sent(&self) -> usize {
        self.bytes_sent.load(Ordering::SeqCst)
    }

    /// Bytes received
    pub fn bytes_received(&self) -> usize {
        self.bytes_received.load(Ordering::SeqCst)
    }

    /// Get the UNIX timestamp of the last started connection
    pub fn connected_at(&self) -> Timestamp {
        Timestamp::from(self.connected_at.load(Ordering::SeqCst))
    }

    /// Calculate latency
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn latency(&self) -> Option<Duration> {
        let latencies = self.latencies.read().await;
        let sum: Duration = latencies.iter().sum();
        sum.checked_div(latencies.len() as u32)
    }

    /// Calculate latency
    #[cfg(all(not(target_arch = "wasm32"), feature = "blocking"))]
    pub fn latency_blocking(&self) -> Option<Duration> {
        RUNTIME.block_on(async { self.latency().await })
    }

    pub(crate) fn new_attempt(&self) {
        self.attempts.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn new_success(&self) {
        self.success.fetch_add(1, Ordering::SeqCst);
        let _ = self
            .connected_at
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| {
                Some(Timestamp::now().as_u64())
            });
    }

    pub(crate) fn add_bytes_sent(&self, size: usize) {
        self.bytes_sent.fetch_add(size, Ordering::SeqCst);
    }

    pub(crate) fn add_bytes_received(&self, size: usize) {
        self.bytes_received.fetch_add(size, Ordering::SeqCst);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn save_latency(&self, latency: Duration) {
        let mut latencies = self.latencies.write().await;
        if latencies.len() >= 5 {
            latencies.pop_back();
        }
        latencies.push_front(latency)
    }
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
            c => Self::Custom(c.to_string()),
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
    url: Url,
    #[cfg(not(target_arch = "wasm32"))]
    proxy: Option<SocketAddr>,
    status: Arc<RwLock<RelayStatus>>,
    #[cfg(feature = "nip11")]
    document: Arc<RwLock<RelayInformationDocument>>,
    opts: RelayOptions,
    stats: RelayConnectionStats,
    auto_connect_loop_running: Arc<AtomicBool>,
    scheduled_for_stop: Arc<AtomicBool>,
    scheduled_for_termination: Arc<AtomicBool>,
    pool_sender: Sender<RelayPoolMessage>,
    relay_sender: Sender<Message>,
    relay_receiver: Arc<Mutex<Receiver<Message>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    subscriptions: Arc<RwLock<HashMap<InternalSubscriptionId, ActiveSubscription>>>,
}

impl PartialEq for Relay {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Relay {
    /// Create new `Relay`
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        url: Url,
        pool_sender: Sender<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
        proxy: Option<SocketAddr>,
        opts: RelayOptions,
    ) -> Self {
        let (relay_sender, relay_receiver) = mpsc::channel::<Message>(1024);

        Self {
            url,
            proxy,
            status: Arc::new(RwLock::new(RelayStatus::Initialized)),
            #[cfg(feature = "nip11")]
            document: Arc::new(RwLock::new(RelayInformationDocument::new())),
            opts,
            stats: RelayConnectionStats::new(),
            auto_connect_loop_running: Arc::new(AtomicBool::new(false)),
            scheduled_for_stop: Arc::new(AtomicBool::new(false)),
            scheduled_for_termination: Arc::new(AtomicBool::new(false)),
            pool_sender,
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
            notification_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new `Relay`
    #[cfg(target_arch = "wasm32")]
    pub fn new(
        url: Url,
        pool_sender: Sender<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
        opts: RelayOptions,
    ) -> Self {
        let (relay_sender, relay_receiver) = mpsc::channel::<Message>(1024);

        Self {
            url,
            status: Arc::new(RwLock::new(RelayStatus::Initialized)),
            #[cfg(feature = "nip11")]
            document: Arc::new(RwLock::new(RelayInformationDocument::new())),
            opts,
            stats: RelayConnectionStats::new(),
            auto_connect_loop_running: Arc::new(AtomicBool::new(false)),
            scheduled_for_stop: Arc::new(AtomicBool::new(false)),
            scheduled_for_termination: Arc::new(AtomicBool::new(false)),
            pool_sender,
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
            notification_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get relay url
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    /// Get proxy
    #[cfg(not(target_arch = "wasm32"))]
    pub fn proxy(&self) -> Option<SocketAddr> {
        self.proxy
    }

    /// Get [`RelayStatus`]
    pub async fn status(&self) -> RelayStatus {
        let status = self.status.read().await;
        *status
    }

    /// Get [`RelayStatus`]
    #[cfg(feature = "blocking")]
    pub fn status_blocking(&self) -> RelayStatus {
        RUNTIME.block_on(async { self.status().await })
    }

    async fn set_status(&self, status: RelayStatus) {
        // Change status
        let mut s = self.status.write().await;
        *s = status;

        // Send notification
        if let Err(e) = self.pool_sender.try_send(RelayPoolMessage::RelayStatus {
            url: self.url(),
            status,
        }) {
            tracing::error!("Impossible to send RelayPoolMessage::RelayStatus message: {e}");
        }
    }

    /// Check if [`Relay`] is connected
    pub async fn is_connected(&self) -> bool {
        self.status().await == RelayStatus::Connected
    }

    /// Get [`RelayInformationDocument`]
    #[cfg(feature = "nip11")]
    pub async fn document(&self) -> RelayInformationDocument {
        let document = self.document.read().await;
        document.clone()
    }

    /// Get [`RelayInformationDocument`]
    #[cfg(all(feature = "nip11", feature = "blocking"))]
    pub fn document_blocking(&self) -> RelayInformationDocument {
        RUNTIME.block_on(async { self.document().await })
    }

    #[cfg(feature = "nip11")]
    async fn set_document(&self, document: RelayInformationDocument) {
        let mut d = self.document.write().await;
        *d = document;
    }

    /// Get [`ActiveSubscription`]
    pub async fn subscriptions(&self) -> HashMap<InternalSubscriptionId, ActiveSubscription> {
        let subscription = self.subscriptions.read().await;
        subscription.clone()
    }

    /// Update [`ActiveSubscription`]
    pub async fn update_subscription_filters(
        &self,
        internal_id: InternalSubscriptionId,
        filters: Vec<Filter>,
    ) {
        let mut s = self.subscriptions.write().await;
        s.entry(internal_id)
            .and_modify(|sub| sub.filters = filters.clone())
            .or_insert_with(|| ActiveSubscription::with_filters(filters));
    }

    /// Get [`RelayOptions`]
    pub fn opts(&self) -> RelayOptions {
        self.opts.clone()
    }

    /// Get [`RelayConnectionStats`]
    pub fn stats(&self) -> RelayConnectionStats {
        self.stats.clone()
    }

    /// Get queue len
    pub fn queue(&self) -> usize {
        self.relay_sender.max_capacity() - self.relay_sender.capacity()
    }

    fn is_auto_connect_loop_running(&self) -> bool {
        self.auto_connect_loop_running.load(Ordering::SeqCst)
    }

    fn set_auto_connect_loop_running(&self, value: bool) {
        let _ =
            self.auto_connect_loop_running
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(value));
    }

    fn is_scheduled_for_stop(&self) -> bool {
        self.scheduled_for_stop.load(Ordering::SeqCst)
    }

    fn schedule_for_stop(&self, value: bool) {
        let _ = self
            .scheduled_for_stop
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(value));
    }

    fn is_scheduled_for_termination(&self) -> bool {
        self.scheduled_for_termination.load(Ordering::SeqCst)
    }

    fn schedule_for_termination(&self, value: bool) {
        let _ =
            self.scheduled_for_termination
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(value));
    }

    /// Connect to relay and keep alive connection
    pub async fn connect(&self, wait_for_connection: bool) {
        self.schedule_for_stop(false);
        self.schedule_for_termination(false);

        if wait_for_connection {
            if let RelayStatus::Initialized | RelayStatus::Stopped | RelayStatus::Terminated =
                self.status().await
            {
                self.try_connect().await
            }
        }

        if !self.is_auto_connect_loop_running() {
            self.set_auto_connect_loop_running(true);

            tracing::debug!("Auto connect loop started for {}", self.url);

            if !wait_for_connection {
                self.set_status(RelayStatus::Initialized).await;
            }

            let relay = self.clone();
            thread::spawn(async move {
                loop {
                    let queue = relay.queue();
                    if queue > 0 {
                        tracing::info!(
                            "{} messages queued for {} (capacity: {})",
                            queue,
                            relay.url(),
                            relay.relay_sender.capacity()
                        );
                    }

                    // Schedule relay for termination
                    // Needed to terminate the auto reconnect loop, also if the relay is not connected yet.
                    if relay.is_scheduled_for_stop() {
                        relay.set_status(RelayStatus::Stopped).await;
                        relay.schedule_for_stop(false);
                        tracing::debug!(
                            "Auto connect loop terminated for {} [stop - schedule]",
                            relay.url
                        );
                        break;
                    } else if relay.is_scheduled_for_termination() {
                        relay.set_status(RelayStatus::Terminated).await;
                        relay.schedule_for_termination(false);
                        tracing::debug!(
                            "Auto connect loop terminated for {} [schedule]",
                            relay.url
                        );
                        break;
                    }

                    // Check status
                    match relay.status().await {
                        RelayStatus::Initialized | RelayStatus::Disconnected => {
                            relay.try_connect().await
                        }
                        RelayStatus::Stopped | RelayStatus::Terminated => {
                            tracing::debug!("Auto connect loop terminated for {}", relay.url);
                            break;
                        }
                        _ => (),
                    };

                    thread::sleep(Duration::from_secs(10)).await;
                }

                relay.set_auto_connect_loop_running(false);
            });
        } else {
            tracing::warn!("Auto connect loop for {} is already running!", self.url)
        }
    }

    async fn try_connect(&self) {
        self.stats.new_attempt();

        let url: String = self.url.to_string();

        // Set RelayStatus to `Connecting`
        self.set_status(RelayStatus::Connecting).await;
        tracing::debug!("Connecting to {}", url);

        // Request `RelayInformationDocument`
        #[cfg(feature = "nip11")]
        {
            let relay = self.clone();
            thread::spawn(async move {
                #[cfg(not(target_arch = "wasm32"))]
                let document = RelayInformationDocument::get(relay.url(), relay.proxy()).await;
                #[cfg(target_arch = "wasm32")]
                let document = RelayInformationDocument::get(relay.url()).await;

                match document {
                    Ok(document) => relay.set_document(document).await,
                    Err(e) => tracing::error!(
                        "Impossible to get information document from {}: {}",
                        relay.url,
                        e
                    ),
                };
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        let connection = net::native::connect(&self.url, self.proxy, None).await;
        #[cfg(target_arch = "wasm32")]
        let connection = net::wasm::connect(&self.url).await;

        // Connect
        match connection {
            Ok((mut ws_tx, mut ws_rx)) => {
                self.set_status(RelayStatus::Connected).await;
                tracing::info!("Connected to {}", url);

                self.stats.new_success();

                #[cfg(not(target_arch = "wasm32"))]
                let ping_abort_handle: AbortHandle = {
                    let relay = self.clone();
                    thread::abortable(async move {
                        tracing::debug!("Relay Ping Thread Started");

                        loop {
                            if relay.stats.ping.last_nonce() != 0 && !relay.stats.ping.replied() {
                                tracing::warn!("{} not replied to ping", relay.url);
                                relay.stats.ping.reset();
                                break;
                            }

                            let nonce: u64 = nostr::secp256k1::rand::random();
                            relay.stats.ping.set_last_nonce(nonce);
                            relay.stats.ping.set_replied(false);
                            if let Err(e) = relay.send_relay_event(RelayEvent::Ping { nonce }, None)
                            {
                                tracing::error!("Impossible to ping {}: {e}", relay.url);
                                break;
                            };
                            thread::sleep(Duration::from_secs(60)).await;
                        }

                        tracing::debug!("Exited from Ping Thread of {}", relay.url);

                        if let Err(err) = relay.disconnect().await {
                            tracing::error!("Impossible to disconnect {}: {}", relay.url, err);
                        }
                    })
                };

                let relay = self.clone();
                thread::spawn(async move {
                    tracing::debug!("Relay Event Thread Started");
                    let mut rx = relay.relay_receiver.lock().await;
                    while let Some((relay_event, oneshot_sender)) = rx.recv().await {
                        match relay_event {
                            RelayEvent::SendMsg(msg) => {
                                let json = msg.as_json();
                                let size: usize = json.as_bytes().len();
                                tracing::debug!(
                                    "Sending {json} to {} (size: {size} bytes)",
                                    relay.url
                                );
                                match ws_tx.send(WsMessage::Text(json)).await {
                                    Ok(_) => {
                                        relay.stats.add_bytes_sent(size);
                                        if let Some(sender) = oneshot_sender {
                                            if let Err(e) = sender.send(true) {
                                                tracing::error!(
                                                    "Impossible to send oneshot msg: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Impossible to send msg to {}: {}",
                                            relay.url(),
                                            e.to_string()
                                        );
                                        if let Some(sender) = oneshot_sender {
                                            if let Err(e) = sender.send(false) {
                                                tracing::error!(
                                                    "Impossible to send oneshot msg: {}",
                                                    e
                                                );
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                            RelayEvent::Batch(msgs) => {
                                let len = msgs.len();
                                let size: usize =
                                    msgs.iter().map(|msg| msg.as_json().as_bytes().len()).sum();
                                tracing::debug!(
                                    "Sending {len} messages to {} (size: {size} bytes)",
                                    relay.url
                                );
                                let msgs = msgs
                                    .into_iter()
                                    .map(|msg| Ok(WsMessage::Text(msg.as_json())));
                                let mut stream = futures_util::stream::iter(msgs);
                                match ws_tx.send_all(&mut stream).await {
                                    Ok(_) => {
                                        relay.stats.add_bytes_sent(size);
                                        if let Some(sender) = oneshot_sender {
                                            if let Err(e) = sender.send(true) {
                                                tracing::error!(
                                                    "Impossible to send oneshot msg: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Impossible to send {len} messages to {}: {}",
                                            relay.url(),
                                            e.to_string()
                                        );
                                        if let Some(sender) = oneshot_sender {
                                            if let Err(e) = sender.send(false) {
                                                tracing::error!(
                                                    "Impossible to send oneshot msg: {}",
                                                    e
                                                );
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            RelayEvent::Ping { nonce } => {
                                match ws_tx
                                    .send(WsMessage::Ping(nonce.to_string().as_bytes().to_vec()))
                                    .await
                                {
                                    Ok(_) => {
                                        relay.stats.ping.just_sent().await;
                                        tracing::debug!("Ping {} (nonce {})", relay.url, nonce);
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Impossible to ping {}: {}",
                                            relay.url(),
                                            e.to_string()
                                        );
                                    }
                                }
                            }
                            RelayEvent::Close => {
                                let _ = ws_tx.close().await;
                                relay.set_status(RelayStatus::Disconnected).await;
                                tracing::info!("Disconnected from {}", url);
                                break;
                            }
                            RelayEvent::Stop => {
                                if relay.is_scheduled_for_stop() {
                                    let _ = ws_tx.close().await;
                                    relay.set_status(RelayStatus::Stopped).await;
                                    relay.schedule_for_stop(false);
                                    tracing::info!("Stopped {}", url);
                                    break;
                                }
                            }
                            RelayEvent::Terminate => {
                                if relay.is_scheduled_for_termination() {
                                    let _ = ws_tx.close().await;
                                    relay.set_status(RelayStatus::Terminated).await;
                                    relay.schedule_for_termination(false);
                                    tracing::info!("Completely disconnected from {}", url);
                                    break;
                                }
                            }
                        }
                    }

                    tracing::debug!("Exited from Relay Event Thread");

                    #[cfg(not(target_arch = "wasm32"))]
                    ping_abort_handle.abort();
                });

                let relay = self.clone();
                thread::spawn(async move {
                    tracing::debug!("Relay Message Thread Started");

                    async fn func(relay: &Relay, data: Vec<u8>) -> bool {
                        relay.stats.add_bytes_received(data.len());
                        match String::from_utf8(data) {
                            Ok(data) => match RelayMessage::from_json(&data) {
                                Ok(msg) => {
                                    tracing::trace!("Received message to {}: {:?}", relay.url, msg);
                                    if let Err(err) = relay
                                        .pool_sender
                                        .send(RelayPoolMessage::ReceivedMsg {
                                            relay_url: relay.url(),
                                            msg,
                                        })
                                        .await
                                    {
                                        tracing::error!(
                                            "Impossible to send ReceivedMsg to pool: {}",
                                            &err
                                        );
                                        return true; // Exit
                                    };
                                }
                                Err(e) => {
                                    match e {
                                        MessageHandleError::EmptyMsg => (),
                                        _ => tracing::error!("{e}: {data}"),
                                    };
                                }
                            },
                            Err(err) => tracing::error!("{}", err),
                        }

                        false
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    while let Some(msg_res) = ws_rx.next().await {
                        if let Ok(msg) = msg_res {
                            match msg {
                                WsMessage::Pong(bytes) => match String::from_utf8(bytes) {
                                    Ok(nonce) => match nonce.parse::<u64>() {
                                        Ok(nonce) => {
                                            if relay.stats.ping.last_nonce() == nonce {
                                                tracing::debug!(
                                                    "Pong from {} match nonce: {}",
                                                    relay.url,
                                                    nonce
                                                );
                                                relay.stats.ping.set_replied(true);
                                                let sent_at = relay.stats.ping.sent_at().await;
                                                relay.stats.save_latency(sent_at.elapsed()).await;
                                            } else {
                                                tracing::error!("Pong nonce not match: received={nonce}, expected={}", relay.stats.ping.last_nonce());
                                            }
                                        }
                                        Err(e) => tracing::error!("{e}"),
                                    },
                                    Err(e) => tracing::error!("{e}"),
                                },
                                _ => {
                                    let data: Vec<u8> = msg.into_data();
                                    let exit: bool = func(&relay, data).await;
                                    if exit {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    #[cfg(target_arch = "wasm32")]
                    while let Some(msg) = ws_rx.next().await {
                        let data: Vec<u8> = msg.as_ref().to_vec();
                        let exit: bool = func(&relay, data).await;
                        if exit {
                            break;
                        }
                    }

                    tracing::debug!("Exited from Message Thread of {}", relay.url);

                    if let Err(err) = relay.disconnect().await {
                        tracing::error!("Impossible to disconnect {}: {}", relay.url, err);
                    }
                });

                // Subscribe to relay
                if self.opts.read() {
                    if let Err(e) = self.resubscribe_all(None).await {
                        tracing::error!(
                            "Impossible to subscribe to {}: {}",
                            self.url(),
                            e.to_string()
                        )
                    }
                }
            }
            Err(err) => {
                self.set_status(RelayStatus::Disconnected).await;
                tracing::error!("Impossible to connect to {}: {}", url, err);
            }
        };
    }

    fn send_relay_event(
        &self,
        relay_msg: RelayEvent,
        sender: Option<oneshot::Sender<bool>>,
    ) -> Result<(), Error> {
        self.relay_sender
            .try_send((relay_msg, sender))
            .map_err(|_| Error::MessageNotSent)
    }

    /// Disconnect from relay and set status to 'Disconnected'
    async fn disconnect(&self) -> Result<(), Error> {
        let status = self.status().await;
        if status.ne(&RelayStatus::Disconnected)
            && status.ne(&RelayStatus::Stopped)
            && status.ne(&RelayStatus::Terminated)
        {
            self.send_relay_event(RelayEvent::Close, None)?;
        }
        Ok(())
    }

    /// Disconnect from relay and set status to 'Stopped'
    pub async fn stop(&self) -> Result<(), Error> {
        self.schedule_for_stop(true);
        let status = self.status().await;
        if status.ne(&RelayStatus::Disconnected)
            && status.ne(&RelayStatus::Stopped)
            && status.ne(&RelayStatus::Terminated)
        {
            self.send_relay_event(RelayEvent::Stop, None)?;
        }
        Ok(())
    }

    /// Disconnect from relay and set status to 'Terminated'
    pub async fn terminate(&self) -> Result<(), Error> {
        self.schedule_for_termination(true);
        let status = self.status().await;
        if status.ne(&RelayStatus::Disconnected)
            && status.ne(&RelayStatus::Stopped)
            && status.ne(&RelayStatus::Terminated)
        {
            self.send_relay_event(RelayEvent::Terminate, None)?;
        }
        Ok(())
    }

    /// Send msg to relay
    pub async fn send_msg(&self, msg: ClientMessage, wait: Option<Duration>) -> Result<(), Error> {
        if !self.opts.write() {
            if let ClientMessage::Event(_) = msg {
                return Err(Error::WriteDisabled);
            }
        }

        if !self.opts.read() {
            if let ClientMessage::Req { .. } | ClientMessage::Close(_) = msg {
                return Err(Error::ReadDisabled);
            }
        }

        match wait {
            Some(timeout) => {
                let (tx, rx) = oneshot::channel::<bool>();
                self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)), Some(tx))?;
                match time::timeout(Some(timeout), rx).await {
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
                    _ => Err(Error::RecvTimeout),
                }
            }
            None => self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)), None),
        }
    }

    /// Send multiple [`ClientMessage`] at once
    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        wait: Option<Duration>,
    ) -> Result<(), Error> {
        if !self.opts.write() && msgs.iter().any(|msg| msg.is_event()) {
            return Err(Error::WriteDisabled);
        }

        if !self.opts.read() && msgs.iter().any(|msg| msg.is_req() || msg.is_close()) {
            return Err(Error::ReadDisabled);
        }

        match wait {
            Some(timeout) => {
                let (tx, rx) = oneshot::channel::<bool>();
                self.send_relay_event(RelayEvent::Batch(msgs), Some(tx))?;
                match time::timeout(Some(timeout), rx).await {
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
                    _ => Err(Error::RecvTimeout),
                }
            }
            None => self.send_relay_event(RelayEvent::Batch(msgs), None),
        }
    }

    /// Send event and wait for `OK` relay msg
    pub async fn send_event(&self, event: Event, opts: RelaySendOptions) -> Result<EventId, Error> {
        let id: EventId = event.id;

        if opts.skip_disconnected && !self.is_connected().await && self.stats.attempts() > 1 {
            return Err(Error::EventNotPublished(String::from(
                "relay not connected",
            )));
        }

        if event.is_ephemeral() {
            tracing::warn!(
                "Trying to send an ephemeral event: skip the wait for `OK` msg from relay."
            );
            self.send_msg(ClientMessage::new_event(event), Some(opts.timeout))
                .await?;
            Ok(id)
        } else {
            time::timeout(Some(opts.timeout), async {
                self.send_msg(ClientMessage::new_event(event), None).await?;
                let mut notifications = self.notification_sender.subscribe();
                while let Ok(notification) = notifications.recv().await {
                    match notification {
                        RelayPoolNotification::Message(
                            url,
                            RelayMessage::Ok {
                                event_id,
                                status,
                                message,
                            },
                        ) => {
                            if self.url == url && id == event_id {
                                if status {
                                    return Ok(event_id);
                                } else {
                                    return Err(Error::EventNotPublished(message));
                                }
                            }
                        }
                        RelayPoolNotification::RelayStatus { url, status } => {
                            if opts.skip_disconnected && url == self.url {
                                if let RelayStatus::Disconnected
                                | RelayStatus::Stopped
                                | RelayStatus::Terminated = status
                                {
                                    return Err(Error::EventNotPublished(String::from(
                                        "relay not connected (status changed)",
                                    )));
                                }
                            }
                        }
                        _ => (),
                    }
                }
                Err(Error::LoopTerminated)
            })
            .await
            .ok_or(Error::Timeout)?
        }
    }

    /// Send multiple [`Event`] at once
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        if events.is_empty() {
            return Err(Error::BatchEventEmpty);
        }

        if opts.skip_disconnected && !self.is_connected().await && self.stats.attempts() > 1 {
            return Err(Error::EventNotPublished(String::from(
                "relay not connected",
            )));
        }

        let mut msgs: Vec<ClientMessage> = Vec::with_capacity(events.len());
        let mut missing: HashSet<EventId> = HashSet::new();

        for event in events.into_iter() {
            if !event.is_ephemeral() {
                missing.insert(event.id);
            } else {
                tracing::warn!(
                    "Trying to batch ephemeral event: skip the wait for `OK` msg from relay."
                );
            }
            msgs.push(ClientMessage::new_event(event));
        }

        time::timeout(Some(opts.timeout), async {
            self.batch_msg(msgs, None).await?;
            let mut published: HashSet<EventId> = HashSet::new();
            let mut not_published: HashMap<EventId, String> = HashMap::new();
            let mut notifications = self.notification_sender.subscribe();
            while let Ok(notification) = notifications.recv().await {
                match notification {
                    RelayPoolNotification::Message(
                        url,
                        RelayMessage::Ok {
                            event_id,
                            status,
                            message,
                        },
                    ) => {
                        if self.url == url && missing.remove(&event_id) {
                            if status {
                                published.insert(event_id);
                            } else {
                                not_published.insert(event_id, message);
                            }
                        }
                    }
                    RelayPoolNotification::RelayStatus { url, status } => {
                        if opts.skip_disconnected && url == self.url {
                            if let RelayStatus::Disconnected
                            | RelayStatus::Stopped
                            | RelayStatus::Terminated = status
                            {
                                return Err(Error::EventNotPublished(String::from(
                                    "relay not connected (status changed)",
                                )));
                            }
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

    /// Subscribes relay with existing filter
    async fn resubscribe_all(&self, wait: Option<Duration>) -> Result<(), Error> {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let subscriptions = self.subscriptions().await;

        for (internal_id, sub) in subscriptions.into_iter() {
            if !sub.filters.is_empty() {
                self.send_msg(ClientMessage::new_req(sub.id.clone(), sub.filters), wait)
                    .await?;
            } else {
                tracing::warn!("Subscription '{internal_id}' has empty filters");
            }
        }

        Ok(())
    }

    async fn resubscribe(
        &self,
        internal_id: InternalSubscriptionId,
        wait: Option<Duration>,
    ) -> Result<(), Error> {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let subscriptions = self.subscriptions().await;
        let sub = subscriptions
            .get(&internal_id)
            .ok_or(Error::InternalIdNotFound)?;

        self.send_msg(
            ClientMessage::new_req(sub.id.clone(), sub.filters.clone()),
            wait,
        )
        .await?;

        Ok(())
    }

    /// Subscribe to filter with internal ID set to `InternalSubscriptionId::Default`
    pub async fn subscribe(
        &self,
        filters: Vec<Filter>,
        wait: Option<Duration>,
    ) -> Result<(), Error> {
        self.subscribe_with_internal_id(InternalSubscriptionId::Default, filters, wait)
            .await
    }

    /// Subscribe with custom internal ID
    pub async fn subscribe_with_internal_id(
        &self,
        internal_id: InternalSubscriptionId,
        filters: Vec<Filter>,
        wait: Option<Duration>,
    ) -> Result<(), Error> {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        if filters.is_empty() {
            return Err(Error::FiltersEmpty);
        }

        self.update_subscription_filters(internal_id.clone(), filters)
            .await;
        self.resubscribe(internal_id, wait).await
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self, wait: Option<Duration>) -> Result<(), Error> {
        self.unsubscribe_with_internal_id(InternalSubscriptionId::Default, wait)
            .await
    }

    /// Unsubscribe with custom internal id
    pub async fn unsubscribe_with_internal_id(
        &self,
        internal_id: InternalSubscriptionId,
        wait: Option<Duration>,
    ) -> Result<(), Error> {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let mut subscriptions = self.subscriptions().await;
        let subscription = subscriptions
            .remove(&internal_id)
            .ok_or(Error::InternalIdNotFound)?;
        self.send_msg(ClientMessage::close(subscription.id), wait)
            .await?;
        Ok(())
    }

    /// Unsubscribe from all subscriptions
    pub async fn unsubscribe_all(&self, wait: Option<Duration>) -> Result<(), Error> {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let subscriptions = self.subscriptions().await;

        for sub in subscriptions.into_values() {
            self.send_msg(ClientMessage::close(sub.id.clone()), wait)
                .await?;
        }

        Ok(())
    }

    async fn handle_events_of<F>(
        &self,
        id: SubscriptionId,
        timeout: Duration,
        opts: FilterOptions,
        callback: impl Fn(Event) -> F,
    ) -> Result<(), Error>
    where
        F: Future<Output = ()>,
    {
        let mut counter = 0;
        let mut received_eose: bool = false;

        let mut notifications = self.notification_sender.subscribe();
        time::timeout(Some(timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Message(_, msg) = notification {
                    match msg {
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
                        _ => {
                            tracing::debug!("Receive unhandled message {msg:?} from {}", self.url)
                        }
                    };
                }
            }
        })
        .await
        .ok_or(Error::Timeout)?;

        if let FilterOptions::WaitDurationAfterEOSE(duration) = opts {
            time::timeout(Some(duration), async {
                while let Ok(notification) = notifications.recv().await {
                    if let RelayPoolNotification::Message(
                        _,
                        RelayMessage::Event {
                            subscription_id,
                            event,
                        },
                    ) = notification
                    {
                        if subscription_id.eq(&id) {
                            callback(*event).await;
                        }
                    }
                }
            })
            .await;
        }

        Ok(())
    }

    /// Get events of filters with custom callback
    pub async fn get_events_of_with_callback<F>(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
        callback: impl Fn(Event) -> F,
    ) -> Result<(), Error>
    where
        F: Future<Output = ()>,
    {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let id = SubscriptionId::generate();

        self.send_msg(ClientMessage::new_req(id.clone(), filters), None)
            .await?;

        self.handle_events_of(id.clone(), timeout, opts, callback)
            .await?;

        // Unsubscribe
        self.send_msg(ClientMessage::close(id), None).await?;

        Ok(())
    }

    /// Get events of filters
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let events: Mutex<Vec<Event>> = Mutex::new(Vec::new());
        self.get_events_of_with_callback(filters, timeout, opts, |event| async {
            let mut events = events.lock().await;
            events.push(event);
        })
        .await?;
        Ok(events.into_inner())
    }

    /// Request events of filter. All events will be sent to notification listener,
    /// until the EOSE "end of stored events" message is received from the relay.
    pub fn req_events_of(&self, filters: Vec<Filter>, timeout: Duration, opts: FilterOptions) {
        if !self.opts.read() {
            tracing::error!("{}", Error::ReadDisabled);
        }

        let relay = self.clone();
        thread::spawn(async move {
            let id = SubscriptionId::generate();

            // Subscribe
            if let Err(e) = relay
                .send_msg(ClientMessage::new_req(id.clone(), filters), None)
                .await
            {
                tracing::error!(
                    "Impossible to send REQ to {}: {}",
                    relay.url(),
                    e.to_string()
                );
            };

            if let Err(e) = relay
                .handle_events_of(id.clone(), timeout, opts, |_| async {})
                .await
            {
                tracing::error!("{e}");
            }

            // Unsubscribe
            if let Err(e) = relay.send_msg(ClientMessage::close(id), None).await {
                tracing::error!(
                    "Impossible to close subscription with {}: {}",
                    relay.url(),
                    e.to_string()
                );
            }
        });
    }
}
