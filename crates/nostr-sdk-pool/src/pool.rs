// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::thread;
use nostr::message::MessageHandleError;
use nostr::nips::nip01::Coordinate;
use nostr::{
    event, ClientMessage, Event, EventId, Filter, JsonUtil, MissingPartialEvent, PartialEvent,
    RawRelayMessage, RelayMessage, SubscriptionId, Timestamp, TryIntoUrl, Url,
};
use nostr_database::{DatabaseError, DynNostrDatabase, IntoNostrDatabase, MemoryDatabase, Order};
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::limits::Limits;
use crate::options::{
    FilterOptions, NegentropyOptions, RelayOptions, RelayPoolOptions, RelaySendOptions,
};
use crate::relay::{Error as RelayError, InternalSubscriptionId, Relay, RelayStatus};

/// [`RelayPool`] error
#[derive(Debug, Error)]
pub enum Error {
    /// Url parse error
    #[error("impossible to parse URL: {0}")]
    Url(#[from] nostr::types::url::ParseError),
    /// Relay error
    #[error(transparent)]
    Relay(#[from] RelayError),
    /// Event error
    #[error(transparent)]
    Event(#[from] event::Error),
    /// Partial Event error
    #[error(transparent)]
    PartialEvent(#[from] event::partial::Error),
    /// Message handler error
    #[error(transparent)]
    MessageHandler(#[from] MessageHandleError),
    /// Database error
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// Thread error
    #[error(transparent)]
    Thread(#[from] thread::Error),
    /// No relays
    #[error("no relays")]
    NoRelays,
    /// No relays specified
    #[error("no relays sepcified")]
    NoRelaysSpecified,
    /// Msg not sent
    #[error("message not sent")]
    MsgNotSent,
    /// Msgs not sent
    #[error("messages not sent")]
    MsgsNotSent,
    /// Event/s not published
    #[error("event/s not published")]
    EventNotPublished,
    /// Relay not found
    #[error("relay not found")]
    RelayNotFound,
    /// Event expired
    #[error("event expired")]
    EventExpired,
}

/// Relay Pool Message
#[derive(Debug)]
pub enum RelayPoolMessage {
    /// Received new message
    ReceivedMsg {
        /// Relay url
        relay_url: Url,
        /// Relay message
        msg: RawRelayMessage,
    },
    /// Relay status changed
    RelayStatus {
        /// Relay url
        relay_url: Url,
        /// Relay Status
        status: RelayStatus,
    },
    /// Stop
    Stop,
    /// Shutdown
    Shutdown,
}

/// Relay Pool Notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayPoolNotification {
    /// Received an [`Event`]. Does not include events sent by this client.
    Event {
        /// Relay url
        relay_url: Url,
        /// Event
        event: Event,
    },
    /// Received a [`RelayMessage`]. Includes messages wrapping events that were sent by this client.
    Message {
        /// Relay url
        relay_url: Url,
        /// Relay Message
        message: RelayMessage,
    },
    /// Relay status changed
    RelayStatus {
        /// Relay url
        relay_url: Url,
        /// Relay Status
        status: RelayStatus,
    },
    /// Stop
    Stop,
    /// Shutdown
    Shutdown,
}

#[derive(Debug, Clone)]
struct RelayPoolTask {
    database: Arc<DynNostrDatabase>,
    receiver: Arc<Mutex<Receiver<RelayPoolMessage>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    running: Arc<AtomicBool>,
}

impl RelayPoolTask {
    pub fn new(
        database: Arc<DynNostrDatabase>,
        pool_task_receiver: Receiver<RelayPoolMessage>,
        notification_sender: broadcast::Sender<RelayPoolNotification>,
    ) -> Self {
        Self {
            database,
            receiver: Arc::new(Mutex::new(pool_task_receiver)),
            notification_sender,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn set_running_to(&self, value: bool) {
        self.running.store(value, Ordering::SeqCst);
    }

    pub fn run(&self) {
        if self.is_running() {
            tracing::warn!("Relay Pool Task is already running!")
        } else {
            tracing::debug!("RelayPoolTask Thread Started");
            self.set_running_to(true);
            let this = self.clone();
            let _ = thread::spawn(async move {
                let mut receiver = this.receiver.lock().await;
                while let Some(msg) = receiver.recv().await {
                    match msg {
                        RelayPoolMessage::ReceivedMsg { relay_url, msg } => {
                            match this.handle_relay_message(relay_url.clone(), msg).await {
                                Ok(Some(msg)) => {
                                    let _ = this.notification_sender.send(
                                        RelayPoolNotification::Message {
                                            relay_url: relay_url.clone(),
                                            message: msg.clone(),
                                        },
                                    );

                                    match msg {
                                        RelayMessage::Notice { message } => {
                                            tracing::warn!("Notice from {relay_url}: {message}")
                                        }
                                        RelayMessage::Ok {
                                            event_id,
                                            status,
                                            message,
                                        } => {
                                            tracing::debug!("Received OK from {relay_url} for event {event_id}: status={status}, message={message}");
                                        }
                                        _ => (),
                                    }
                                }
                                Ok(None) => (),
                                Err(e) => tracing::error!(
                                    "Impossible to handle relay message from {relay_url}: {e}"
                                ),
                            }
                        }
                        RelayPoolMessage::RelayStatus { relay_url, status } => {
                            let _ = this
                                .notification_sender
                                .send(RelayPoolNotification::RelayStatus { relay_url, status });
                        }
                        RelayPoolMessage::Stop => {
                            tracing::debug!("Received stop msg");
                            this.set_running_to(false);
                            if let Err(e) =
                                this.notification_sender.send(RelayPoolNotification::Stop)
                            {
                                tracing::error!("Impossible to send STOP notification: {e}");
                            }
                            break;
                        }
                        RelayPoolMessage::Shutdown => {
                            tracing::debug!("Received shutdown msg");
                            this.set_running_to(false);
                            receiver.close();
                            if let Err(e) = this
                                .notification_sender
                                .send(RelayPoolNotification::Shutdown)
                            {
                                tracing::error!("Impossible to send SHUTDOWN notification: {}", e);
                            }
                            break;
                        }
                    }
                }

                tracing::debug!("Exited from RelayPoolTask thread");
            });
        }
    }

    #[tracing::instrument(skip(self), level = "trace")]
    async fn handle_relay_message(
        &self,
        relay_url: Url,
        msg: RawRelayMessage,
    ) -> Result<Option<RelayMessage>, Error> {
        match msg {
            RawRelayMessage::Event {
                subscription_id,
                event,
            } => {
                // Deserialize partial event (id, pubkey and sig)
                let partial_event: PartialEvent = PartialEvent::from_json(event.to_string())?;

                // Check if event has been deleted
                if self
                    .database
                    .has_event_id_been_deleted(&partial_event.id)
                    .await?
                {
                    tracing::warn!(
                        "Received event {} that was deleted: type=id, relay_url={relay_url}",
                        partial_event.id
                    );
                    return Ok(None);
                }

                // Deserialize missing event fields
                let missing: MissingPartialEvent =
                    MissingPartialEvent::from_json(event.to_string())?;

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
                            "Received event {} that was deleted: type=coordinate, relay_url={relay_url}",
                            partial_event.id
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
                    .event_id_seen(partial_event.id, relay_url.clone())
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

                // If not seen, send RelayPoolNotification::Event
                if !seen {
                    let _ = self.notification_sender.send(RelayPoolNotification::Event {
                        relay_url,
                        event: event.clone(),
                    });
                }

                // Compose RelayMessage
                Ok(Some(RelayMessage::Event {
                    subscription_id: SubscriptionId::new(subscription_id),
                    event: Box::new(event),
                }))
            }
            m => Ok(Some(RelayMessage::try_from(m)?)),
        }
    }
}

/// Relay Pool
#[derive(Debug, Clone)]
pub struct RelayPool {
    database: Arc<DynNostrDatabase>,
    relays: Arc<RwLock<HashMap<Url, Relay>>>,
    pool_task_sender: Sender<RelayPoolMessage>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    filters: Arc<RwLock<Vec<Filter>>>,
    pool_task: RelayPoolTask,
    opts: RelayPoolOptions,
    dropped: Arc<AtomicBool>,
}

impl Drop for RelayPool {
    fn drop(&mut self) {
        if self.opts.shutdown_on_drop {
            if self.dropped.load(Ordering::SeqCst) {
                tracing::warn!("Relay Pool already dropped");
            } else {
                tracing::debug!("Dropping the Relay Pool...");
                self.dropped.store(true, Ordering::SeqCst);
                let pool = self.clone();
                let _ = thread::spawn(async move {
                    pool.shutdown()
                        .await
                        .expect("Impossible to drop the relay pool")
                });
            }
        }
    }
}

impl RelayPool {
    /// Create new `RelayPool`
    pub fn new(opts: RelayPoolOptions) -> Self {
        Self::with_database(opts, Arc::new(MemoryDatabase::default()))
    }

    /// New with database
    pub fn with_database<D>(opts: RelayPoolOptions, database: D) -> Self
    where
        D: IntoNostrDatabase,
    {
        let (notification_sender, _) = broadcast::channel(opts.notification_channel_size);
        let (pool_task_sender, pool_task_receiver) = mpsc::channel(opts.task_channel_size);

        let database: Arc<DynNostrDatabase> = database.into_nostr_database();

        let relay_pool_task = RelayPoolTask::new(
            database.clone(),
            pool_task_receiver,
            notification_sender.clone(),
        );

        let pool = Self {
            database,
            relays: Arc::new(RwLock::new(HashMap::new())),
            pool_task_sender,
            notification_sender,
            filters: Arc::new(RwLock::new(Vec::new())),
            pool_task: relay_pool_task,
            opts,
            dropped: Arc::new(AtomicBool::new(false)),
        };

        pool.start();

        pool
    }

    /// Start Relay Pool Task
    pub fn start(&self) {
        self.pool_task.run();
    }

    /// Stop
    pub async fn stop(&self) -> Result<(), Error> {
        let relays = self.relays().await;
        for relay in relays.values() {
            relay.stop().await?;
        }
        if let Err(e) = self.pool_task_sender.try_send(RelayPoolMessage::Stop) {
            tracing::error!("Impossible to send STOP message: {e}");
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
        })?;
        Ok(())
    }

    /// Get new notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.notification_sender.subscribe()
    }

    /// Get database
    pub fn database(&self) -> Arc<DynNostrDatabase> {
        self.database.clone()
    }

    /// Get relays
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        let relays = self.relays.read().await;
        relays.clone()
    }

    async fn internal_relay(&self, url: &Url) -> Result<Relay, Error> {
        let relays = self.relays.read().await;
        relays.get(url).cloned().ok_or(Error::RelayNotFound)
    }

    /// Get [`Relay`]
    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: Url = url.try_into_url()?;
        self.internal_relay(&url).await
    }

    /// Get subscription filters
    pub async fn subscription_filters(&self) -> Vec<Filter> {
        self.filters.read().await.clone()
    }

    /// Update subscription filters
    async fn update_subscription_filters(&self, filters: Vec<Filter>) {
        let mut f = self.filters.write().await;
        *f = filters;
    }

    /// Add new relay
    pub async fn add_relay<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: Url = url.try_into_url()?;
        let mut relays = self.relays.write().await;
        if !relays.contains_key(&url) {
            let relay = Relay::new(
                url,
                self.database.clone(),
                self.pool_task_sender.clone(),
                self.notification_sender.clone(),
                opts,
                Limits::default(),
            );
            relays.insert(relay.url(), relay);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Disconnect and remove relay
    pub async fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: Url = url.try_into_url()?;
        let mut relays = self.relays.write().await;
        if let Some(relay) = relays.remove(&url) {
            self.disconnect_relay(&relay).await?;
        }
        Ok(())
    }

    /// Disconnect and remove all relays
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        let mut relays = self.relays.write().await;
        for relay in relays.values() {
            self.disconnect_relay(relay).await?;
        }
        relays.clear();
        Ok(())
    }

    /// Send client message
    pub async fn send_msg(&self, msg: ClientMessage, opts: RelaySendOptions) -> Result<(), Error> {
        let relays = self.relays().await;
        self.send_msg_to(relays.into_keys(), msg, opts).await
    }

    /// Send multiple client messages at once
    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        let relays = self.relays().await;
        self.batch_msg_to(relays.into_keys(), msgs, opts).await
    }

    /// Send client message to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn send_msg_to<I, U>(
        &self,
        urls: I,
        msg: ClientMessage,
        opts: RelaySendOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.batch_msg_to(urls, vec![msg], opts).await
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Compose URLs
        let urls: HashSet<Url> = urls
            .into_iter()
            .map(|u| u.try_into_url())
            .collect::<Result<_, _>>()?;

        // Check if urls set isn't empty
        if urls.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Save events into database
        for msg in msgs.iter() {
            if let ClientMessage::Event(event) = msg {
                self.database.save_event(event).await?;
            }
        }

        // Get relays
        let relays: HashMap<Url, Relay> = self.relays().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // If passed only 1 url, not use threads
        if urls.len() == 1 {
            let url: Url = urls.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: &Relay = relays.get(&url).ok_or(Error::RelayNotFound)?;
            relay.batch_msg(msgs, opts).await?;
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let sent_to_at_least_one_relay: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
            let mut handles = Vec::with_capacity(urls.len());

            for (url, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
                let msgs = msgs.clone();
                let sent = sent_to_at_least_one_relay.clone();
                let handle = thread::spawn(async move {
                    match relay.batch_msg(msgs, opts).await {
                        Ok(_) => {
                            sent.store(true, Ordering::SeqCst);
                        }
                        Err(e) => tracing::error!("Impossible to send msg to {url}: {e}"),
                    }
                })?;
                handles.push(handle);
            }

            for handle in handles.into_iter() {
                handle.join().await?;
            }

            if !sent_to_at_least_one_relay.load(Ordering::SeqCst) {
                return Err(Error::MsgNotSent);
            }
        }

        Ok(())
    }

    /// Send event and wait for `OK` relay msg
    pub async fn send_event(&self, event: Event, opts: RelaySendOptions) -> Result<EventId, Error> {
        let relays: HashMap<Url, Relay> = self.relays().await;
        self.send_event_to(relays.into_keys(), event, opts).await
    }

    /// Send multiple [`Event`] at once
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        let relays = self.relays().await;
        self.batch_event_to(relays.into_keys(), events, opts).await
    }

    /// Send event to a specific relays
    pub async fn send_event_to<I, U>(
        &self,
        urls: I,
        event: Event,
        opts: RelaySendOptions,
    ) -> Result<EventId, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let event_id: EventId = event.id;
        self.batch_event_to(urls, vec![event], opts).await?;
        Ok(event_id)
    }

    /// Send event to a specific relays
    pub async fn batch_event_to<I, U>(
        &self,
        urls: I,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Compose URLs
        let urls: HashSet<Url> = urls
            .into_iter()
            .map(|u| u.try_into_url())
            .collect::<Result<_, _>>()?;

        // Check if urls set isn't empty
        if urls.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Save events into database
        for event in events.iter() {
            self.database.save_event(event).await?;
        }

        // Get relays
        let relays: HashMap<Url, Relay> = self.relays().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // If passed only 1 url, not use threads
        if urls.len() == 1 {
            let url: Url = urls.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: &Relay = relays.get(&url).ok_or(Error::RelayNotFound)?;
            relay.batch_event(events, opts).await?;
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let sent_to_at_least_one_relay: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
            let mut handles = Vec::with_capacity(urls.len());

            for (url, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
                let events = events.clone();
                let sent = sent_to_at_least_one_relay.clone();
                let handle = thread::spawn(async move {
                    match relay.batch_event(events, opts).await {
                        Ok(_) => {
                            sent.store(true, Ordering::SeqCst);
                        }
                        Err(e) => tracing::error!("Impossible to send event to {url}: {e}"),
                    }
                })?;
                handles.push(handle);
            }

            for handle in handles.into_iter() {
                handle.join().await?;
            }

            if !sent_to_at_least_one_relay.load(Ordering::SeqCst) {
                return Err(Error::EventNotPublished);
            }
        }

        Ok(())
    }

    /// Subscribe to filters
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Pool`
    pub async fn subscribe(&self, filters: Vec<Filter>, opts: RelaySendOptions) {
        let relays = self.relays().await;
        self.update_subscription_filters(filters.clone()).await;
        for relay in relays.values() {
            if let Err(e) = relay
                .subscribe_with_internal_id(InternalSubscriptionId::Pool, filters.clone(), opts)
                .await
            {
                tracing::error!("{e}");
            }
        }
    }

    /// Unsubscribe from filters
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Pool`
    pub async fn unsubscribe(&self, opts: RelaySendOptions) {
        let relays = self.relays().await;
        for relay in relays.values() {
            if let Err(e) = relay
                .unsubscribe_with_internal_id(InternalSubscriptionId::Pool, opts)
                .await
            {
                tracing::error!("{e}");
            }
        }
    }

    /// Get events of filters
    ///
    /// Get events both from **local database** and **relays**
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let relays = self.relays().await;
        self.get_events_from(relays.into_keys(), filters, timeout, opts)
            .await
    }

    /// Get events of filters from specific relays
    ///
    /// Get events both from **local database** and **relays**
    ///
    /// If no relay is specified, will be queried only the database.
    pub async fn get_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let urls: HashSet<Url> = urls
            .into_iter()
            .map(|u| u.try_into_url())
            .collect::<Result<_, _>>()?;

        if urls.is_empty() {
            Ok(self.database.query(filters, Order::Desc).await?)
        } else if urls.len() == 1 {
            let url: Url = urls.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: Relay = self.internal_relay(&url).await?;
            Ok(relay.get_events_of(filters, timeout, opts).await?)
        } else {
            let relays: HashMap<Url, Relay> = self.relays().await;

            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let stored_events: Vec<Event> = self
                .database
                .query(filters.clone(), Order::Desc)
                .await
                .unwrap_or_default();

            // Compose IDs and Events collections
            let ids: Arc<Mutex<HashSet<EventId>>> =
                Arc::new(Mutex::new(stored_events.iter().map(|e| e.id()).collect()));
            let events: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(stored_events));

            // Filter relays and start query
            let mut handles = Vec::with_capacity(urls.len());
            for (url, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
                let filters = filters.clone();
                let ids = ids.clone();
                let events = events.clone();
                let handle = thread::spawn(async move {
                    if let Err(e) = relay
                        .get_events_of_with_callback(filters, timeout, opts, |event| async {
                            let mut ids = ids.lock().await;
                            if !ids.contains(&event.id()) {
                                let mut events = events.lock().await;
                                ids.insert(event.id());
                                events.push(event);
                            }
                        })
                        .await
                    {
                        tracing::error!("Failed to get events from {url}: {e}");
                    }
                })?;
                handles.push(handle);
            }

            // Join threads
            for handle in handles.into_iter() {
                handle.join().await?;
            }

            Ok(events.lock_owned().await.clone())
        }
    }

    /// Request events of filter.
    ///
    /// If the events aren't already stored in the database, will be sent to notification listener
    /// until the EOSE "end of stored events" message is received from the relay.
    pub async fn req_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) {
        let relays = self.relays().await;
        for relay in relays.values() {
            relay.req_events_of(filters.clone(), timeout, opts);
        }
    }

    /// Request events of filter from specific relays.
    ///
    /// If the events aren't already stored in the database, will be sent to notification listener
    /// until the EOSE "end of stored events" message is received from the relay.
    pub async fn req_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let urls: HashSet<Url> = urls
            .into_iter()
            .map(|u| u.try_into_url())
            .collect::<Result<_, _>>()?;
        let relays: HashMap<Url, Relay> = self.relays().await;
        for (_, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
            relay.req_events_of(filters.clone(), timeout, opts);
        }
        Ok(())
    }

    /// Connect to all added relays and keep connection alive
    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        let relays: HashMap<Url, Relay> = self.relays().await;

        if connection_timeout.is_some() {
            let mut handles = Vec::with_capacity(relays.len());

            for relay in relays.into_values() {
                let pool = self.clone();
                let handle = thread::spawn(async move {
                    pool.connect_relay(&relay, connection_timeout).await;
                });
                handles.push(handle);
            }

            for handle in handles.into_iter().flatten() {
                if let Err(e) = handle.join().await {
                    tracing::error!("Impossible to join thread: {e}")
                }
            }
        } else {
            for relay in relays.values() {
                self.connect_relay(relay, None).await;
            }
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
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Pool`
    pub async fn connect_relay(&self, relay: &Relay, connection_timeout: Option<Duration>) {
        let filters: Vec<Filter> = self.subscription_filters().await;
        relay
            .update_subscription_filters(InternalSubscriptionId::Pool, filters)
            .await;
        relay.connect(connection_timeout).await;
    }

    /// Disconnect from relay
    pub async fn disconnect_relay(&self, relay: &Relay) -> Result<(), Error> {
        relay.terminate().await?;
        Ok(())
    }

    /// Negentropy reconciliation
    pub async fn reconcile(&self, filter: Filter, opts: NegentropyOptions) -> Result<(), Error> {
        let items: Vec<(EventId, Timestamp)> =
            self.database.negentropy_items(filter.clone()).await?;
        self.reconcile_with_items(filter, items, opts).await
    }

    /// Negentropy reconciliation with custom items
    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<(), Error> {
        let mut handles = Vec::new();
        let relays = self.relays().await;
        for (url, relay) in relays.into_iter() {
            let filter = filter.clone();
            let my_items = items.clone();
            let handle = thread::spawn(async move {
                if let Err(e) = relay.reconcile(filter, my_items, opts).await {
                    tracing::error!("Failed to get reconcile with {url}: {e}");
                }
            })?;
            handles.push(handle);
        }

        for handle in handles.into_iter() {
            handle.join().await?;
        }

        Ok(())
    }
}
