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

use async_utility::futures_util::stream::AbortHandle;
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
    ClientMessage, Event, EventId, Filter, JsonUtil, Keys, MissingPartialEvent, PartialEvent,
    RawRelayMessage, RelayMessage, SubscriptionId, Timestamp, Url,
};
use nostr_database::{DynNostrDatabase, Order};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot, Mutex, RwLock};

use super::flags::AtomicRelayServiceFlags;
use super::options::{
    FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions, SubscribeAutoCloseOptions,
    SubscribeOptions, MAX_ADJ_RETRY_SEC, MIN_RETRY_SEC, NEGENTROPY_BATCH_SIZE_DOWN,
    NEGENTROPY_HIGH_WATER_UP, NEGENTROPY_LOW_WATER_UP,
};
use super::stats::RelayConnectionStats;
use super::{Error, RelayNotification, RelayStatus};
use crate::pool::RelayPoolNotification;

type Message = (RelayEvent, Option<oneshot::Sender<bool>>);

const MIN_ATTEMPTS: usize = 1;
const MIN_UPTIME: f64 = 0.90;
#[cfg(not(target_arch = "wasm32"))]
const PING_INTERVAL: u64 = 55;

/// Relay event
#[derive(Debug)]
enum RelayEvent {
    /// Send messages
    Batch(Vec<ClientMessage>),
    /// Ping
    #[cfg(not(target_arch = "wasm32"))]
    Ping {
        /// Nonce
        nonce: u64,
    },
    /// Close
    #[cfg(not(target_arch = "wasm32"))]
    Close,
    /// Stop
    Stop,
    /// Completely disconnect
    Terminate,
}

#[derive(Debug, Clone)]
pub(crate) struct InternalRelay {
    pub(super) url: Url,
    status: Arc<RwLock<RelayStatus>>,
    #[cfg(feature = "nip11")]
    document: Arc<RwLock<RelayInformationDocument>>,
    opts: RelayOptions,
    stats: RelayConnectionStats,
    database: Arc<DynNostrDatabase>,
    scheduled_for_stop: Arc<AtomicBool>,
    scheduled_for_termination: Arc<AtomicBool>,
    relay_sender: Sender<Message>,
    relay_receiver: Arc<Mutex<Receiver<Message>>>,
    pub(super) internal_notification_sender: broadcast::Sender<RelayNotification>,
    external_notification_sender: Arc<RwLock<Option<broadcast::Sender<RelayPoolNotification>>>>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Vec<Filter>>>>,
}

impl AtomicDestroyer for InternalRelay {
    fn name(&self) -> Option<String> {
        Some(format!("Relay {}", self.url))
    }

    fn on_destroy(&self) {
        let relay = self.clone();
        let _ = thread::spawn(async move {
            if let Err(e) = relay.terminate().await {
                tracing::error!("Impossible to shutdown {} relay: {e}", relay.url);
            }
        });
    }
}

impl InternalRelay {
    pub fn new(url: Url, database: Arc<DynNostrDatabase>, opts: RelayOptions) -> Self {
        let (relay_sender, relay_receiver) = mpsc::channel::<Message>(1024);
        let (relay_notification_sender, ..) = broadcast::channel::<RelayNotification>(2048);

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
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
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

    async fn set_status(&self, status: RelayStatus) {
        // Change status
        let mut s = self.status.write().await;
        *s = status;

        // Send notification
        self.send_notification(RelayNotification::RelayStatus { status })
            .await;
    }

    #[inline]
    pub fn flags(&self) -> AtomicRelayServiceFlags {
        self.opts.flags.clone()
    }

    #[inline]
    pub async fn is_connected(&self) -> bool {
        self.status().await == RelayStatus::Connected
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
        subscription.clone()
    }

    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        let subscription = self.subscriptions.read().await;
        subscription.get(id).cloned()
    }

    pub(crate) async fn update_subscription(&self, id: SubscriptionId, filters: Vec<Filter>) {
        let mut subscriptions = self.subscriptions.write().await;
        let current: &mut Vec<Filter> = subscriptions.entry(id).or_default();
        *current = filters;
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
        self.relay_sender.max_capacity() - self.relay_sender.capacity()
    }

    #[inline]
    fn is_scheduled_for_stop(&self) -> bool {
        self.scheduled_for_stop.load(Ordering::SeqCst)
    }

    #[inline]
    fn schedule_for_stop(&self, value: bool) {
        self.scheduled_for_stop.store(value, Ordering::SeqCst);
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

    async fn send_notification(&self, notification: RelayNotification) {
        // Send internal notification
        let _ = self.internal_notification_sender.send(notification.clone());

        // Send external notification
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
                RelayNotification::RelayStatus { status } => RelayPoolNotification::RelayStatus {
                    relay_url: self.url(),
                    status,
                },
                RelayNotification::Shutdown => RelayPoolNotification::Shutdown,
                RelayNotification::Stop => RelayPoolNotification::Stop,
            };

            // Send notification
            let _ = external_notification_sender.send(notification);
        }
    }

    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        self.schedule_for_stop(false);
        self.schedule_for_termination(false);

        if let RelayStatus::Initialized | RelayStatus::Stopped | RelayStatus::Terminated =
            self.status().await
        {
            if self.opts.get_reconnect() {
                if connection_timeout.is_some() {
                    self.try_connect(connection_timeout).await
                }

                tracing::debug!("Auto connect loop started for {}", self.url);

                if connection_timeout.is_none() {
                    self.set_status(RelayStatus::Pending).await;
                }

                let relay = self.clone();
                let _ = thread::spawn(async move {
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
                            | RelayStatus::Disconnected => {
                                relay.try_connect(connection_timeout).await
                            }
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
            } else if connection_timeout.is_some() {
                self.try_connect(connection_timeout).await
            } else {
                let relay = self.clone();
                let _ = thread::spawn(async move { relay.try_connect(connection_timeout).await });
            }
        }
    }

    #[cfg(feature = "nip11")]
    fn request_nip11_document(&self) {
        let relay = self.clone();
        let _ = thread::spawn(async move {
            match RelayInformationDocument::get(relay.url(), relay.proxy()).await {
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
    fn spawn_pinger(&self) -> Option<AbortHandle> {
        let relay = self.clone();
        thread::abortable(async move {
            if relay.opts.flags.has_ping() {
                tracing::debug!("Relay Ping Thread Started");

                loop {
                    if relay.stats.ping.last_nonce() != 0 && !relay.stats.ping.replied() {
                        tracing::warn!("{} not replied to ping", relay.url);
                        relay.stats.ping.reset();
                        break;
                    }

                    let nonce: u64 = rand::random();
                    relay.stats.ping.set_last_nonce(nonce);
                    relay.stats.ping.set_replied(false);

                    if let Err(e) = relay.send_relay_event(RelayEvent::Ping { nonce }, None) {
                        tracing::error!("Impossible to ping {}: {e}", relay.url);
                        break;
                    };

                    thread::sleep(Duration::from_secs(PING_INTERVAL)).await;
                }

                tracing::debug!("Exited from Ping Thread of {}", relay.url);

                if let Err(err) = relay.disconnect().await {
                    tracing::error!("Impossible to disconnect {}: {}", relay.url, err);
                }
            }
        })
        .ok()
    }

    #[inline]
    #[cfg(target_arch = "wasm32")]
    fn spawn_pinger(&self) -> Option<AbortHandle> {
        None
    }

    fn spawn_message_handler(
        &self,
        mut ws_tx: Sink,
        mut ws_rx: Stream,
        _ping_abort_handle: Option<AbortHandle>,
    ) {
        let relay = self.clone();
        let _ = thread::spawn(async move {
            tracing::debug!("Relay Message Handler started for {}", relay.url);

            let sender = async {
                let mut rx = relay.relay_receiver.lock().await;
                while let Some((relay_event, oneshot_sender)) = rx.recv().await {
                    match relay_event {
                        RelayEvent::Batch(msgs) => {
                            let msgs: Vec<String> =
                                msgs.into_iter().map(|msg| msg.as_json()).collect();
                            let size: usize = msgs.iter().map(|msg| msg.as_bytes().len()).sum();
                            let len = msgs.len();

                            if len == 1 {
                                if let Some(json) = msgs.first() {
                                    tracing::debug!(
                                        "Sending {json} to {} (size: {size} bytes)",
                                        relay.url
                                    );
                                }
                            } else {
                                tracing::debug!(
                                    "Sending {len} messages to {} (size: {size} bytes)",
                                    relay.url
                                );
                            }

                            let msgs = msgs.into_iter().map(|msg| Ok(WsMessage::Text(msg)));
                            let mut stream = futures_util::stream::iter(msgs);
                            match ws_tx.send_all(&mut stream).await {
                                Ok(_) => {
                                    relay.stats.add_bytes_sent(size);
                                    if let Some(sender) = oneshot_sender {
                                        if sender.send(true).is_err() {
                                            tracing::error!("Impossible to send oneshot msg");
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
                            if relay.opts.flags.has_ping() {
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
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        RelayEvent::Close => {
                            let _ = ws_tx.close().await;
                            relay.set_status(RelayStatus::Disconnected).await;
                            tracing::info!("Disconnected from {}", relay.url);
                            break;
                        }
                        RelayEvent::Stop => {
                            if relay.is_scheduled_for_stop() {
                                let _ = ws_tx.close().await;
                                relay.set_status(RelayStatus::Stopped).await;
                                relay.schedule_for_stop(false);
                                tracing::info!("Stopped {}", relay.url);
                                break;
                            }
                        }
                        RelayEvent::Terminate => {
                            if relay.is_scheduled_for_termination() {
                                let _ = ws_tx.close().await;
                                relay.set_status(RelayStatus::Terminated).await;
                                relay.schedule_for_termination(false);
                                tracing::info!("Completely disconnected from {}", relay.url);
                                break;
                            }
                        }
                    }
                }
            };

            // TODO: use a single receiver (require changes to `async-wsocket`)

            // Native receiver
            #[cfg(not(target_arch = "wasm32"))]
            let receiver = async {
                while let Some(msg) = ws_rx.next().await {
                    if let Ok(msg) = msg {
                        match msg {
                            WsMessage::Pong(bytes) => {
                                if relay.opts.flags.has_ping() {
                                    match String::from_utf8(bytes) {
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
                                                    relay
                                                        .stats
                                                        .save_latency(sent_at.elapsed())
                                                        .await;
                                                } else {
                                                    tracing::error!("Pong nonce not match: received={nonce}, expected={}", relay.stats.ping.last_nonce());
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
                                relay.handle_relay_message_infallible(&data).await;
                            }
                        }
                    }
                }
            };

            // WASM receiver
            #[cfg(target_arch = "wasm32")]
            let receiver = async {
                while let Some(msg) = ws_rx.next().await {
                    let data: &[u8] = msg.as_ref();
                    relay.handle_relay_message_infallible(data).await;
                }
            };

            // Wait that one of the futures terminate
            tokio::select! {
                _ = receiver => {
                    tracing::trace!("Relay connection closed for {}", relay.url);
                }
                _ = sender => {
                    tracing::trace!("Relay sender exited for {}", relay.url);
                }
            }

            tracing::debug!("Exited from Message Handler of {}", relay.url);

            // Abort pinger
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(handle) = _ping_abort_handle {
                handle.abort();
            }
        });
    }

    async fn try_connect(&self, connection_timeout: Option<Duration>) {
        self.stats.new_attempt();

        let url: String = self.url.to_string();

        // Set RelayStatus to `Connecting`
        self.set_status(RelayStatus::Connecting).await;
        tracing::debug!("Connecting to {}", url);

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
                self.set_status(RelayStatus::Connected).await;
                tracing::info!("Connected to {url}");

                self.stats.new_success();

                // Spawn pinger
                let ping_abort_handle: Option<AbortHandle> = self.spawn_pinger();

                // Spawn message handler
                self.spawn_message_handler(ws_tx, ws_rx, ping_abort_handle);

                // Subscribe to relay
                if self.opts.flags.has_read() {
                    if let Err(e) = self
                        .resubscribe_all(RelaySendOptions::default().skip_send_confirmation(true))
                        .await
                    {
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

    #[inline(always)]
    async fn handle_relay_message_infallible(&self, msg: &[u8]) {
        match self.handle_relay_message(msg).await {
            Ok(Some(message)) => {
                match &message {
                    RelayMessage::Notice { message } => {
                        tracing::warn!("Notice from {}: {message}", self.url)
                    }
                    RelayMessage::Ok {
                        event_id,
                        status,
                        message,
                    } => {
                        tracing::debug!("Received OK from {} for event {event_id}: status={status}, message={message}", self.url);
                    }
                    _ => (),
                }

                // Send notification
                self.send_notification(RelayNotification::Message { message })
                    .await;
            }
            Ok(None) | Err(Error::MessageHandle(MessageHandleError::EmptyMsg)) => (),
            Err(e) => tracing::error!("Impossible to handle relay message from {}: {e}", self.url),
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
        tracing::trace!("Received message from {}: {:?}", self.url, msg);

        // Handle msg
        match msg {
            RawRelayMessage::Event {
                subscription_id,
                event,
            } => {
                // Check event size
                if let Some(max_size) = self.opts.limits.events.max_size {
                    let size: usize = event.as_json().as_bytes().len();
                    let max_size: usize = max_size as usize;
                    if size > max_size {
                        return Err(Error::EventTooLarge { size, max_size });
                    }
                }

                // Check tags limit
                if let Some(max_num_tags) = self.opts.limits.events.max_num_tags {
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

                // Check if event is replaceable and has coordinate
                if missing.kind.is_replaceable() || missing.kind.is_parameterized_replaceable() {
                    let coordinate: Coordinate =
                        Coordinate::new(missing.kind, partial_event.pubkey.clone())
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
                if self
                    .database
                    .has_event_already_been_saved(&partial_event.id)
                    .await?
                {
                    tracing::trace!("Event {} already saved into database", partial_event.id);
                    return Ok(None);
                }

                // Compose full event
                let event: Event = partial_event.merge(missing)?;

                // Check if it's expired
                if event.is_expired() {
                    return Err(Error::EventExpired);
                }

                // Verify event
                event.verify()?;

                // Save event
                self.database.save_event(&event).await?;

                // Box event
                let event: Box<Event> = Box::new(event);

                // Check if seen
                if !seen {
                    // Send notification
                    self.send_notification(RelayNotification::Event {
                        subscription_id: SubscriptionId::new(&subscription_id),
                        event: event.clone(),
                    })
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

    #[inline]
    fn send_relay_event(
        &self,
        relay_msg: RelayEvent,
        sender: Option<oneshot::Sender<bool>>,
    ) -> Result<(), Error> {
        self.relay_sender
            .try_send((relay_msg, sender))
            .map_err(|_| Error::MessageNotSent)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn disconnect(&self) -> Result<(), Error> {
        let status = self.status().await;
        if !status.is_disconnected() {
            self.send_relay_event(RelayEvent::Close, None)?;
        }
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Error> {
        self.schedule_for_stop(true);
        let status = self.status().await;
        if !status.is_disconnected() {
            self.send_relay_event(RelayEvent::Stop, None)?;
        }
        self.send_notification(RelayNotification::Stop).await;
        Ok(())
    }

    pub async fn terminate(&self) -> Result<(), Error> {
        self.schedule_for_termination(true);
        let status = self.status().await;
        if !status.is_disconnected() {
            self.send_relay_event(RelayEvent::Terminate, None)?;
        }
        self.send_notification(RelayNotification::Shutdown).await;
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
            self.send_relay_event(RelayEvent::Batch(msgs), None)
        } else {
            let (tx, rx) = oneshot::channel::<bool>();
            self.send_relay_event(RelayEvent::Batch(msgs), Some(tx))?;
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
                _ => Err(Error::RecvTimeout),
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

    async fn resubscribe_all(&self, opts: RelaySendOptions) -> Result<(), Error> {
        if !self.opts.flags.has_read() {
            return Err(Error::ReadDisabled);
        }

        let subscriptions = self.subscriptions().await;
        for (id, filters) in subscriptions.into_iter() {
            if !filters.is_empty() {
                self.send_msg(ClientMessage::req(id, filters), opts).await?;
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

        // Compose and send message
        let msg: ClientMessage = ClientMessage::req(id.clone(), filters.clone());
        self.send_msg(msg, opts.send_opts).await?;

        // Check if auto-close condition is set
        if let Some(opts) = opts.auto_close {
            let this = self.clone();
            thread::spawn(async move {
                let sub_id = id.clone();
                let relay = this.clone();
                let res = time::timeout(opts.timeout, async move {
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
                                        | FilterOptions::WaitDurationAfterEOSE(_) = opts.filter
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
                            RelayNotification::Stop | RelayNotification::Shutdown => {
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
                                    RelayNotification::Stop | RelayNotification::Shutdown => {
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
        } else {
            // No auto-close subscription: update subscription filters
            self.update_subscription(id.clone(), filters).await;
        }

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
                    RelayNotification::Stop | RelayNotification::Shutdown => break,
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
                        RelayNotification::Stop | RelayNotification::Shutdown => break,
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

    pub async fn reconcile(&self, filter: Filter, opts: NegentropyOptions) -> Result<(), Error> {
        let items = self.database.negentropy_items(filter.clone()).await?;
        self.reconcile_with_items(filter, items, opts).await
    }

    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<(), Error> {
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

        let do_up: bool = opts.direction.do_up();
        let do_down: bool = opts.direction.do_down();
        let mut in_flight_up: HashSet<EventId> = HashSet::new();
        let mut in_flight_down = false;
        let mut sync_done = false;
        let mut have_ids: Vec<Bytes> = Vec::new();
        let mut need_ids: Vec<Bytes> = Vec::new();
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
                                let query: Bytes = Bytes::from_hex(message)?;
                                let msg: Option<Bytes> = negentropy.reconcile_with_ids(
                                    &query,
                                    &mut have_ids,
                                    &mut need_ids,
                                )?;

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
                            if in_flight_up.remove(&event_id) && !status {
                                tracing::error!(
                                    "Unable to upload event {event_id} to {}: {message}",
                                    self.url
                                );
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
                RelayNotification::Stop | RelayNotification::Shutdown => break,
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

        Ok(())
    }

    pub async fn support_negentropy(&self) -> Result<bool, Error> {
        let pk = Keys::generate();
        let filter = Filter::new().author(pk.public_key().clone());
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
