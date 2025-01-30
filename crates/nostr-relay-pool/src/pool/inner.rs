// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_utility::task;
use atomic_destructor::AtomicDestroyer;
use nostr_database::prelude::*;
use tokio::sync::{broadcast, RwLock};

use super::options::RelayPoolOptions;
use super::{Error, RelayPoolNotification};
use crate::relay::flags::RelayServiceFlags;
use crate::relay::options::RelayOptions;
use crate::relay::Relay;
use crate::shared::SharedState;

pub(super) type Relays = HashMap<RelayUrl, Relay>;

// Instead of wrap every field in an `Arc<T>`, which increases the number of atomic operations,
// put all fields that require an `Arc` here.
#[derive(Debug)]
pub(super) struct AtomicPrivateData {
    pub(super) relays: RwLock<Relays>,
    subscriptions: RwLock<HashMap<SubscriptionId, Filter>>,
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
        task::spawn(async move { pool.shutdown().await });
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

    pub async fn shutdown(&self) {
        if self.is_shutdown() {
            return;
        }

        // Disconnect and force remove all relays
        self.force_remove_all_relays().await;

        // Send shutdown notification
        let _ = self
            .notification_sender
            .send(RelayPoolNotification::Shutdown);

        // Mark as shutdown
        self.atomic.shutdown.store(true, Ordering::SeqCst);
    }

    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Filter> {
        self.atomic.subscriptions.read().await.clone()
    }

    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Filter> {
        let subscriptions = self.atomic.subscriptions.read().await;
        subscriptions.get(id).cloned()
    }

    pub async fn save_subscription(&self, id: SubscriptionId, filter: Filter) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        let current: &mut Filter = subscriptions.entry(id).or_default();
        *current = filter;
    }

    pub(crate) async fn remove_subscription(&self, id: &SubscriptionId) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        subscriptions.remove(id);
    }

    pub(crate) async fn remove_all_subscriptions(&self) {
        let mut subscriptions = self.atomic.subscriptions.write().await;
        subscriptions.clear();
    }

    pub async fn add_relay<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
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

        // If relay has `READ` flag, inherit pool subscriptions
        if relay.flags().has_read() {
            let subscriptions = self.subscriptions().await;
            for (id, filters) in subscriptions.into_iter() {
                relay.inner.update_subscription(id, filters, false).await;
            }
        }

        // Insert relay into map
        relays.insert(relay.url().clone(), relay);

        Ok(true)
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

        // Remove relay
        let relay: Relay = relays.remove(&url).ok_or(Error::RelayNotFound)?;

        // If NOT force, check if it has `GOSSIP` flag
        if !force {
            // If can't be removed, re-insert it.
            if !can_remove_relay(&relay) {
                relays.insert(url, relay);
                return Ok(());
            }
        }

        // Disconnect
        relay.disconnect();

        Ok(())
    }

    pub async fn remove_all_relays(&self) {
        // Acquire write lock
        let mut relays = self.atomic.relays.write().await;

        // Retains all relays that can't be removed
        relays.retain(|_, r| !can_remove_relay(r));
    }

    pub async fn force_remove_all_relays(&self) {
        // Acquire write lock
        let mut relays = self.atomic.relays.write().await;

        // Disconnect all relays
        for relay in relays.values() {
            relay.disconnect();
        }

        // Clear map
        relays.clear();
    }
}

/// Return `true` if the relay can be removed
///
/// If it CAN'T be removed,
/// the flags are automatically updated (remove `READ`, `WRITE` and `DISCOVERY` flags).
fn can_remove_relay(relay: &Relay) -> bool {
    let flags = relay.flags();
    if flags.has_any(RelayServiceFlags::GOSSIP) {
        // Remove READ, WRITE and DISCOVERY flags
        flags.remove(
            RelayServiceFlags::READ | RelayServiceFlags::WRITE | RelayServiceFlags::DISCOVERY,
        );

        // Relay has `GOSSIP` flag so it can't be removed.
        return false;
    }

    // Relay can be removed
    true
}
