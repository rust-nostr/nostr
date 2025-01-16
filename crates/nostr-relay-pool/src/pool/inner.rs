// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::futures_util::future;
use async_utility::task;
use atomic_destructor::AtomicDestroyer;
use nostr_database::prelude::*;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock, RwLockReadGuard};

use super::options::RelayPoolOptions;
use super::{Error, RelayPoolNotification};
use crate::relay::options::{RelayOptions, ReqExitPolicy};
use crate::relay::{FlagCheck, Relay};
use crate::shared::SharedState;
use crate::stream::ReceiverStream;
use crate::RelayServiceFlags;

type Relays = HashMap<RelayUrl, Relay>;

// Instead of wrap every field in an `Arc<T>`, which increases the number of atomic operations,
// put all fields that require an `Arc` here.
#[derive(Debug)]
pub(super) struct AtomicPrivateData {
    pub(super) relays: RwLock<Relays>,
    subscriptions: RwLock<HashMap<SubscriptionId, Vec<Filter>>>,
    shutdown: AtomicBool,
}

#[derive(Debug, Clone)]
pub struct InnerRelayPool {
    pub(super) state: SharedState,
    pub(super) atomic: Arc<AtomicPrivateData>,
    pub(super) notification_sender: broadcast::Sender<RelayPoolNotification>, // TODO: move to shared state?
    opts: RelayPoolOptions,
}

impl AtomicDestroyer for InnerRelayPool {
    fn on_destroy(&self) {
        let pool = self.clone();
        task::spawn(async move {
            match pool.shutdown().await {
                Ok(()) => tracing::debug!("Relay pool destroyed."),
                Err(e) => tracing::error!(error = %e, "Impossible to destroy pool."),
            }
        });
    }
}

impl InnerRelayPool {
    pub fn new(opts: RelayPoolOptions, state: SharedState) -> Self {
        let (notification_sender, _) = broadcast::channel(opts.notification_channel_size);

        Self {
            state,
            atomic: Arc::new(AtomicPrivateData {
                relays: RwLock::new(HashMap::new()),
                subscriptions: RwLock::new(HashMap::new()),
                shutdown: AtomicBool::new(false),
            }),
            notification_sender,
            opts,
        }
    }

    pub(super) fn is_shutdown(&self) -> bool {
        self.atomic.shutdown.load(Ordering::SeqCst)
    }

    pub async fn shutdown(&self) -> Result<(), Error> {
        // TODO: check if already shutdown

        // Disconnect and force remove all relays
        self.remove_all_relays(true).await?;

        // Send shutdown notification
        let _ = self
            .notification_sender
            .send(RelayPoolNotification::Shutdown);

        // Mark as shutdown
        self.atomic.shutdown.store(true, Ordering::SeqCst);

        Ok(())
    }

    pub(super) fn internal_relays_with_flag<'a>(
        &self,
        txn: &'a RwLockReadGuard<'a, Relays>,
        flag: RelayServiceFlags,
        check: FlagCheck,
    ) -> impl Iterator<Item = (&'a RelayUrl, &'a Relay)> + 'a {
        txn.iter().filter(move |(_, r)| r.flags().has(flag, check))
    }

    /// Get relays with `READ` or `WRITE` relays
    pub(super) async fn relay_urls(&self) -> Vec<RelayUrl> {
        let relays = self.atomic.relays.read().await;
        self.internal_relays_with_flag(
            &relays,
            RelayServiceFlags::READ | RelayServiceFlags::WRITE,
            FlagCheck::Any,
        )
        .map(|(k, ..)| k.clone())
        .collect()
    }

    pub(super) async fn read_relay_urls(&self) -> Vec<RelayUrl> {
        let relays = self.atomic.relays.read().await;
        self.internal_relays_with_flag(&relays, RelayServiceFlags::READ, FlagCheck::All)
            .map(|(k, ..)| k.clone())
            .collect()
    }

    pub(super) async fn write_relay_urls(&self) -> Vec<RelayUrl> {
        let relays = self.atomic.relays.read().await;
        self.internal_relays_with_flag(&relays, RelayServiceFlags::WRITE, FlagCheck::All)
            .map(|(k, ..)| k.clone())
            .collect()
    }

    pub(super) fn internal_relay<'a>(
        &self,
        txn: &'a RwLockReadGuard<'a, Relays>,
        url: &RelayUrl,
    ) -> Result<&'a Relay, Error> {
        txn.get(url).ok_or(Error::RelayNotFound)
    }

    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: RelayUrl = url.try_into_url()?;
        let relays = self.atomic.relays.read().await;
        self.internal_relay(&relays, &url).cloned()
    }

    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        self.atomic.subscriptions.read().await.clone()
    }

    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
        let subscriptions = self.atomic.subscriptions.read().await;
        subscriptions.get(id).cloned()
    }

    pub async fn save_subscription(&self, id: SubscriptionId, filters: Vec<Filter>) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        let current: &mut Vec<Filter> = subscriptions.entry(id).or_default();
        *current = filters;
    }

    pub(crate) async fn remove_subscription(&self, id: &SubscriptionId) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        subscriptions.remove(id);
    }

    pub(crate) async fn remove_all_subscriptions(&self) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
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
        let url: RelayUrl = url.try_into_url()?;

        // Check if the pool has been shutdown
        if self.is_shutdown() {
            return Err(Error::Shutdown);
        }

        // Get relays
        let mut relays = self.atomic.relays.write().await;

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
        let relay: Relay = Relay::internal_custom(url, self.state.clone(), opts);

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

    pub async fn get_or_add_relay(
        &self,
        url: RelayUrl,
        inherit_pool_subscriptions: bool,
        opts: RelayOptions,
    ) -> Result<Option<Relay>, Error> {
        match self.relay(&url).await {
            Ok(relay) => Ok(Some(relay)),
            Err(..) => {
                self.add_relay(url, inherit_pool_subscriptions, opts)
                    .await?;
                Ok(None)
            }
        }
    }

    async fn internal_remove_relay(
        &self,
        relays: &mut Relays,
        url: RelayUrl,
        force: bool,
    ) -> Result<(), Error> {
        // Remove relay
        let relay = relays.remove(&url).ok_or(Error::RelayNotFound)?;

        // If NOT force, check if has `GOSSIP` flag
        if !force {
            let flags = relay.flags();
            if flags.has_any(RelayServiceFlags::GOSSIP) {
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

        Ok(())
    }

    pub async fn remove_relay<U>(&self, url: U, force: bool) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert into url
        let url: RelayUrl = url.try_into_url()?;

        // Acquire write lock
        let mut relays = self.atomic.relays.write().await;

        // Remove
        self.internal_remove_relay(&mut relays, url, force).await
    }

    pub async fn remove_all_relays(&self, force: bool) -> Result<(), Error> {
        // Acquire write lock
        let mut relays = self.atomic.relays.write().await;

        // Collect all relay urls
        let urls: Vec<RelayUrl> = relays.keys().cloned().collect();

        // Iter urls and remove relays
        for url in urls.into_iter() {
            self.internal_remove_relay(&mut relays, url, force).await?;
        }

        Ok(())
    }

    // Keep this methof here to avoif to have to `stealth_clone` before spawning the task (see `FULL RELAY CLONE` below)
    pub async fn stream_events_targeted<I, U>(
        &self,
        targets: I,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = (U, Vec<Filter>)>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        // Collect targets
        let targets: HashMap<RelayUrl, Vec<Filter>> = targets
            .into_iter()
            .map(|(u, f)| Ok((u.try_into_url()?, f)))
            .collect::<Result<_, Error>>()?;

        // Check if urls set is empty
        if targets.is_empty() {
            return Err(Error::NoRelaysSpecified);
        }

        // Lock with read shared access
        let relays = self.atomic.relays.read().await;

        // Check if empty
        if relays.is_empty() {
            return Err(Error::NoRelays);
        }

        // Check if urls set contains ONLY already added relays
        if !targets.keys().all(|url| relays.contains_key(url)) {
            return Err(Error::RelayNotFound);
        }

        // Drop
        drop(relays);

        // Create channel
        let (tx, rx) = mpsc::channel::<Event>(targets.len() * 512);

        // Spawn
        let this = self.clone(); // <-- FULL RELAY CLONE
        task::spawn(async move {
            // Lock with read shared access
            let relays = this.atomic.relays.read().await;

            let ids: Mutex<HashSet<EventId>> = Mutex::new(HashSet::new());

            let mut urls: Vec<RelayUrl> = Vec::with_capacity(targets.len());
            let mut futures = Vec::with_capacity(targets.len());

            // Filter relays and start query
            for (url, filters) in targets.into_iter() {
                match this.internal_relay(&relays, &url) {
                    Ok(relay) => {
                        urls.push(url);
                        futures.push(relay.fetch_events_with_callback(
                            filters,
                            timeout,
                            policy,
                            |event| async {
                                let mut ids = ids.lock().await;
                                if ids.insert(event.id) {
                                    drop(ids);
                                    let _ = tx.try_send(event);
                                }
                            },
                        ));
                    }
                    // TODO: remove this
                    Err(e) => tracing::error!("{e}"),
                }
            }

            // Join futures
            let list = future::join_all(futures).await;

            // Iter results
            for (url, result) in urls.into_iter().zip(list.into_iter()) {
                if let Err(e) = result {
                    tracing::error!(url = %url, error = %e, "Failed to stream events.");
                }
            }
        });

        // Return stream
        Ok(ReceiverStream::new(rx))
    }
}
