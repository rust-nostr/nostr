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
use nostr::prelude::*;
use nostr_database::{DynNostrDatabase, IntoNostrDatabase};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock, RwLockReadGuard};
use tokio_stream::wrappers::ReceiverStream;

use super::options::RelayPoolOptions;
use super::{Error, Output, RelayPoolNotification};
use crate::relay::options::{FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions};
use crate::relay::{FlagCheck, Reconciliation, Relay, RelayFiltering};
use crate::{util, RelayServiceFlags, SubscribeOptions};

type Relays = HashMap<Url, Relay>;

#[derive(Debug, Clone)]
pub struct InternalRelayPool {
    database: Arc<DynNostrDatabase>,
    relays: Arc<RwLock<Relays>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Vec<Filter>>>>,
    filtering: RelayFiltering,
    //opts: RelayPoolOptions,
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
            filtering: RelayFiltering::new(opts.filtering_mode),
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

    pub fn filtering(&self) -> RelayFiltering {
        self.filtering.clone()
    }

    pub async fn all_relays(&self) -> HashMap<Url, Relay> {
        let relays = self.relays.read().await;
        relays.clone()
    }

    #[inline]
    fn internal_relays_with_flag<'a>(
        &self,
        txn: &'a RwLockReadGuard<'a, Relays>,
        flag: RelayServiceFlags,
        check: FlagCheck,
    ) -> impl Iterator<Item = (&'a Url, &'a Relay)> + 'a {
        txn.iter()
            .filter(move |(_, r)| r.flags_ref().has(flag, check))
    }

    /// Get relays that has `READ` or `WRITE` flags
    #[inline]
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        self.relays_with_flag(
            RelayServiceFlags::READ | RelayServiceFlags::WRITE,
            FlagCheck::Any,
        )
        .await
    }

    /// Get relays that have a certain [RelayServiceFlag] enabled
    pub async fn relays_with_flag(
        &self,
        flag: RelayServiceFlags,
        check: FlagCheck,
    ) -> HashMap<Url, Relay> {
        let relays = self.relays.read().await;
        self.internal_relays_with_flag(&relays, flag, check)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get relays with `READ` or `WRITE` relays
    async fn relay_urls(&self) -> Vec<Url> {
        let relays = self.relays.read().await;
        self.internal_relays_with_flag(
            &relays,
            RelayServiceFlags::READ | RelayServiceFlags::WRITE,
            FlagCheck::Any,
        )
        .map(|(k, ..)| k.clone())
        .collect()
    }

    async fn read_relay_urls(&self) -> Vec<Url> {
        let relays = self.relays.read().await;
        self.internal_relays_with_flag(&relays, RelayServiceFlags::READ, FlagCheck::All)
            .map(|(k, ..)| k.clone())
            .collect()
    }

    async fn write_relay_urls(&self) -> Vec<Url> {
        let relays = self.relays.read().await;
        self.internal_relays_with_flag(&relays, RelayServiceFlags::WRITE, FlagCheck::All)
            .map(|(k, ..)| k.clone())
            .collect()
    }

    #[inline]
    fn internal_relay<'a>(
        &self,
        txn: &'a RwLockReadGuard<'a, Relays>,
        url: &'a Url,
    ) -> Result<&'a Relay, Error> {
        txn.get(url).ok_or(Error::RelayNotFound)
    }

    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: Url = url.try_into_url()?;
        let relays = self.relays.read().await;
        self.internal_relay(&relays, &url).cloned()
    }

    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        self.subscriptions.read().await.clone()
    }

    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.get(id).cloned()
    }

    pub async fn save_subscription(&self, id: SubscriptionId, filters: Vec<Filter>) {
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

    pub async fn add_relay<U>(
        &self,
        url: U,
        inherit_pool_subscriptions: bool,
        opts: RelayOptions,
    ) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert into url
        let url: Url = url.try_into_url()?;

        // Get relays
        let mut relays = self.relays.write().await;

        // Check if map already contains url
        if relays.contains_key(&url) {
            return Ok(false);
        }

        // Compose new relay
        let relay =
            Relay::internal_custom(url, self.database.clone(), self.filtering.clone(), opts);

        // Set notification sender
        relay
            .set_notification_sender(Some(self.notification_sender.clone()))
            .await;

        // Set relay subscriptions
        if inherit_pool_subscriptions {
            let subscriptions = self.subscriptions().await;
            for (id, filters) in subscriptions.into_iter() {
                relay.inner.update_subscription(id, filters, false).await;
            }
        }

        // Insert relay into map
        relays.insert(relay.url(), relay);

        Ok(true)
    }

    pub async fn get_or_add_relay<U>(
        &self,
        url: U,
        inherit_pool_subscriptions: bool,
        opts: RelayOptions,
    ) -> Result<Option<Relay>, Error>
    where
        U: TryIntoUrl + Clone,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        match self.relay(url.clone()).await {
            Ok(relay) => Ok(Some(relay)),
            Err(..) => {
                self.add_relay(url, inherit_pool_subscriptions, opts)
                    .await?;
                Ok(None)
            }
        }
    }

    pub async fn remove_relay<U>(&self, url: U, force: bool) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert into url
        let url: Url = url.try_into_url()?;

        // Acquire write lock
        let mut relays = self.relays.write().await;

        // Remove relay
        if let Some(relay) = relays.remove(&url) {
            // It NOT force, check if has INBOX or OUTBOX flags
            if !force {
                let flags = relay.flags_ref();
                if flags.has_any(RelayServiceFlags::INBOX | RelayServiceFlags::OUTBOX) {
                    // Remove READ, WRITE and DISCOVERY flags
                    flags.remove(
                        RelayServiceFlags::READ
                            | RelayServiceFlags::WRITE
                            | RelayServiceFlags::DISCOVERY,
                    );

                    // Re-insert
                    relays.insert(url, relay);
                    return Ok(());
                }
            }

            // Disconnect
            relay.disconnect().await?;
        }

        Ok(())
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

        // Lock with read shared access
        let relays = self.relays.read().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // If passed only 1 url, not use threads
        if urls.len() == 1 {
            let url: Url = urls.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            relay.batch_msg(msgs, opts).await?;
            Ok(Output::success(url, ()))
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let result: Arc<Mutex<Output<()>>> = Arc::new(Mutex::new(Output::default()));
            let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(urls.len());

            for (url, relay) in relays.iter().filter(|(url, ..)| urls.contains(url)) {
                let url: Url = url.clone();
                let relay: Relay = relay.clone();
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
        let urls: Vec<Url> = self.write_relay_urls().await;
        self.send_event_to(urls, event, opts).await
    }

    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        let urls: Vec<Url> = self.write_relay_urls().await;
        self.batch_event_to(urls, events, opts).await
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

        // Lock with read shared access
        let relays = self.relays.read().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // If passed only 1 url, not use threads
        if urls.len() == 1 {
            let url: Url = urls.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            relay.batch_event(events, opts).await?;
            Ok(Output::success(url, ()))
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let result: Arc<Mutex<Output<()>>> = Arc::new(Mutex::new(Output::default()));
            let mut handles = Vec::with_capacity(urls.len());

            for (url, relay) in relays.iter().filter(|(url, ..)| urls.contains(url)) {
                let url: Url = url.clone();
                let relay: Relay = relay.clone();
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
            // Save subscription
            self.save_subscription(id.clone(), filters.clone()).await;
        }

        // Get relay urls
        let urls: Vec<Url> = self.read_relay_urls().await;

        // Subscribe
        self.subscribe_with_id_to(urls, id, filters, opts).await
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

    #[inline]
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
        let targets = urls.into_iter().map(|u| (u, filters.clone()));
        self.subscribe_targeted(id, targets, opts).await
    }

    pub async fn subscribe_targeted<I, U>(
        &self,
        id: SubscriptionId,
        targets: I,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = (U, Vec<Filter>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Collect targets map
        let mut map: HashMap<Url, Vec<Filter>> = HashMap::new();
        for (url, filters) in targets.into_iter() {
            map.insert(url.try_into_url()?, filters);
        }

        // Check if urls set is empty
        if map.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Check if relays map is empty
        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // If passed only 1 url, not use threads
        if map.len() == 1 {
            let (url, filters) = map.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            relay.subscribe_with_id(id, filters, opts).await?;
            Ok(Output::success(url, ()))
        } else {
            // Check if urls set contains ONLY already added relays
            if !map.keys().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            let result: Arc<Mutex<Output<()>>> = Arc::new(Mutex::new(Output::default()));
            let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(map.len());

            // Subscribe
            for (url, filters) in map.into_iter() {
                let relay: Relay = self.internal_relay(&relays, &url).cloned()?;
                let id: SubscriptionId = id.clone();
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
        // Remove subscription from pool
        self.remove_subscription(&id).await;

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Remove subscription from relays
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe(id.clone(), opts).await {
                tracing::error!("{e}");
            }
        }
    }

    pub async fn unsubscribe_all(&self, opts: RelaySendOptions) {
        // Remove subscriptions from pool
        self.remove_all_subscriptions().await;

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Unsubscribe relays
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe_all(opts).await {
                tracing::error!("{e}");
            }
        }
    }

    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let urls: Vec<Url> = self.read_relay_urls().await;
        self.get_events_from(urls, filters, timeout, opts).await
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

        // Lock with read shared access
        let relays = self.relays.read().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        if urls.len() == 1 {
            let url: Url = urls.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            Ok(relay.get_events_of(filters, timeout, opts).await?)
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            // Compose events collections
            let events: Arc<Mutex<BTreeSet<Event>>> = Arc::new(Mutex::new(BTreeSet::new()));

            // Filter relays and start query
            let mut handles = Vec::with_capacity(urls.len());
            for (url, relay) in relays.iter().filter(|(url, ..)| urls.contains(url)) {
                let url: Url = url.clone();
                let relay: Relay = relay.clone();
                let filters = filters.clone();
                let events = events.clone();
                let handle = thread::spawn(async move {
                    if let Err(e) = relay
                        .get_events_of_with_callback(filters, timeout, opts, |event| async {
                            let mut events = events.lock().await;
                            events.insert(event);
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
            let events: BTreeSet<Event> = util::take_mutex_ownership(events).await;
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

    #[inline]
    pub async fn stream_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<ReceiverStream<Event>, Error> {
        let urls: Vec<Url> = self.read_relay_urls().await;
        self.stream_events_from(urls, filters, timeout, opts).await
    }

    #[inline]
    pub async fn stream_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let targets = urls.into_iter().map(|u| (u, filters.clone()));
        self.stream_events_targeted(targets, timeout, opts).await
    }

    pub async fn stream_events_targeted<I, U>(
        &self,
        targets: I,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = (U, Vec<Filter>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Collect targets map
        let mut map: HashMap<Url, Vec<Filter>> = HashMap::new();
        for (url, filters) in targets.into_iter() {
            map.insert(url.try_into_url()?, filters);
        }

        // Check if urls set is empty
        if map.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        let (tx, rx) = mpsc::channel::<Event>(4096); // TODO: change to unbounded or allow to change this value?

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Check if empty
        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // Check if urls set contains ONLY already added relays
        if !map.keys().all(|url| relays.contains_key(url)) {
            return Err(Error::RelayNotFound);
        }

        // Compose events collections
        let ids: Arc<Mutex<HashSet<EventId>>> = Arc::new(Mutex::new(HashSet::new()));

        // Filter relays and start query
        for (url, filters) in map.into_iter() {
            let relay: Relay = self.internal_relay(&relays, &url).cloned()?;
            let tx = tx.clone();
            let ids = ids.clone();
            thread::spawn(async move {
                if let Err(e) = relay
                    .get_events_of_with_callback(filters, timeout, opts, |event| async {
                        let mut ids = ids.lock().await;
                        if ids.insert(event.id) {
                            drop(ids);
                            let _ = tx.try_send(event); // TODO: log error?
                        }
                    })
                    .await
                {
                    tracing::error!("Failed to stream events from '{url}': {e}");
                }
            })?;
        }

        Ok(ReceiverStream::new(rx))
    }

    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        // Lock with read shared access
        let relays = self.relays.read().await;

        match connection_timeout {
            Some(..) => {
                let mut handles = Vec::with_capacity(relays.len());

                // False positive
                // Relay is borrowed and then moved to a thread so MUST be cloned.
                #[allow(clippy::unnecessary_to_owned)]
                for relay in relays.values().cloned() {
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
            }
            None => {
                // Iter values and connect
                for relay in relays.values() {
                    relay.connect(None).await;
                }
            }
        }
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        // Lock with read shared access
        let relays = self.relays.read().await;

        // Iter values and disconnect
        for relay in relays.values() {
            relay.disconnect().await?;
        }

        Ok(())
    }

    pub async fn connect_relay<U>(
        &self,
        url: U,
        connection_timeout: Option<Duration>,
    ) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert url
        let url: Url = url.try_into_url()?;

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Get relay
        let relay: &Relay = self.internal_relay(&relays, &url)?;

        // Connect
        relay.connect(connection_timeout).await;

        Ok(())
    }

    pub async fn disconnect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert url
        let url: Url = url.try_into_url()?;

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Get relay
        let relay: &Relay = self.internal_relay(&relays, &url)?;

        // Disconnect
        relay.disconnect().await?;

        Ok(())
    }

    #[inline]
    pub async fn reconcile(
        &self,
        filter: Filter,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        let urls: Vec<Url> = self.relay_urls().await;
        self.reconcile_with(urls, filter, opts).await
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
        let urls: Vec<Url> = self.relay_urls().await;
        self.reconcile_advanced(urls, filter, items, opts).await
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

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Check if empty
        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        if urls.len() == 1 {
            let url: Url = urls.into_iter().next().ok_or(Error::RelayNotFound)?;
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            let res: Reconciliation = relay.reconcile_with_items(filter, items, opts).await?;
            Ok(Output::success(url, res))
        } else {
            // Check if urls set contains ONLY already added relays
            if !urls.iter().all(|url| relays.contains_key(url)) {
                return Err(Error::RelayNotFound);
            }

            // Filter relays and start query
            let result: Arc<Mutex<Output<Reconciliation>>> =
                Arc::new(Mutex::new(Output::default()));
            let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(urls.len());
            for (url, relay) in relays.iter().filter(|(url, ..)| urls.contains(url)) {
                let url: Url = url.clone();
                let relay: Relay = relay.clone();
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
