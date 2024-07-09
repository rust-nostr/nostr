// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::btree_set::IntoIter;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter::Rev;
use std::sync::Arc;
use std::time::Duration;

use async_utility::thread::JoinHandle;
use async_utility::{thread, time};
use atomic_destructor::AtomicDestroyer;
use nostr::{ClientMessage, Event, EventId, Filter, SubscriptionId, Timestamp, TryIntoUrl, Url};
use nostr_database::{DynNostrDatabase, IntoNostrDatabase, Order};
use tokio::sync::{broadcast, Mutex, RwLock};

use super::options::RelayPoolOptions;
use super::{Error, Output, RelayPoolNotification};
use crate::relay::options::{FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions};
use crate::relay::{Reconciliation, Relay, RelayBlacklist};
use crate::{util, SubscribeOptions};

#[derive(Debug, Clone)]
pub struct InternalRelayPool {
    database: Arc<DynNostrDatabase>,
    relays: Arc<RwLock<HashMap<Url, Relay>>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Vec<Filter>>>>,
    blacklist: RelayBlacklist,
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
            blacklist: RelayBlacklist::empty(),
            //opts,
        }
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

    pub fn blacklist(&self) -> RelayBlacklist {
        self.blacklist.clone()
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

    async fn update_pool_subscription(&self, id: SubscriptionId, filters: Vec<Filter>) {
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
        // Convert into url
        let url: Url = url.try_into_url()?;

        // Get relays
        let mut relays = self.relays.write().await;

        // Check if map already contains url
        if !relays.contains_key(&url) {
            // Compose new relay
            let relay = Relay::custom(url, self.database.clone(), self.blacklist.clone(), opts);

            // Set notification sender
            relay
                .set_notification_sender(Some(self.notification_sender.clone()))
                .await;

            // Set relay subscriptions
            let subscriptions = self.subscriptions().await;
            for (id, filters) in subscriptions.into_iter() {
                relay.inner.update_subscription(id, filters, false).await;
            }

            // Insert relay into map
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
            relay.disconnect().await?;
        }
        Ok(())
    }
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        let mut relays = self.relays.write().await;
        for relay in relays.values() {
            relay.disconnect().await?;
        }
        relays.clear();
        Ok(())
    }

    pub async fn send_msg(
        &self,
        msg: ClientMessage,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        let relays = self.relays().await;
        self.send_msg_to(relays.into_keys(), msg, opts).await
    }

    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        let relays = self.relays().await;
        self.batch_msg_to(relays.into_keys(), msgs, opts).await
    }

    pub async fn send_msg_to<I, U>(
        &self,
        urls: I,
        msg: ClientMessage,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error>
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
    ) -> Result<Output<()>, Error>
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
            Ok(Output::success(url, ()))
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let result: Arc<Mutex<Output<()>>> = Arc::new(Mutex::new(Output::default()));
            let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(urls.len());

            for (url, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
                let msgs: Vec<ClientMessage> = msgs.clone();
                let result: Arc<Mutex<Output<()>>> = result.clone();
                let handle: JoinHandle<()> = thread::spawn(async move {
                    match relay.batch_msg(msgs, opts).await {
                        Ok(_) => {
                            // Success, insert relay url in 'success' set result
                            let mut result = result.lock().await;
                            result.success.insert(url);
                        }
                        Err(e) => {
                            tracing::error!("Impossible to send msg to {url}: {e}");

                            // Failed, insert relay url in 'failed' map result
                            let mut result = result.lock().await;
                            result.failed.insert(url, Some(e.to_string()));
                        }
                    }
                })?;
                handles.push(handle);
            }

            for handle in handles.into_iter() {
                handle.join().await?;
            }

            let result: Output<()> = util::take_mutex_ownership(result).await;

            if result.success.is_empty() {
                return Err(Error::MsgNotSent);
            }

            Ok(result)
        }
    }

    pub async fn send_event(
        &self,
        event: Event,
        opts: RelaySendOptions,
    ) -> Result<Output<EventId>, Error> {
        let relays: HashMap<Url, Relay> = self.relays().await;
        self.send_event_to(relays.into_keys(), event, opts).await
    }

    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        let relays = self.relays().await;
        self.batch_event_to(relays.into_keys(), events, opts).await
    }

    pub async fn send_event_to<I, U>(
        &self,
        urls: I,
        event: Event,
        opts: RelaySendOptions,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let event_id: EventId = event.id;
        let output: Output<()> = self.batch_event_to(urls, vec![event], opts).await?;
        Ok(Output {
            val: event_id,
            success: output.success,
            failed: output.failed,
        })
    }

    pub async fn batch_event_to<I, U>(
        &self,
        urls: I,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error>
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
            Ok(Output::success(url, ()))
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let result: Arc<Mutex<Output<()>>> = Arc::new(Mutex::new(Output::default()));
            let mut handles = Vec::with_capacity(urls.len());

            for (url, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
                let events: Vec<Event> = events.clone();
                let result: Arc<Mutex<Output<()>>> = result.clone();
                let handle = thread::spawn(async move {
                    match relay.batch_event(events, opts).await {
                        Ok(_) => {
                            // Success, insert relay url in 'success' set result
                            let mut result = result.lock().await;
                            result.success.insert(url);
                        }
                        Err(e) => {
                            tracing::error!("Impossible to send event to {url}: {e}");

                            // Failed, insert relay url in 'failed' map result
                            let mut result = result.lock().await;
                            result.failed.insert(url, Some(e.to_string()));
                        }
                    }
                })?;
                handles.push(handle);
            }

            for handle in handles.into_iter() {
                handle.join().await?;
            }

            let result: Output<()> = util::take_mutex_ownership(result).await;

            if result.success.is_empty() {
                return Err(Error::EventNotPublished);
            }

            Ok(result)
        }
    }

    pub async fn subscribe(
        &self,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<SubscriptionId>, Error> {
        let id: SubscriptionId = SubscriptionId::generate();
        let output: Output<()> = self.subscribe_with_id(id.clone(), filters, opts).await?;
        Ok(Output {
            val: id,
            success: output.success,
            failed: output.failed,
        })
    }

    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error> {
        // Check if isn't auto-closing subscription
        if !opts.is_auto_closing() {
            // Update pool subscriptions
            self.update_pool_subscription(id.clone(), filters.clone())
                .await;
        }

        // Get relays
        let relays = self.relays().await;

        // Subscribe
        self.subscribe_with_id_to(relays.into_keys(), id, filters, opts)
            .await
    }

    pub async fn subscribe_to<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<SubscriptionId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let id: SubscriptionId = SubscriptionId::generate();
        let output: Output<()> = self
            .subscribe_with_id_to(urls, id.clone(), filters, opts)
            .await?;
        Ok(Output {
            val: id,
            success: output.success,
            failed: output.failed,
        })
    }

    pub async fn subscribe_with_id_to<I, U>(
        &self,
        urls: I,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error>
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

        // Get relays
        let relays: HashMap<Url, Relay> = self.relays().await;

        // Check if relays map is empty
        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // If passed only 1 url, not use threads
        if urls.len() == 1 {
            let url: Url = urls.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: &Relay = relays.get(&url).ok_or(Error::RelayNotFound)?;
            relay.subscribe_with_id(id, filters, opts).await?;
            Ok(Output::success(url, ()))
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let result: Arc<Mutex<Output<()>>> = Arc::new(Mutex::new(Output::default()));
            let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(urls.len());

            // Subscribe
            for (url, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
                let id: SubscriptionId = id.clone();
                let filters: Vec<Filter> = filters.clone();
                let result: Arc<Mutex<Output<()>>> = result.clone();
                let handle: JoinHandle<()> = thread::spawn(async move {
                    match relay.subscribe_with_id(id, filters, opts).await {
                        Ok(_) => {
                            // Success, insert relay url in 'success' set result
                            let mut result = result.lock().await;
                            result.success.insert(url);
                        }
                        Err(e) => {
                            tracing::error!("Impossible to subscribe to '{url}': {e}");

                            // Failed, insert relay url in 'failed' map result
                            let mut result = result.lock().await;
                            result.failed.insert(url, Some(e.to_string()));
                        }
                    }
                })?;
                handles.push(handle);
            }

            for handle in handles.into_iter() {
                handle.join().await?;
            }

            let result: Output<()> = util::take_mutex_ownership(result).await;

            if result.success.is_empty() {
                return Err(Error::NotSubscribed);
            }

            Ok(result)
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
                let handle = thread::spawn(async move {
                    relay.connect(connection_timeout).await;
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
                relay.connect(None).await;
            }
        }
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        let relays = self.relays().await;
        for relay in relays.into_values() {
            relay.disconnect().await?;
        }
        Ok(())
    }

    #[inline]
    pub async fn reconcile(
        &self,
        filter: Filter,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        let relays: HashMap<Url, Relay> = self.relays().await;
        self.reconcile_with(relays.into_keys(), filter, opts).await
    }

    #[inline]
    pub async fn reconcile_with<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let items: Vec<(EventId, Timestamp)> =
            self.database.negentropy_items(filter.clone()).await?;
        self.reconcile_advanced(urls, filter, items, opts).await
    }

    #[inline]
    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        let relays: HashMap<Url, Relay> = self.relays().await;
        self.reconcile_advanced(relays.into_keys(), filter, items, opts)
            .await
    }

    pub async fn reconcile_advanced<I, U>(
        &self,
        urls: I,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error>
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
            let res: Reconciliation = relay.reconcile_with_items(filter, items, opts).await?;
            Ok(Output::success(url, res))
        } else {
            let relays: HashMap<Url, Relay> = self.relays().await;

            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            // Filter relays and start query
            let result: Arc<Mutex<Output<Reconciliation>>> =
                Arc::new(Mutex::new(Output::default()));
            let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(urls.len());
            for (url, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
                let filter: Filter = filter.clone();
                let my_items: Vec<(EventId, Timestamp)> = items.clone();
                let result: Arc<Mutex<Output<Reconciliation>>> = result.clone();
                let handle: JoinHandle<()> = thread::spawn(async move {
                    match relay.reconcile_with_items(filter, my_items, opts).await {
                        Ok(rec) => {
                            // Success, insert relay url in 'success' set result
                            let mut result = result.lock().await;
                            result.success.insert(url);
                            result.merge(rec);
                        }
                        Err(e) => {
                            tracing::error!("Failed to get reconcile with {url}: {e}");

                            // Failed, insert relay url in 'failed' map result
                            let mut result = result.lock().await;
                            result.failed.insert(url, Some(e.to_string()));
                        }
                    }
                })?;
                handles.push(handle);
            }

            for handle in handles.into_iter() {
                handle.join().await?;
            }

            let result: Output<Reconciliation> = util::take_mutex_ownership(result).await;

            if result.success.is_empty() {
                return Err(Error::NegentropyReconciliationFailed);
            }

            Ok(result)
        }
    }
}
