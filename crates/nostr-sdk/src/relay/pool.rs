// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Relay Pool

use std::collections::{HashMap, VecDeque};
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::thread;
use nostr::url::Url;
use nostr::{ClientMessage, Event, EventId, Filter, RelayMessage};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, Mutex};

use super::options::RelayPoolOptions;
use super::{Error as RelayError, FilterOptions, Relay, RelayOptions};

/// [`RelayPool`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Relay error
    #[error(transparent)]
    Relay(#[from] RelayError),
    /// No relays
    #[error("no relays")]
    NoRelays,
    /// Msg not sent
    #[error("msg not sent")]
    MsgNotSent,
    /// Relay not found
    #[error("relay not found")]
    RelayNotFound,
    /// Thread error
    #[error(transparent)]
    Thread(#[from] thread::Error),
}

/// Relay Pool Message
#[derive(Debug)]
pub enum RelayPoolMessage {
    /// Received new message
    ReceivedMsg {
        /// Relay url
        relay_url: Url,
        /// Relay message
        msg: RelayMessage,
    },
    /// Event sent
    EventSent(Box<Event>),
    /// Stop
    Stop,
    /// Shutdown
    Shutdown,
}

/// Relay Pool Notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayPoolNotification {
    /// Received an [`Event`]. Does not include events sent by this client.
    Event(Url, Event),
    /// Received a [`RelayMessage`]. Includes messages wrapping events that were sent by this client.
    Message(Url, RelayMessage),
    /// Stop
    Stop,
    /// Shutdown
    Shutdown,
}

#[derive(Debug, Clone)]
struct RelayPoolTask {
    receiver: Arc<Mutex<Receiver<RelayPoolMessage>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    events: Arc<Mutex<VecDeque<EventId>>>,
    running: Arc<AtomicBool>,
    max_seen_events: usize,
}

impl RelayPoolTask {
    pub fn new(
        pool_task_receiver: Receiver<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
        max_seen_events: usize,
    ) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(pool_task_receiver)),
            events: Arc::new(Mutex::new(VecDeque::new())),
            notification_sender,
            running: Arc::new(AtomicBool::new(false)),
            max_seen_events,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn set_running_to(&self, value: bool) {
        let _ = self
            .running
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(value));
    }

    pub async fn clear_already_seen_events(&self) {
        let mut events = self.events.lock().await;
        events.clear();
    }

    pub fn run(&self) {
        if self.is_running() {
            log::warn!("Relay Pool Task is already running!")
        } else {
            log::debug!("RelayPoolTask Thread Started");
            self.set_running_to(true);
            let this = self.clone();
            thread::spawn(async move {
                let mut receiver = this.receiver.lock().await;
                while let Some(msg) = receiver.recv().await {
                    match msg {
                        RelayPoolMessage::ReceivedMsg { relay_url, msg } => {
                            let _ = this
                                .notification_sender
                                .send(RelayPoolNotification::Message(
                                    relay_url.clone(),
                                    msg.clone(),
                                ));

                            if let RelayMessage::Event { event, .. } = msg {
                                // Verifies if the event is valid
                                if event.verify().is_ok() {
                                    // Adds only new events
                                    if this.add_event(event.id).await {
                                        let notification = RelayPoolNotification::Event(
                                            relay_url,
                                            event.as_ref().clone(),
                                        );
                                        let _ = this.notification_sender.send(notification);
                                    }
                                }
                            }
                        }
                        RelayPoolMessage::EventSent(event) => {
                            this.add_event(event.id).await;
                        }
                        RelayPoolMessage::Stop => {
                            log::debug!("Received stop msg");
                            if let Err(e) =
                                this.notification_sender.send(RelayPoolNotification::Stop)
                            {
                                log::error!("Impossible to send STOP notification: {}", e);
                            }
                            this.set_running_to(false);
                            break;
                        }
                        RelayPoolMessage::Shutdown => {
                            log::debug!("Received shutdown msg");
                            if let Err(e) = this
                                .notification_sender
                                .send(RelayPoolNotification::Shutdown)
                            {
                                log::error!("Impossible to send SHUTDOWN notification: {}", e);
                            }
                            this.set_running_to(false);
                            receiver.close();
                            break;
                        }
                    }
                }

                log::debug!("Exited from RelayPoolTask thread");
            });
        }
    }

    async fn add_event(&self, event_id: EventId) -> bool {
        let mut events = self.events.lock().await;
        if events.contains(&event_id) {
            false
        } else {
            while events.len() >= self.max_seen_events {
                events.pop_front();
            }
            events.push_back(event_id);
            true
        }
    }
}

/// Relay Pool
#[derive(Debug, Clone)]
pub struct RelayPool {
    relays: Arc<Mutex<HashMap<Url, Relay>>>,
    pool_task_sender: Sender<RelayPoolMessage>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    filters: Arc<Mutex<Vec<Filter>>>,
    pool_task: RelayPoolTask,
}

impl RelayPool {
    /// Create new `RelayPool`
    pub fn new(opts: RelayPoolOptions) -> Self {
        let (notification_sender, _) = broadcast::channel(opts.notification_channel_size);
        let (pool_task_sender, pool_task_receiver) = mpsc::channel(opts.task_channel_size);

        let relay_pool_task = RelayPoolTask::new(
            pool_task_receiver,
            notification_sender.clone(),
            opts.task_max_seen_events,
        );

        let pool = Self {
            relays: Arc::new(Mutex::new(HashMap::new())),
            pool_task_sender,
            notification_sender,
            filters: Arc::new(Mutex::new(Vec::new())),
            pool_task: relay_pool_task,
        };

        pool.start();

        pool
    }

    /// Start [`RelayPoolTask`]
    pub fn start(&self) {
        self.pool_task.run();
    }

    /// Stop
    pub async fn stop(&self) -> Result<(), Error> {
        let relays = self.relays().await;
        for relay in relays.values() {
            relay.stop().await?;
        }
        if let Err(e) = self.pool_task_sender.send(RelayPoolMessage::Stop).await {
            log::error!("Impossible to send STOP message: {e}");
        }
        Ok(())
    }

    /// Check if [`RelayPool`] is running
    pub fn is_running(&self) -> bool {
        self.pool_task.is_running()
    }

    /// Completely shutdown pool
    pub async fn shutdown(self) -> Result<(), Error> {
        self.disconnect().await?;
        thread::spawn(async move {
            thread::sleep(Duration::from_secs(3)).await;
            let _ = self.pool_task_sender.send(RelayPoolMessage::Shutdown).await;
        });
        Ok(())
    }

    /// Clear already seen events
    pub async fn clear_already_seen_events(&self) {
        self.pool_task.clear_already_seen_events().await;
    }

    /// Get new notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.notification_sender.subscribe()
    }

    /// Get relays
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        let relays = self.relays.lock().await;
        relays.clone()
    }

    /// Get [`Relay`]
    pub async fn relay(&self, url: &Url) -> Result<Relay, Error> {
        let relays = self.relays.lock().await;
        relays.get(url).cloned().ok_or(Error::RelayNotFound)
    }

    /// Get subscription filters
    pub async fn subscription_filters(&self) -> Vec<Filter> {
        self.filters.lock().await.clone()
    }

    /// Update subscription filters
    async fn update_subscription_filters(&self, filters: Vec<Filter>) {
        let mut f = self.filters.lock().await;
        *f = filters;
    }

    /// Add new relay
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn add_relay(
        &self,
        url: Url,
        proxy: Option<SocketAddr>,
        opts: RelayOptions,
    ) -> Result<(), Error> {
        let mut relays = self.relays.lock().await;
        if !relays.contains_key(&url) {
            let relay = Relay::new(
                url,
                self.pool_task_sender.clone(),
                self.notification_sender.clone(),
                proxy,
                opts,
            );
            relays.insert(relay.url(), relay);
        }
        Ok(())
    }

    /// Add new relay
    #[cfg(target_arch = "wasm32")]
    pub async fn add_relay(&self, url: Url, opts: RelayOptions) -> Result<(), Error> {
        let mut relays = self.relays.lock().await;
        if !relays.contains_key(&url) {
            let relay = Relay::new(
                url,
                self.pool_task_sender.clone(),
                self.notification_sender.clone(),
                opts,
            );
            relays.insert(relay.url(), relay);
        }
        Ok(())
    }

    /// Disconnect and remove relay
    pub async fn remove_relay(&self, url: Url) -> Result<(), Error> {
        let mut relays = self.relays.lock().await;
        if let Some(relay) = relays.remove(&url) {
            self.disconnect_relay(&relay).await?;
        }
        Ok(())
    }

    /// Send client message
    pub async fn send_msg(&self, msg: ClientMessage, wait: Option<Duration>) -> Result<(), Error> {
        let relays = self.relays().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        if let ClientMessage::Event(event) = &msg {
            if let Err(e) = self
                .pool_task_sender
                .send(RelayPoolMessage::EventSent(event.clone()))
                .await
            {
                log::error!("{e}");
            };
        }

        let sent_to_at_least_one_relay: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();

        for (url, relay) in relays.into_iter() {
            let msg = msg.clone();
            let sent = sent_to_at_least_one_relay.clone();
            let handle = thread::spawn(async move {
                match relay.send_msg(msg, wait).await {
                    Ok(_) => {
                        let _ =
                            sent.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(true));
                    }
                    Err(e) => log::error!("Impossible to send msg to {url}: {e}"),
                }
            });
            handles.push(handle);
        }

        for handle in handles.into_iter().flatten() {
            handle.join().await?;
        }

        if !sent_to_at_least_one_relay.load(Ordering::SeqCst) {
            return Err(Error::MsgNotSent);
        }

        Ok(())
    }

    /// Send client message
    pub async fn send_msg_to(
        &self,
        url: Url,
        msg: ClientMessage,
        wait: Option<Duration>,
    ) -> Result<(), Error> {
        let relays = self.relays().await;
        if let Some(relay) = relays.get(&url) {
            relay.send_msg(msg, wait).await?;
            Ok(())
        } else {
            Err(Error::RelayNotFound)
        }
    }

    /// Subscribe to filters
    pub async fn subscribe(&self, filters: Vec<Filter>, wait: Option<Duration>) {
        let relays = self.relays().await;
        self.update_subscription_filters(filters.clone()).await;
        for relay in relays.values() {
            if let Err(e) = relay.subscribe(filters.clone(), wait).await {
                log::error!("{e}");
            }
        }
    }

    /// Unsubscribe from filters
    pub async fn unsubscribe(&self, wait: Option<Duration>) {
        let relays = self.relays().await;
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe(wait).await {
                log::error!("{e}");
            }
        }
    }

    /// Get events of filters
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let events: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        let relays = self.relays().await;
        for (url, relay) in relays.into_iter() {
            let filters = filters.clone();
            let events = events.clone();
            let handle = thread::spawn(async move {
                if let Err(e) = relay
                    .get_events_of_with_callback(filters, timeout, opts, |event| async {
                        events.lock().await.push(event);
                    })
                    .await
                {
                    log::error!("Failed to get events from {url}: {e}");
                }
            });
            handles.push(handle);
        }

        for handle in handles.into_iter().flatten() {
            handle.join().await?;
        }

        Ok(events.lock_owned().await.clone())
    }

    /// Request events of filter. All events will be sent to notification listener
    /// until the EOSE "end of stored events" message is received from the relay.
    pub async fn req_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
        opts: FilterOptions,
    ) {
        let relays = self.relays().await;
        for relay in relays.values() {
            relay.req_events_of(filters.clone(), timeout, opts);
        }
    }

    /// Connect to all added relays and keep connection alive
    pub async fn connect(&self, wait_for_connection: bool) {
        let relays = self.relays().await;
        for relay in relays.values() {
            self.connect_relay(relay, wait_for_connection).await;
        }
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) -> Result<(), Error> {
        let relays = self.relays().await;
        for relay in relays.values() {
            self.disconnect_relay(relay).await?;
        }
        Ok(())
    }

    /// Connect to relay
    pub async fn connect_relay(&self, relay: &Relay, wait_for_connection: bool) {
        let filters: Vec<Filter> = self.subscription_filters().await;
        relay.update_subscription_filters(filters).await;
        relay.connect(wait_for_connection).await;
    }

    /// Disconnect from relay
    pub async fn disconnect_relay(&self, relay: &Relay) -> Result<(), Error> {
        relay.terminate().await?;
        Ok(())
    }
}
