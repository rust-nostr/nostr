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
use super::{RelayPoolBuilder, RelayPoolNotification};
use crate::relay::Relay;
use crate::shared::SharedState;

pub(super) type Relays = HashMap<RelayUrl, Relay>;

// Instead of wrap every field in an `Arc<T>`, which increases the number of atomic operations,
// put all fields that require an `Arc` here.
#[derive(Debug)]
pub(super) struct AtomicPrivateData {
    pub(super) relays: RwLock<Relays>,
    pub(super) subscriptions: RwLock<HashMap<SubscriptionId, Filter>>,
    pub(super) shutdown: AtomicBool,
}

#[derive(Debug, Clone)]
pub struct InnerRelayPool {
    pub(super) state: SharedState,
    pub(super) atomic: Arc<AtomicPrivateData>,
    pub(super) notification_sender: broadcast::Sender<RelayPoolNotification>, // TODO: move to shared state?
    pub(super) opts: RelayPoolOptions,
}

impl AtomicDestroyer for InnerRelayPool {
    fn on_destroy(&self) {
        let pool = self.clone();
        task::spawn(async move { pool.shutdown().await });
    }
}

impl InnerRelayPool {
    pub(super) fn from_builder(builder: RelayPoolBuilder) -> Self {
        let (notification_sender, _) = broadcast::channel(builder.opts.notification_channel_size);

        Self {
            state: SharedState::new(
                builder.__database,
                builder.websocket_transport,
                builder.__signer,
                builder.admit_policy,
                builder.opts.nip42_auto_authentication,
                builder.monitor,
            ),
            atomic: Arc::new(AtomicPrivateData {
                relays: RwLock::new(HashMap::new()),
                subscriptions: RwLock::new(HashMap::new()),
                shutdown: AtomicBool::new(false),
            }),
            notification_sender,
            opts: builder.opts,
        }
    }

    pub async fn shutdown(&self) {
        // Mark as shutdown
        // If the previous value was `true`,
        // meaning that was already shutdown, immediately returns.
        if self.atomic.shutdown.swap(true, Ordering::SeqCst) {
            return;
        }

        // Disconnect and force remove all relays
        self.force_remove_all_relays().await;

        // Send shutdown notification
        let _ = self
            .notification_sender
            .send(RelayPoolNotification::Shutdown);
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
