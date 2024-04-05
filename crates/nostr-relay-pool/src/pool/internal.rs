// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::btree_set::IntoIter;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter::Rev;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::{thread, time};
use atomic_destructor::AtomicDestroyer;
use nostr::{ClientMessage, Event, EventId, Filter, SubscriptionId, Timestamp, TryIntoUrl, Url};
use nostr_database::{DynNostrDatabase, IntoNostrDatabase, Order};
use tokio::sync::{broadcast, Mutex, RwLock};

use super::options::RelayPoolOptions;
use super::{Error, RelayPoolNotification};
use crate::relay::options::{FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions};
use crate::relay::Relay;
use crate::SubscribeOptions;

#[derive(Debug, Clone)]
pub struct InternalRelayPool {
    database: Arc<DynNostrDatabase>,
    relays: Arc<RwLock<HashMap<Url, Relay>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Vec<Filter>>>>,
    // opts: RelayPoolOptions,
}

impl AtomicDestroyer for InternalRelayPool {
    fn name(&self) -> Option<String> {
        Some(String::from("Relay Pool"))
    }

    fn on_destroy(&self) {
        let pool = self.clone();
        let _ = thread::spawn(async move {
            if let Err(e) = pool.shutdown().await {
                tracing::error!("Impossible to shutdown Relay Pool: {e}");
            }
        });
    }
}

impl InternalRelayPool {
    pub fn with_database<D>(opts: RelayPoolOptions, database: D) -> Self
    where
        D: IntoNostrDatabase,
    {
        let (notification_sender, _) = broadcast::channel(opts.notification_channel_size);

        Self {
            database: database.into_nostr_database(),
            relays: Arc::new(RwLock::new(HashMap::new())),
            notification_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            //opts,
        }
    }

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

    pub async fn shutdown(&self) -> Result<(), Error> {
        // Disconnect all relays
        self.disconnect().await?;

        // Send shutdown notification
        time::timeout(Some(Duration::from_secs(1)), async move {
            let _ = self
                .notification_sender
                .send(RelayPoolNotification::Shutdown);
        })
        .await;

        tracing::info!("Relay pool shutdown");

        Ok(())
    }

    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.notification_sender.subscribe()
    }

    pub fn database(&self) -> Arc<DynNostrDatabase> {
        self.database.clone()
    }

    pub async fn relays(&self) -> HashMap<Url, Relay> {
        let relays = self.relays.read().await;
        relays.clone()
    }

    async fn internal_relay(&self, url: &Url) -> Result<Relay, Error> {
        let relays = self.relays.read().await;
        relays.get(url).cloned().ok_or(Error::RelayNotFound)
    }

    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: Url = url.try_into_url()?;
        self.internal_relay(&url).await
    }

    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        self.subscriptions.read().await.clone()
    }

    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.get(id).cloned()
    }

    async fn update_subscription(&self, id: SubscriptionId, filters: Vec<Filter>) {
        let mut subscriptions = self.subscriptions.write().await;
        let current: &mut Vec<Filter> = subscriptions.entry(id).or_default();
        *current = filters;
    }

    pub(crate) async fn remove_subscription(&self, id: &SubscriptionId) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(id);
    }

    pub(crate) async fn remove_all_subscriptions(&self) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.clear();
    }

    pub async fn add_relay<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: Url = url.try_into_url()?;
        let mut relays = self.relays.write().await;
        if !relays.contains_key(&url) {
            let relay = Relay::custom(url, self.database.clone(), opts);
            relay
                .set_notification_sender(Some(self.notification_sender.clone()))
                .await;
            relays.insert(relay.url(), relay);
            Ok(true)
        } else {
            Ok(false)
        }
    }

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
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        let mut relays = self.relays.write().await;
        for relay in relays.values() {
            relay.terminate().await?;
        }
        relays.clear();
        Ok(())
    }

    pub async fn send_msg(&self, msg: ClientMessage, opts: RelaySendOptions) -> Result<(), Error> {
        let relays = self.relays().await;
        self.send_msg_to(relays.into_keys(), msg, opts).await
    }

    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        let relays = self.relays().await;
        self.batch_msg_to(relays.into_keys(), msgs, opts).await
    }

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

        // Check if urls set is empty
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

    pub async fn send_event(&self, event: Event, opts: RelaySendOptions) -> Result<EventId, Error> {
        let relays: HashMap<Url, Relay> = self.relays().await;
        self.send_event_to(relays.into_keys(), event, opts).await
    }

    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        let relays = self.relays().await;
        self.batch_event_to(relays.into_keys(), events, opts).await
    }

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

        // Check if urls set is empty
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

    pub async fn subscribe(&self, filters: Vec<Filter>, opts: SubscribeOptions) -> SubscriptionId {
        let id: SubscriptionId = SubscriptionId::generate();
        self.subscribe_with_id(id.clone(), filters, opts).await;
        id
    }

    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) {
        // Get relays
        let relays = self.relays().await;

        // Check if isn't auto-closing subscription
        if !opts.is_auto_closing() {
            // Update pool subscriptions
            self.update_subscription(id.clone(), filters.clone()).await;
        }

        // Subscribe
        for relay in relays.values() {
            if let Err(e) = relay
                .subscribe_with_id(id.clone(), filters.clone(), opts)
                .await
            {
                tracing::error!("{e}");
            }
        }
    }

    pub async fn unsubscribe(&self, id: SubscriptionId, opts: RelaySendOptions) {
        let relays = self.relays().await;
        self.remove_subscription(&id).await;
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe(id.clone(), opts).await {
                tracing::error!("{e}");
            }
        }
    }

    pub async fn unsubscribe_all(&self, opts: RelaySendOptions) {
        let relays = self.relays().await;
        self.remove_all_subscriptions().await;
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe_all(opts).await {
                tracing::error!("{e}");
            }
        }
    }

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

        // Check if urls set is empty
        if urls.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        if urls.len() == 1 {
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

            // Lock events, iterate set and revert order (events are sorted in ascending order in the BTreeSet)
            let events: BTreeSet<Event> = events.lock().await.clone();
            let iter: Rev<IntoIter<Event>> = events.into_iter().rev();

            // Check how many filters are passed and return the limit
            let limit: Option<usize> = match (filters.len(), filters.first()) {
                (1, Some(filter)) => filter.limit,
                _ => None,
            };

            // Check limit
            match limit {
                Some(limit) => Ok(iter.take(limit).collect()),
                None => Ok(iter.collect()),
            }
        }
    }

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

    pub async fn disconnect(&self) -> Result<(), Error> {
        let relays = self.relays().await;
        for relay in relays.into_values() {
            relay.terminate().await?;
        }
        Ok(())
    }

    pub(crate) async fn connect_relay(&self, relay: &Relay, connection_timeout: Option<Duration>) {
        let subscriptions = self.subscriptions().await;
        for (id, filters) in subscriptions.into_iter() {
            relay.inner.update_subscription(id, filters).await;
        }
        relay.connect(connection_timeout).await;
    }

    pub async fn reconcile(&self, filter: Filter, opts: NegentropyOptions) -> Result<(), Error> {
        let items: Vec<(EventId, Timestamp)> =
            self.database.negentropy_items(filter.clone()).await?;
        self.reconcile_with_items(filter, items, opts).await
    }

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
                if let Err(e) = relay.reconcile_with_items(filter, my_items, opts).await {
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
