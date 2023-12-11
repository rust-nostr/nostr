// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay

use std::collections::{HashMap, HashSet};
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::ops::Mul;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{cmp, fmt};

#[cfg(not(target_arch = "wasm32"))]
use async_utility::futures_util::stream::AbortHandle;
use async_utility::{futures_util, thread, time};
use nostr::message::relay::NegentropyErrorCode;
use nostr::message::MessageHandleError;
use nostr::negentropy::{self, Bytes, Negentropy};
#[cfg(feature = "nip11")]
use nostr::nips::nip11::RelayInformationDocument;
use nostr::secp256k1::rand::{self, Rng};
use nostr::{
    ClientMessage, Event, EventId, Filter, JsonUtil, Keys, RawRelayMessage, RelayMessage,
    SubscriptionId, Timestamp, Url,
};
use nostr_database::{DatabaseError, DynNostrDatabase};
use nostr_sdk_net::futures_util::{Future, SinkExt, StreamExt};
use nostr_sdk_net::{self as net, WsMessage};
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot, Mutex, RwLock};

pub mod limits;
mod options;
pub mod pool;
mod stats;

pub use self::limits::Limits;
pub use self::options::{
    FilterOptions, NegentropyOptions, RelayOptions, RelayPoolOptions, RelaySendOptions,
};
use self::options::{MAX_ADJ_RETRY_SEC, MIN_RETRY_SEC};
pub use self::pool::{RelayPoolMessage, RelayPoolNotification};
pub use self::stats::RelayConnectionStats;
#[cfg(feature = "blocking")]
use crate::RUNTIME;

type Message = (RelayEvent, Option<oneshot::Sender<bool>>);

const MIN_UPTIME: f64 = 0.90;
#[cfg(not(target_arch = "wasm32"))]
const PING_INTERVAL: u64 = 55;

/// [`Relay`] error
#[derive(Debug, Error)]
pub enum Error {
    /// Negentropy error
    #[error(transparent)]
    Negentropy(#[from] negentropy::Error),
    /// Database error
    #[error(transparent)]
    Database(#[from] DatabaseError),
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
    /// Relay not connected
    #[error("relay not connected")]
    NotConnected,
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
    /// Reconciliation error
    #[error("negentropy reconciliation error: {0}")]
    NegentropyReconciliation(NegentropyErrorCode),
    /// Negentropy not supported
    #[error("negentropy not supported")]
    NegentropyNotSupported,
    /// Unknown negentropy error
    #[error("unknown negentropy error")]
    UnknownNegentropyError,
}

/// Relay connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayStatus {
    /// Relay initialized
    Initialized,
    /// Pending
    Pending,
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
            Self::Pending => write!(f, "Pending"),
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
    url: Url,
    #[cfg(not(target_arch = "wasm32"))]
    proxy: Option<SocketAddr>,
    status: Arc<RwLock<RelayStatus>>,
    #[cfg(feature = "nip11")]
    document: Arc<RwLock<RelayInformationDocument>>,
    opts: RelayOptions,
    stats: RelayConnectionStats,
    database: Arc<DynNostrDatabase>,
    scheduled_for_stop: Arc<AtomicBool>,
    scheduled_for_termination: Arc<AtomicBool>,
    pool_sender: Sender<RelayPoolMessage>,
    relay_sender: Sender<Message>,
    relay_receiver: Arc<Mutex<Receiver<Message>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    subscriptions: Arc<RwLock<HashMap<InternalSubscriptionId, ActiveSubscription>>>,
    limits: Limits,
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
        database: Arc<DynNostrDatabase>,
        pool_sender: Sender<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
        proxy: Option<SocketAddr>,
        opts: RelayOptions,
        limits: Limits,
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
            database,
            scheduled_for_stop: Arc::new(AtomicBool::new(false)),
            scheduled_for_termination: Arc::new(AtomicBool::new(false)),
            pool_sender,
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
            notification_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            limits,
        }
    }

    /// Create new `Relay`
    #[cfg(target_arch = "wasm32")]
    pub fn new(
        url: Url,
        database: Arc<DynNostrDatabase>,
        pool_sender: Sender<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
        opts: RelayOptions,
        limits: Limits,
    ) -> Self {
        let (relay_sender, relay_receiver) = mpsc::channel::<Message>(1024);

        Self {
            url,
            status: Arc::new(RwLock::new(RelayStatus::Initialized)),
            #[cfg(feature = "nip11")]
            document: Arc::new(RwLock::new(RelayInformationDocument::new())),
            opts,
            stats: RelayConnectionStats::new(),
            database,
            scheduled_for_stop: Arc::new(AtomicBool::new(false)),
            scheduled_for_termination: Arc::new(AtomicBool::new(false)),
            pool_sender,
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
            notification_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            limits,
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

    /// Get [`ActiveSubscription`] by [`InternalSubscriptionId`]
    pub async fn subscription(
        &self,
        internal_id: &InternalSubscriptionId,
    ) -> Option<ActiveSubscription> {
        let subscription = self.subscriptions.read().await;
        subscription.get(internal_id).cloned()
    }

    async fn update_subscription_filters(
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

        if let RelayStatus::Initialized | RelayStatus::Stopped | RelayStatus::Terminated =
            self.status().await
        {
            if self.opts.get_reconnect() {
                if wait_for_connection {
                    self.try_connect().await
                }

                tracing::debug!("Auto connect loop started for {}", self.url);

                if !wait_for_connection {
                    self.set_status(RelayStatus::Pending).await;
                }

                let relay = self.clone();
                thread::abortable(async move {
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
                            RelayStatus::Initialized
                            | RelayStatus::Pending
                            | RelayStatus::Disconnected => relay.try_connect().await,
                            RelayStatus::Stopped | RelayStatus::Terminated => {
                                tracing::debug!("Auto connect loop terminated for {}", relay.url);
                                break;
                            }
                            _ => (),
                        };

                        let retry_sec: u64 = if relay.opts.get_adjust_retry_sec() {
                            let var: u64 =
                                relay.stats.attempts().saturating_sub(relay.stats.success()) as u64;
                            if var >= 3 {
                                let retry_interval: i64 =
                                    cmp::min(MIN_RETRY_SEC * (1 + var), MAX_ADJ_RETRY_SEC) as i64;
                                let jitter: i64 = rand::thread_rng().gen_range(-1..=1);
                                retry_interval.saturating_add(jitter) as u64
                            } else {
                                relay.opts().get_retry_sec()
                            }
                        } else {
                            relay.opts().get_retry_sec()
                        };

                        tracing::trace!("{} retry time set to {retry_sec} secs", relay.url);
                        thread::sleep(Duration::from_secs(retry_sec)).await;
                    }
                });
            } else if wait_for_connection {
                self.try_connect().await
            } else {
                let relay = self.clone();
                thread::spawn(async move { relay.try_connect().await });
            }
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

                            let nonce: u64 = rand::thread_rng().gen();
                            if relay.stats.ping.set_last_nonce(nonce)
                                && relay.stats.ping.set_replied(false)
                            {
                                if let Err(e) =
                                    relay.send_relay_event(RelayEvent::Ping { nonce }, None)
                                {
                                    tracing::error!("Impossible to ping {}: {e}", relay.url);
                                    break;
                                };
                            } else {
                                tracing::warn!(
                                    "`last_nonce` or `replied` not updated for {}!",
                                    relay.url
                                );
                            }

                            thread::sleep(Duration::from_secs(PING_INTERVAL)).await;
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
                        let size: usize = data.len();
                        let max_size: usize = relay.limits.messages.max_size as usize;
                        relay.stats.add_bytes_received(size);
                        if size <= max_size {
                            match RawRelayMessage::from_json(&data) {
                                Ok(msg) => {
                                    tracing::trace!(
                                        "Received message from {}: {:?}",
                                        relay.url,
                                        msg
                                    );
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
                                Err(e) => match e {
                                    MessageHandleError::EmptyMsg => (),
                                    _ => tracing::error!("{e}: {}", String::from_utf8_lossy(&data)),
                                },
                            };
                        } else {
                            tracing::error!("Received message too large from {}: size={size}, max_size={max_size}", relay.url);
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
                if self.opts.get_read() {
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
        if !self.opts.get_write() {
            if let ClientMessage::Event(_) = msg {
                return Err(Error::WriteDisabled);
            }
        }

        if !self.opts.get_read() {
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
        if !self.opts.get_write() && msgs.iter().any(|msg| msg.is_event()) {
            return Err(Error::WriteDisabled);
        }

        if !self.opts.get_read() && msgs.iter().any(|msg| msg.is_req() || msg.is_close()) {
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

        if opts.skip_disconnected
            && !self.is_connected().await
            && self.stats.attempts() > 1
            && self.stats.uptime() < MIN_UPTIME
        {
            return Err(Error::EventNotPublished(String::from(
                "relay not connected",
            )));
        }

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

    /// Send multiple [`Event`] at once
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        if events.is_empty() {
            return Err(Error::BatchEventEmpty);
        }

        if opts.skip_disconnected
            && !self.is_connected().await
            && self.stats.attempts() > 1
            && self.stats.uptime() < MIN_UPTIME
        {
            return Err(Error::EventNotPublished(String::from(
                "relay not connected",
            )));
        }

        let mut msgs: Vec<ClientMessage> = Vec::with_capacity(events.len());
        let mut missing: HashSet<EventId> = HashSet::new();

        for event in events.into_iter() {
            missing.insert(event.id);
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
        if !self.opts.get_read() {
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
        if !self.opts.get_read() {
            return Err(Error::ReadDisabled);
        }

        let sub: ActiveSubscription = self
            .subscription(&internal_id)
            .await
            .ok_or(Error::InternalIdNotFound)?;
        self.send_msg(ClientMessage::new_req(sub.id, sub.filters), wait)
            .await?;

        Ok(())
    }

    /// Subscribe to filters
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Default`
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
        if !self.opts.get_read() {
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
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Default`
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
        if !self.opts.get_read() {
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
        if !self.opts.get_read() {
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
        if !self.is_connected().await
            && self.stats.attempts() > 1
            && self.stats.uptime() < MIN_UPTIME
        {
            return Err(Error::NotConnected);
        }

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
    async fn get_events_of_with_callback<F>(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
        callback: impl Fn(Event) -> F,
    ) -> Result<(), Error>
    where
        F: Future<Output = ()>,
    {
        if !self.opts.get_read() {
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
    ///
    /// Get events from local database and relay
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let stored_events: Vec<Event> = self
            .database
            .query(filters.clone())
            .await
            .unwrap_or_default();
        let events: Mutex<Vec<Event>> = Mutex::new(stored_events);
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
        if !self.opts.get_read() {
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

    /// Count events of filters
    pub async fn count_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<usize, Error> {
        let id = SubscriptionId::generate();
        self.send_msg(ClientMessage::new_count(id.clone(), filters), None)
            .await?;

        let mut count = 0;

        let mut notifications = self.notification_sender.subscribe();
        time::timeout(Some(timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Message(
                    url,
                    RelayMessage::Count {
                        subscription_id,
                        count: c,
                    },
                ) = notification
                {
                    if subscription_id == id && url == self.url {
                        count = c;
                        break;
                    }
                }
            }
        })
        .await
        .ok_or(Error::Timeout)?;

        // Unsubscribe
        self.send_msg(ClientMessage::close(id), None).await?;

        Ok(count)
    }

    /// Negentropy reconciliation
    pub async fn reconcile(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<(), Error> {
        if !self.opts.get_read() {
            return Err(Error::ReadDisabled);
        }

        if !self.is_connected().await
            && self.stats.attempts() > 1
            && self.stats.uptime() < MIN_UPTIME
        {
            return Err(Error::NotConnected);
        }

        let id_size: usize = 32;

        let mut negentropy = Negentropy::new(id_size, Some(4_096))?;

        for (id, timestamp) in items.into_iter() {
            let id = Bytes::from_slice(id.as_bytes());
            negentropy.add_item(timestamp.as_u64(), id)?;
        }

        negentropy.seal()?;

        let sub_id = SubscriptionId::generate();
        let open_msg = ClientMessage::neg_open(&mut negentropy, &sub_id, filter)?;

        self.send_msg(open_msg, Some(Duration::from_secs(10)))
            .await?;

        let mut notifications = self.notification_sender.subscribe();
        let mut temp_notifications = self.notification_sender.subscribe();

        // Check if negentropy it's supported
        time::timeout(Some(opts.initial_timeout), async {
            while let Ok(notification) = temp_notifications.recv().await {
                if let RelayPoolNotification::Message(url, msg) = notification {
                    if url == self.url {
                        match msg {
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
                                if message.contains("bad msg: unknown cmd") {
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
            }

            Ok::<(), Error>(())
        })
        .await
        .ok_or(Error::Timeout)??;

        while let Ok(notification) = notifications.recv().await {
            match notification {
                RelayPoolNotification::Message(url, msg) => {
                    if url == self.url {
                        match msg {
                            RelayMessage::NegMsg {
                                subscription_id,
                                message,
                            } => {
                                if subscription_id == sub_id {
                                    let query: Bytes = Bytes::from_hex(message)?;
                                    let mut have_ids: Vec<Bytes> = Vec::new();
                                    let mut need_ids: Vec<Bytes> = Vec::new();
                                    let msg: Option<Bytes> = negentropy.reconcile_with_ids(
                                        &query,
                                        &mut have_ids,
                                        &mut need_ids,
                                    )?;

                                    if opts.bidirectional {
                                        let ids = have_ids
                                            .into_iter()
                                            .filter_map(|id| EventId::from_slice(&id).ok());
                                        let filter = Filter::new().ids(ids);
                                        let events: Vec<Event> =
                                            self.database.query(vec![filter]).await?;
                                        let msgs: Vec<ClientMessage> = events
                                            .into_iter()
                                            .map(ClientMessage::new_event)
                                            .collect();
                                        if let Err(e) = self
                                            .batch_msg(msgs, Some(opts.batch_send_timeout))
                                            .await
                                        {
                                            tracing::error!("negentropy reconciliation: impossible to batch events to {}: {e}", self.url);
                                        }
                                    }

                                    if need_ids.is_empty() {
                                        tracing::info!(
                                            "Negentropy reconciliation terminated for {}",
                                            self.url
                                        );
                                        break;
                                    }

                                    let ids = need_ids
                                        .into_iter()
                                        .filter_map(|id| EventId::from_slice(&id).ok());
                                    let filter = Filter::new().ids(ids);
                                    if !filter.ids.is_empty() {
                                        let timeout: Duration = opts.static_get_events_timeout
                                            + opts
                                                .relative_get_events_timeout
                                                .mul(filter.ids.len() as u32);
                                        self.get_events_of(
                                            vec![filter],
                                            timeout,
                                            FilterOptions::ExitOnEOSE,
                                        )
                                        .await?;
                                    } else {
                                        tracing::warn!("negentropy reconciliation: tried to send empty filters to {}", self.url);
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
                                                None,
                                            )
                                            .await?;
                                        }
                                        None => {
                                            tracing::info!(
                                                "Negentropy reconciliation terminated for {}",
                                                self.url
                                            );
                                            break;
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
                            _ => (),
                        }
                    }
                }
                RelayPoolNotification::RelayStatus { url, status } => {
                    if url == self.url && status != RelayStatus::Connected {
                        return Err(Error::NotConnected);
                    }
                }
                RelayPoolNotification::Stop | RelayPoolNotification::Shutdown => break,
                _ => (),
            };
        }

        let close_msg = ClientMessage::NegClose {
            subscription_id: sub_id,
        };
        self.send_msg(close_msg, None).await?;

        Ok(())
    }

    /// Check if relay support negentropy protocol
    pub async fn support_negentropy(&self) -> Result<bool, Error> {
        let pk = Keys::generate();
        let filter = Filter::new().author(pk.public_key());
        match self
            .reconcile(
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
