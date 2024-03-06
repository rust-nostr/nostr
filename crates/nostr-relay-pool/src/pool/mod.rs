// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::{thread, time};
use nostr::message::MessageHandleError;
use nostr::{ClientMessage, Event, EventId, Filter, RelayMessage, Timestamp, TryIntoUrl, Url};
use nostr_database::{DatabaseError, DynNostrDatabase, IntoNostrDatabase, MemoryDatabase, Order};
use thiserror::Error;
use tokio::sync::{broadcast, Mutex, RwLock};

pub mod options;

use self::options::RelayPoolOptions;
use crate::relay::limits::Limits;
use crate::relay::options::{FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions};
use crate::relay::{Error as RelayError, InternalSubscriptionId, Relay, RelayStatus};
use crate::util::SaturatingUsize;

/// [`RelayPool`] error
#[derive(Debug, Error)]
pub enum Error {
    /// Url parse error
    #[error("impossible to parse URL: {0}")]
    Url(#[from] nostr::types::url::ParseError),
    /// Relay error
    #[error(transparent)]
    Relay(#[from] RelayError),
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

/// Relay Pool
#[derive(Debug)]
pub struct RelayPool {
    database: Arc<DynNostrDatabase>,
    relays: Arc<RwLock<HashMap<Url, Relay>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    filters: Arc<RwLock<Vec<Filter>>>,
    opts: RelayPoolOptions,
    shutdown: Arc<AtomicBool>,
    ref_counter: Arc<AtomicUsize>,
}

impl Default for RelayPool {
    fn default() -> Self {
        Self::new(RelayPoolOptions::default())
    }
}

impl RelayPool {
    fn internal_clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            relays: self.relays.clone(),
            notification_sender: self.notification_sender.clone(),
            filters: self.filters.clone(),
            opts: self.opts,
            shutdown: self.shutdown.clone(),
            ref_counter: self.ref_counter.clone(),
        }
    }
}

impl Clone for RelayPool {
    fn clone(&self) -> Self {
        // Increase counter
        let new_ref_counter: usize = self.ref_counter.saturating_increment(Ordering::SeqCst);
        tracing::debug!("Relay Pool cloned: ref counter increased to {new_ref_counter}");

        // Clone
        self.internal_clone()
    }
}

impl Drop for RelayPool {
    fn drop(&mut self) {
        // Check if already shutdown
        if self.shutdown.load(Ordering::SeqCst) {
            tracing::debug!("Relay Pool already shutdown");
        } else {
            // Decrease counter
            let new_ref_counter: usize = self.ref_counter.saturating_decrement(Ordering::SeqCst);
            tracing::debug!("Relay Pool dropped: ref counter decreased to {new_ref_counter}");

            // Check if it's time for shutdown
            if new_ref_counter == 0 {
                tracing::debug!("Shutting down Relay Pool...");

                // Mark as shutdown
                self.shutdown.store(true, Ordering::SeqCst);

                // Internally clone pool and shutdown
                let pool: RelayPool = self.internal_clone(); // TODO: avoid this, use InternalRelayPool?
                let _ = thread::spawn(async move {
                    // TODO: avoid this (cause drop recursion)
                    if let Err(e) = pool.shutdown().await {
                        tracing::error!("Impossible to shutdown Relay Pool: {e}");
                    }
                });

                tracing::info!("Relay Pool shutdown.");
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

        Self {
            database: database.into_nostr_database(),
            relays: Arc::new(RwLock::new(HashMap::new())),
            notification_sender,
            filters: Arc::new(RwLock::new(Vec::new())),
            opts,
            shutdown: Arc::new(AtomicBool::new(false)),
            ref_counter: Arc::new(AtomicUsize::new(1)),
        }
    }

    /// Stop
    ///
    /// Call `connect` to re-start relays connections
    pub async fn stop(&self) -> Result<(), Error> {
        let relays = self.relays().await;
        for relay in relays.values() {
            relay.stop().await?;
        }
        if let Err(e) = self.notification_sender.send(RelayPoolNotification::Stop) {
            tracing::error!("Impossible to send STOP notification: {e}");
        }
        Ok(())
    }

    /// Completely shutdown pool
    pub async fn shutdown(self) -> Result<(), Error> {
        // Disconnect all relays
        self.disconnect().await?;

        // Send shutdown notification
        thread::spawn(async move {
            time::timeout(Some(Duration::from_secs(3)), async move {
                let _ = self
                    .notification_sender
                    .send(RelayPoolNotification::Shutdown);
            })
            .await;
        })?;

        Ok(())
    }

    /// Get new **pool** notification listener
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
            let relay = Relay::custom(url, self.database.clone(), opts, Limits::default());
            relay
                .set_notification_sender(Some(self.notification_sender.clone()))
                .await;
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
            relay.terminate().await?;
        }
        Ok(())
    }

    /// Disconnect and remove all relays
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        let mut relays = self.relays.write().await;
        for relay in relays.values() {
            relay.terminate().await?;
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
            let events: Arc<Mutex<BTreeSet<Event>>> =
                Arc::new(Mutex::new(stored_events.into_iter().collect()));

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
                                events.insert(event);
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

            Ok(events
                .lock_owned()
                .await
                .clone()
                .into_iter()
                .rev()
                .collect())
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
        for relay in relays.into_values() {
            relay.terminate().await?;
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
