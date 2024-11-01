// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use async_utility::futures_util::{future, StreamExt};
use async_utility::thread;
use atomic_destructor::AtomicDestroyer;
use nostr::prelude::*;
use nostr_database::{DynNostrDatabase, Events, IntoNostrDatabase};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock, RwLockReadGuard};
use tokio_stream::wrappers::ReceiverStream;

use super::constants::MAX_CONNECTING_CHUNK;
use super::options::RelayPoolOptions;
use super::{Error, Output, RelayPoolNotification};
use crate::relay::options::{FilterOptions, RelayOptions, RelaySendOptions, SyncOptions};
use crate::relay::{FlagCheck, Reconciliation, Relay, RelayFiltering};
use crate::{RelayServiceFlags, SubscribeOptions};

type Relays = HashMap<Url, Relay>;

#[derive(Debug, Clone)]
pub struct InnerRelayPool {
    pub(super) database: Arc<DynNostrDatabase>,
    relays: Arc<RwLock<Relays>>,
    notification_sender: broadcast::Sender<RelayPoolNotification>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Vec<Filter>>>>,
    pub(super) filtering: RelayFiltering,
    opts: RelayPoolOptions,
}

impl AtomicDestroyer for InnerRelayPool {
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

impl InnerRelayPool {
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
            opts,
        }
    }

    pub async fn shutdown(&self) -> Result<(), Error> {
        // Disconnect all relays
        self.disconnect().await?;

        // Send shutdown notification
        let _ = self
            .notification_sender
            .send(RelayPoolNotification::Shutdown);

        tracing::info!("Relay pool shutdown");

        Ok(())
    }

    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.notification_sender.subscribe()
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
        txn.iter().filter(move |(_, r)| r.flags().has(flag, check))
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

    /// Get all relay urls
    async fn all_relay_urls(&self) -> Vec<Url> {
        let relays = self.relays.read().await;
        relays.keys().cloned().collect()
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
        url: &Url,
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

        // Check number fo relays and limit
        if let Some(max) = self.opts.max_relays {
            if relays.len() >= max {
                return Err(Error::TooManyRelays { limit: max });
            }
        }

        // Compose new relay
        let relay =
            Relay::internal_custom(url, self.database.clone(), self.filtering.clone(), opts);

        // Set notification sender
        relay
            .inner
            .set_notification_sender(self.notification_sender.clone())?;

        // Set relay subscriptions
        if inherit_pool_subscriptions {
            let subscriptions = self.subscriptions().await;
            for (id, filters) in subscriptions.into_iter() {
                relay.inner.update_subscription(id, filters, false).await;
            }
        }

        // Insert relay into map
        relays.insert(relay.url().clone(), relay);

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

    async fn internal_remove_relay<U>(
        &self,
        relays: &mut Relays,
        url: U,
        force: bool,
    ) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert into url
        let url: Url = url.try_into_url()?;

        // Remove relay
        if let Some(relay) = relays.remove(&url) {
            // If NOT force, check if has INBOX or OUTBOX flags
            if !force {
                let flags = relay.flags();
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
            relay.disconnect()?;
        }

        Ok(())
    }

    pub async fn remove_relay<U>(&self, url: U, force: bool) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Acquire write lock
        let mut relays = self.relays.write().await;
        self.internal_remove_relay(&mut relays, url, force).await
    }

    pub async fn remove_all_relays(&self, force: bool) -> Result<(), Error> {
        // Get all relay urls
        let urls = self.all_relay_urls().await;

        // Acquire write lock
        let mut relays = self.relays.write().await;

        // Iter urls and remove relays
        for url in urls.into_iter() {
            self.internal_remove_relay(&mut relays, url, force).await?;
        }

        Ok(())
    }

    pub async fn send_msg_to<I, U>(&self, urls: I, msg: ClientMessage) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.batch_msg_to(urls, vec![msg]).await
    }

    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Compose URLs
        let set: HashSet<Url> = urls
            .into_iter()
            .map(|u| u.try_into_url())
            .collect::<Result<_, _>>()?;

        // Check if urls set is empty
        if set.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.relays.read().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // Check if urls set contains ONLY already added relays
        if !set.iter().all(|url| relays.contains_key(url)) {
            return Err(Error::RelayNotFound);
        }

        // Save events
        for msg in msgs.iter() {
            if let ClientMessage::Event(event) = msg {
                self.database.save_event(event).await?;
            }
        }

        let mut urls: Vec<Url> = Vec::with_capacity(set.len());
        let mut futures = Vec::with_capacity(set.len());
        let mut output: Output<()> = Output::default();

        // Compose futures
        for url in set.into_iter() {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            let msgs: Vec<ClientMessage> = msgs.clone();
            urls.push(url);
            futures.push(relay.batch_msg(msgs));
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Iter results and construct output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(..) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                }
                Err(e) => {
                    tracing::error!("Impossible to send message to '{url}': {e}");
                    output.failed.insert(url, Some(e.to_string()));
                }
            }
        }

        if output.success.is_empty() {
            return Err(Error::MsgNotSent);
        }

        Ok(output)
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
        let set: HashSet<Url> = urls
            .into_iter()
            .map(|u| u.try_into_url())
            .collect::<Result<_, _>>()?;

        // Check if urls set is empty
        if set.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.relays.read().await;

        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // Check if urls set contains ONLY already added relays
        if !set.iter().all(|url| relays.contains_key(url)) {
            return Err(Error::RelayNotFound);
        }

        // Save events into database
        for event in events.iter() {
            self.database.save_event(event).await?;
        }

        let mut urls: Vec<Url> = Vec::with_capacity(set.len());
        let mut futures = Vec::with_capacity(set.len());
        let mut output: Output<()> = Output::default();

        // Compose futures
        for url in set.into_iter() {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            let events: Vec<Event> = events.clone();
            urls.push(url);
            futures.push(relay.batch_event(events, opts));
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Iter results and construct output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(..) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                }
                Err(e) => {
                    tracing::error!("Impossible to send event to '{url}': {e}");
                    output.failed.insert(url, Some(e.to_string()));
                }
            }
        }

        if output.success.is_empty() {
            return Err(Error::EventNotPublished);
        }

        Ok(output)
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

        // Check if urls set contains ONLY already added relays
        if !map.keys().all(|url| relays.contains_key(url)) {
            return Err(Error::RelayNotFound);
        }

        let mut urls: Vec<Url> = Vec::with_capacity(map.len());
        let mut futures = Vec::with_capacity(map.len());
        let mut output: Output<()> = Output::default();

        // Compose futures
        for (url, filters) in map.into_iter() {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            let id: SubscriptionId = id.clone();
            urls.push(url);
            futures.push(relay.subscribe_with_id(id, filters, opts));
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Iter results and construct output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(..) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                }
                Err(e) => {
                    tracing::error!("Impossible to subscribe to '{url}': {e}");
                    output.failed.insert(url, Some(e.to_string()));
                }
            }
        }

        if output.success.is_empty() {
            return Err(Error::NotSubscribed);
        }

        Ok(output)
    }

    pub async fn unsubscribe(&self, id: SubscriptionId) {
        // Remove subscription from pool
        self.remove_subscription(&id).await;

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Remove subscription from relays
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe(id.clone()).await {
                tracing::error!("{e}");
            }
        }
    }

    pub async fn unsubscribe_all(&self) {
        // Remove subscriptions from pool
        self.remove_all_subscriptions().await;

        // Lock with read shared access
        let relays = self.relays.read().await;

        // Unsubscribe relays
        for relay in relays.values() {
            if let Err(e) = relay.unsubscribe_all().await {
                tracing::error!("{e}");
            }
        }
    }

    #[inline]
    pub async fn sync(
        &self,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        let urls: Vec<Url> = self.relay_urls().await;
        self.sync_with(urls, filter, opts).await
    }

    #[inline]
    pub async fn sync_with<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Get items
        let items: Vec<(EventId, Timestamp)> =
            self.database.negentropy_items(filter.clone()).await?;

        // Compose filters
        let mut filters: HashMap<Filter, Vec<(EventId, Timestamp)>> = HashMap::with_capacity(1);
        filters.insert(filter, items);

        // Reconcile
        let targets = urls.into_iter().map(|u| (u, filters.clone()));
        self.sync_targeted(targets, opts).await
    }

    pub async fn sync_targeted<I, U>(
        &self,
        targets: I,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = (U, HashMap<Filter, Vec<(EventId, Timestamp)>>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Collect targets map
        // TODO: create hashmap with capacity
        let mut map: HashMap<Url, HashMap<Filter, Vec<(EventId, Timestamp)>>> = HashMap::new();
        for (url, value) in targets.into_iter() {
            map.insert(url.try_into_url()?, value);
        }

        // Check if urls set is empty
        if map.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

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

        // TODO: shared reconciliation output to avoid to request duplicates?

        let mut urls: Vec<Url> = Vec::with_capacity(map.len());
        let mut futures = Vec::with_capacity(map.len());
        let mut output: Output<Reconciliation> = Output::default();

        // Compose futures
        for (url, filters) in map.into_iter() {
            let relay: &Relay = self.internal_relay(&relays, &url)?;
            urls.push(url);
            futures.push(relay.sync_multi(filters, opts));
        }

        // Join futures
        let list = future::join_all(futures).await;

        // Iter results and construct output
        for (url, result) in urls.into_iter().zip(list.into_iter()) {
            match result {
                Ok(reconciliation) => {
                    // Success, insert relay url in 'success' set result
                    output.success.insert(url);
                    output.merge(reconciliation);
                }
                Err(e) => {
                    tracing::error!("Failed to sync events with '{url}': {e}");
                    output.failed.insert(url, Some(e.to_string()));
                }
            }
        }

        // Check if sync failed (no success)
        if output.success.is_empty() {
            return Err(Error::NegentropyReconciliationFailed);
        }

        Ok(output)
    }

    pub async fn fetch_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Events, Error> {
        let urls: Vec<Url> = self.read_relay_urls().await;
        self.fetch_events_from(urls, filters, timeout, opts).await
    }

    pub async fn fetch_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Events, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let mut events: Events = Events::new(&filters);

        // Stream events
        let mut stream = self
            .stream_events_from(urls, filters, timeout, opts)
            .await?;
        while let Some(event) = stream.next().await {
            events.insert(event);
        }

        Ok(events)
    }

    #[inline]
    pub async fn stream_events(
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

    // TODO: change target type to `HashMap<Url, Vec<Filter>>`?
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

        // Drop
        drop(relays);

        // Create channel
        let (tx, rx) = mpsc::channel::<Event>(map.len() * 512);

        // Spawn
        let this = self.clone();
        thread::spawn(async move {
            // Lock with read shared access
            let relays = this.relays.read().await;

            let ids: Mutex<HashSet<EventId>> = Mutex::new(HashSet::new());

            let mut urls: Vec<Url> = Vec::with_capacity(map.len());
            let mut futures = Vec::with_capacity(map.len());

            // Filter relays and start query
            for (url, filters) in map.into_iter() {
                match this.internal_relay(&relays, &url) {
                    Ok(relay) => {
                        urls.push(url);
                        futures.push(relay.fetch_events_with_callback(
                            filters,
                            timeout,
                            opts,
                            |event| async {
                                let mut ids = ids.lock().await;
                                if ids.insert(event.id) {
                                    drop(ids);
                                    let _ = tx.try_send(event);
                                }
                            },
                        ));
                    }
                    Err(e) => tracing::error!("{e}"),
                }
            }

            // Join futures
            let list = future::join_all(futures).await;

            // Iter results
            for (url, result) in urls.into_iter().zip(list.into_iter()) {
                if let Err(e) = result {
                    tracing::error!("Failed to stream events from '{url}': {e}");
                }
            }
        })?;

        // Return stream
        Ok(ReceiverStream::new(rx))
    }

    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        // Lock with read shared access
        let relays = self.relays.read().await;

        let mut futures = Vec::with_capacity(relays.len());

        // Filter only relays that can connect and compose futures
        for relay in relays.values().filter(|r| r.status().can_connect()) {
            futures.push(relay.connect(connection_timeout));
        }

        // Check number of futures
        if futures.len() <= MAX_CONNECTING_CHUNK {
            future::join_all(futures).await;
            return;
        }

        tracing::warn!(
            "Too many relays ({}). Connecting in chunks of {MAX_CONNECTING_CHUNK} relays...",
            futures.len()
        );

        // Join in chunks
        while !futures.is_empty() {
            let upper: usize = cmp::min(MAX_CONNECTING_CHUNK, futures.len());
            let chunk = futures.drain(..upper);
            future::join_all(chunk).await;
        }
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        // Lock with read shared access
        let relays = self.relays.read().await;

        // Iter values and disconnect
        for relay in relays.values() {
            relay.disconnect()?;
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
        relay.disconnect()?;

        Ok(())
    }
}
