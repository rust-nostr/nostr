// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Relay

use std::fmt;
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::{thread, time};
#[cfg(feature = "nip11")]
use nostr::nips::nip11::RelayInformationDocument;
use nostr::{ClientMessage, Event, Filter, RelayMessage, SubscriptionId, Timestamp, Url};
use nostr_sdk_net::futures_util::{Future, SinkExt, StreamExt};
use nostr_sdk_net::{self as net, WsMessage};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot, Mutex};

pub mod pool;

pub use self::pool::{RelayPoolMessage, RelayPoolNotification};
#[cfg(feature = "blocking")]
use crate::RUNTIME;

type Message = (RelayEvent, Option<oneshot::Sender<bool>>);

/// [`Relay`] error
#[derive(Debug, thiserror::Error)]
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
    MessagetNotSent,
    /// Impossible to receive oneshot message
    #[error("impossible to recv msg")]
    OneShotRecvError,
    /// Read actions disabled
    #[error("read actions are disabled for this relay")]
    ReadDisabled,
    /// Write actions disabled
    #[error("write actions are disabled for this relay")]
    WriteDisabled,
    /// Filters empty
    #[error("filters empty")]
    FiltersEmpty,
}

/// Relay connection status
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayStatus {
    /// Relay initialized
    Initialized,
    /// Relay connected
    Connected,
    /// Connecting
    Connecting,
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
            Self::Connected => write!(f, "Connected"),
            Self::Connecting => write!(f, "Connecting"),
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
    // Ping,
    /// Close
    Close,
    /// Stop
    Stop,
    /// Completely disconnect
    Terminate,
}

/// [`Relay`] options
#[derive(Debug, Clone)]
pub struct RelayOptions {
    /// Allow/disallow read actions
    read: Arc<AtomicBool>,
    /// Allow/disallow write actions
    write: Arc<AtomicBool>,
}

impl Default for RelayOptions {
    fn default() -> Self {
        Self::new(true, true)
    }
}

impl RelayOptions {
    /// New [`RelayOptions`]
    pub fn new(read: bool, write: bool) -> Self {
        Self {
            read: Arc::new(AtomicBool::new(read)),
            write: Arc::new(AtomicBool::new(write)),
        }
    }

    /// Get read option
    pub fn read(&self) -> bool {
        self.read.load(Ordering::SeqCst)
    }

    /// Set read option
    pub fn set_read(&self, read: bool) {
        let _ = self
            .read
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(read));
    }

    /// Get write option
    pub fn write(&self) -> bool {
        self.write.load(Ordering::SeqCst)
    }

    /// Set write option
    pub fn set_write(&self, write: bool) {
        let _ = self
            .write
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(write));
    }
}

/// [`Relay`] connection stats
#[derive(Debug, Clone)]
pub struct RelayConnectionStats {
    attempts: Arc<AtomicUsize>,
    success: Arc<AtomicUsize>,
    connected_at: Arc<AtomicU64>,
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
            connected_at: Arc::new(AtomicU64::new(0)),
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

    /// Get the UNIX timestamp of the last started connection
    pub fn connected_at(&self) -> Timestamp {
        Timestamp::from(self.connected_at.load(Ordering::SeqCst))
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
    /// Create new [`ActiveSubscription`]
    pub fn new() -> Self {
        Self {
            id: SubscriptionId::generate(),
            filters: Vec::new(),
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
    status: Arc<Mutex<RelayStatus>>,
    #[cfg(feature = "nip11")]
    document: Arc<Mutex<RelayInformationDocument>>,
    opts: RelayOptions,
    stats: RelayConnectionStats,
    scheduled_for_stop: Arc<AtomicBool>,
    scheduled_for_termination: Arc<AtomicBool>,
    pool_sender: Sender<RelayPoolMessage>,
    relay_sender: Sender<Message>,
    relay_receiver: Arc<Mutex<Receiver<Message>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    subscription: Arc<Mutex<ActiveSubscription>>,
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
            status: Arc::new(Mutex::new(RelayStatus::Initialized)),
            #[cfg(feature = "nip11")]
            document: Arc::new(Mutex::new(RelayInformationDocument::new())),
            opts,
            stats: RelayConnectionStats::new(),
            scheduled_for_stop: Arc::new(AtomicBool::new(false)),
            scheduled_for_termination: Arc::new(AtomicBool::new(false)),
            pool_sender,
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
            notification_sender,
            subscription: Arc::new(Mutex::new(ActiveSubscription::new())),
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
            status: Arc::new(Mutex::new(RelayStatus::Initialized)),
            #[cfg(feature = "nip11")]
            document: Arc::new(Mutex::new(RelayInformationDocument::new())),
            opts,
            stats: RelayConnectionStats::new(),
            scheduled_for_stop: Arc::new(AtomicBool::new(false)),
            scheduled_for_termination: Arc::new(AtomicBool::new(false)),
            pool_sender,
            relay_sender,
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
            notification_sender,
            subscription: Arc::new(Mutex::new(ActiveSubscription::new())),
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
        let status = self.status.lock().await;
        status.clone()
    }

    /// Get [`RelayStatus`]
    #[cfg(feature = "blocking")]
    pub fn status_blocking(&self) -> RelayStatus {
        RUNTIME.block_on(async { self.status().await })
    }

    async fn set_status(&self, status: RelayStatus) {
        let mut s = self.status.lock().await;
        *s = status;
    }

    /// Check if [`Relay`] is connected
    pub async fn is_connected(&self) -> bool {
        self.status().await == RelayStatus::Connected
    }

    /// Get [`RelayInformationDocument`]
    #[cfg(feature = "nip11")]
    pub async fn document(&self) -> RelayInformationDocument {
        let document = self.document.lock().await;
        document.clone()
    }

    /// Get [`RelayInformationDocument`]
    #[cfg(all(feature = "nip11", feature = "blocking"))]
    pub fn document_blocking(&self) -> RelayInformationDocument {
        RUNTIME.block_on(async { self.document().await })
    }

    #[cfg(feature = "nip11")]
    async fn set_document(&self, document: RelayInformationDocument) {
        let mut d = self.document.lock().await;
        *d = document;
    }

    /// Get [`ActiveSubscription`]
    pub async fn subscription(&self) -> ActiveSubscription {
        let subscription = self.subscription.lock().await;
        subscription.clone()
    }

    /// Update [`ActiveSubscription`]
    pub async fn update_subscription_filters(&self, filters: Vec<Filter>) {
        let mut s = self.subscription.lock().await;
        s.filters = filters;
    }

    /// Get [`RelayOptions`]
    pub fn opts(&self) -> RelayOptions {
        self.opts.clone()
    }

    /// Get [`RelayConnectionStats`]
    pub fn stats(&self) -> RelayConnectionStats {
        self.stats.clone()
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
            if wait_for_connection {
                self.try_connect().await
            } else {
                // Update relay status
                self.set_status(RelayStatus::Disconnected).await;
            }

            let relay = self.clone();
            thread::spawn(async move {
                loop {
                    log::debug!(
                        "{} channel capacity: {}",
                        relay.url(),
                        relay.relay_sender.capacity()
                    );

                    // Schedule relay for termination
                    // Needed to terminate the auto reconnect loop, also if the relay is not connected yet.
                    if relay.is_scheduled_for_termination() {
                        relay.set_status(RelayStatus::Terminated).await;
                        relay.schedule_for_termination(false);
                        log::debug!("Auto connect loop terminated for {} [schedule]", relay.url);
                        break;
                    }

                    // Check status
                    match relay.status().await {
                        RelayStatus::Disconnected => relay.try_connect().await,
                        RelayStatus::Terminated => {
                            log::debug!("Auto connect loop terminated for {}", relay.url);
                            break;
                        }
                        _ => (),
                    };

                    thread::sleep(Duration::from_secs(20)).await;
                }
            });
        }
    }

    async fn try_connect(&self) {
        self.stats.new_attempt();

        let url: String = self.url.to_string();

        // Set RelayStatus to `Connecting`
        self.set_status(RelayStatus::Connecting).await;
        log::debug!("Connecting to {}", url);

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
                    Err(e) => log::error!(
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
                log::info!("Connected to {}", url);

                self.stats.new_success();

                let relay = self.clone();
                thread::spawn(async move {
                    log::debug!("Relay Event Thread Started");
                    let mut rx = relay.relay_receiver.lock().await;
                    while let Some((relay_event, oneshot_sender)) = rx.recv().await {
                        match relay_event {
                            RelayEvent::SendMsg(msg) => {
                                log::debug!("Sending message {}", msg.as_json());
                                if let Err(e) = ws_tx.send(WsMessage::Text(msg.as_json())).await {
                                    log::error!(
                                        "Impossible to send msg to {}: {}",
                                        relay.url(),
                                        e.to_string()
                                    );
                                    if let Some(sender) = oneshot_sender {
                                        if let Err(e) = sender.send(false) {
                                            log::error!("Impossible to send oneshot msg: {}", e);
                                        }
                                    }
                                    break;
                                };
                                if let Some(sender) = oneshot_sender {
                                    if let Err(e) = sender.send(true) {
                                        log::error!("Impossible to send oneshot msg: {}", e);
                                    }
                                }
                            }
                            RelayEvent::Close => {
                                let _ = ws_tx.close().await;
                                relay.set_status(RelayStatus::Disconnected).await;
                                log::info!("Disconnected from {}", url);
                                break;
                            }
                            RelayEvent::Stop => {
                                if relay.is_scheduled_for_stop() {
                                    let _ = ws_tx.close().await;
                                    relay.set_status(RelayStatus::Stopped).await;
                                    relay.schedule_for_stop(false);
                                    log::info!("Stopped {}", url);
                                    break;
                                }
                            }
                            RelayEvent::Terminate => {
                                if relay.is_scheduled_for_termination() {
                                    // Unsubscribe from relay
                                    if let Err(e) = relay.unsubscribe(false).await {
                                        log::error!(
                                            "Impossible to unsubscribe from {}: {}",
                                            relay.url(),
                                            e.to_string()
                                        )
                                    }
                                    // Close stream
                                    let _ = ws_tx.close().await;
                                    relay.set_status(RelayStatus::Terminated).await;
                                    relay.schedule_for_termination(false);
                                    log::info!("Completely disconnected from {}", url);
                                    break;
                                }
                            }
                        }
                    }
                    log::debug!("Exited from Relay Event Thread");
                });

                let relay = self.clone();
                thread::spawn(async move {
                    log::debug!("Relay Message Thread Started");

                    async fn func(relay: &Relay, data: Vec<u8>) -> bool {
                        match String::from_utf8(data) {
                            Ok(data) => match RelayMessage::from_json(&data) {
                                Ok(msg) => {
                                    log::trace!("Received message to {}: {:?}", relay.url, msg);
                                    if let Err(err) = relay
                                        .pool_sender
                                        .send(RelayPoolMessage::ReceivedMsg {
                                            relay_url: relay.url(),
                                            msg,
                                        })
                                        .await
                                    {
                                        log::error!(
                                            "Impossible to send ReceivedMsg to pool: {}",
                                            &err
                                        );
                                        return true; // Exit
                                    };
                                }
                                Err(err) => {
                                    log::error!("{}: {}", err, data);
                                }
                            },
                            Err(err) => log::error!("{}", err),
                        }

                        false
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    while let Some(msg_res) = ws_rx.next().await {
                        if let Ok(msg) = msg_res {
                            let data: Vec<u8> = msg.into_data();
                            let exit: bool = func(&relay, data).await;
                            if exit {
                                break;
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

                    log::debug!("Exited from Message Thread of {}", relay.url);

                    if let Err(err) = relay.disconnect().await {
                        log::error!("Impossible to disconnect {}: {}", relay.url, err);
                    }
                });

                // Subscribe to relay
                if self.opts.read() {
                    if let Err(e) = self.resubscribe(false).await {
                        match e {
                            Error::FiltersEmpty => log::debug!("Filters empty for {}", self.url()),
                            _ => log::error!(
                                "Impossible to subscribe to {}: {}",
                                self.url(),
                                e.to_string()
                            ),
                        }
                    }
                }
            }
            Err(err) => {
                self.set_status(RelayStatus::Disconnected).await;
                log::error!("Impossible to connect to {}: {}", url, err);
            }
        };
    }

    async fn send_relay_event(
        &self,
        relay_msg: RelayEvent,
        sender: Option<oneshot::Sender<bool>>,
    ) -> Result<(), Error> {
        time::timeout(Some(Duration::from_secs(60)), async {
            self.relay_sender
                .send((relay_msg, sender))
                .await
                .map_err(|_| Error::MessagetNotSent)
        })
        .await
        .ok_or(Error::ChannelTimeout)??;
        Ok(())
    }

    /// Disconnect from relay and set status to 'Disconnected'
    async fn disconnect(&self) -> Result<(), Error> {
        let status = self.status().await;
        if status.ne(&RelayStatus::Disconnected)
            && status.ne(&RelayStatus::Stopped)
            && status.ne(&RelayStatus::Terminated)
        {
            self.send_relay_event(RelayEvent::Close, None).await?;
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
            self.send_relay_event(RelayEvent::Stop, None).await?;
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
            self.send_relay_event(RelayEvent::Terminate, None).await?;
        }
        Ok(())
    }

    /// Send msg to relay
    ///
    /// if `wait` arg is true, this method will wait for the msg to be sent
    pub async fn send_msg(&self, msg: ClientMessage, wait: bool) -> Result<(), Error> {
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

        if wait {
            let (tx, rx) = oneshot::channel::<bool>();
            self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)), Some(tx))
                .await?;
            match time::timeout(Some(Duration::from_secs(60)), rx).await {
                Some(result) => match result {
                    Ok(val) => {
                        if val {
                            Ok(())
                        } else {
                            Err(Error::MessagetNotSent)
                        }
                    }
                    Err(_) => Err(Error::OneShotRecvError),
                },
                _ => Err(Error::RecvTimeout),
            }
        } else {
            self.send_relay_event(RelayEvent::SendMsg(Box::new(msg)), None)
                .await
        }
    }

    /// Subscribes relay with existing filter
    async fn resubscribe(&self, wait: bool) -> Result<SubscriptionId, Error> {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }
        let subscription = self.subscription().await;

        if subscription.filters.is_empty() {
            return Err(Error::FiltersEmpty);
        }

        self.send_msg(
            ClientMessage::new_req(subscription.id.clone(), subscription.filters),
            wait,
        )
        .await?;
        Ok(subscription.id)
    }

    /// Subscribe
    pub async fn subscribe(
        &self,
        filters: Vec<Filter>,
        wait: bool,
    ) -> Result<SubscriptionId, Error> {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        self.update_subscription_filters(filters).await;
        self.resubscribe(wait).await
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self, wait: bool) -> Result<(), Error> {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let subscription = self.subscription().await;
        self.send_msg(ClientMessage::close(subscription.id), wait)
            .await?;
        Ok(())
    }

    /// Get stored events of filters with custom callback
    pub async fn get_stored_events_of_with_callback<F>(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
        callback: impl Fn(Event) -> F,
    ) -> Result<(), Error>
    where
        F: Future<Output = ()>,
    {
        if !self.opts.read() {
            return Err(Error::ReadDisabled);
        }

        let id = SubscriptionId::generate();

        self.send_msg(ClientMessage::new_req(id.clone(), filters), false)
            .await?;

        let mut notifications = self.notification_sender.subscribe();
        time::timeout(timeout, async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Message(_, msg) = notification {
                    match msg {
                        RelayMessage::Event {
                            subscription_id,
                            event,
                        } => {
                            if subscription_id.eq(&id) {
                                callback(*event).await;
                            }
                        }
                        RelayMessage::EndOfStoredEvents(subscription_id) => {
                            if subscription_id.eq(&id) {
                                break;
                            }
                        }
                        _ => log::debug!("Receive unhandled message {msg:?} on get_stored_events_of_with_callback"),
                    };
                }
            }
        })
        .await
        .ok_or(Error::Timeout)?;

        // Unsubscribe
        self.send_msg(ClientMessage::close(id), false).await?;

        Ok(())
    }

    /// Get stored events of filters
    pub async fn get_stored_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        let events: Mutex<Vec<Event>> = Mutex::new(Vec::new());
        self.get_stored_events_of_with_callback(filters, timeout, |event| async {
            let mut events = events.lock().await;
            events.push(event);
        })
        .await?;
        Ok(events.into_inner())
    }

    /// Request events of filter. All events will be sent to notification listener,
    /// until the EOSE "end of stored events" message is received from the relay.
    pub fn req_stored_events_of(&self, filters: Vec<Filter>, timeout: Option<Duration>) {
        if !self.opts.read() {
            log::error!("{}", Error::ReadDisabled);
        }

        let relay = self.clone();
        thread::spawn(async move {
            let id = SubscriptionId::generate();

            // Subscribe
            if let Err(e) = relay
                .send_msg(ClientMessage::new_req(id.clone(), filters), false)
                .await
            {
                log::error!(
                    "Impossible to send REQ to {}: {}",
                    relay.url(),
                    e.to_string()
                );
            };

            let mut notifications = relay.notification_sender.subscribe();
            if time::timeout(timeout, async {
                while let Ok(notification) = notifications.recv().await {
                    if let RelayPoolNotification::Message(
                        _,
                        RelayMessage::EndOfStoredEvents(subscription_id),
                    ) = notification
                    {
                        if subscription_id.eq(&id) {
                            break;
                        }
                    }
                }
            })
            .await
            .is_none()
            {
                log::error!("{}", Error::Timeout);
            }

            // Unsubscribe
            if let Err(e) = relay.send_msg(ClientMessage::close(id), false).await {
                log::error!(
                    "Impossible to close subscription with {}: {}",
                    relay.url(),
                    e.to_string()
                );
            }
        });
    }
}
