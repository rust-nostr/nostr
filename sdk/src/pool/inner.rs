// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::HashMap;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nostr_database::prelude::*;
use tokio::sync::{broadcast, RwLock};

use super::options::RelayPoolOptions;
use super::{can_remove_relay, RelayPoolBuilder, RelayPoolNotification};
use crate::relay::Relay;
use crate::shared::SharedState;

pub(super) type Relays = HashMap<RelayUrl, Relay>;

// Instead of wrap every field in an `Arc<T>`, which increases the number of atomic operations,
// put all fields that require an `Arc` here.
#[derive(Debug)]
pub(super) struct AtomicPrivateData {
    pub(super) relays: RwLock<Relays>,
    /// Map of subscriptions that will be inherited by new added relays.
    pub(super) inherit_subscriptions: RwLock<HashMap<SubscriptionId, Vec<Filter>>>,
    pub(super) shutdown: AtomicBool,
}

#[derive(Debug, Clone)]
pub struct InnerRelayPool {
    pub(super) state: SharedState,
    pub(super) atomic: Arc<AtomicPrivateData>,
    pub(super) notification_sender: broadcast::Sender<RelayPoolNotification>, // TODO: move to shared state?
    pub(super) opts: RelayPoolOptions,
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
                inherit_subscriptions: RwLock::new(HashMap::new()),
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
        self.remove_all_relays(true).await;

        // Send shutdown notification
        let _ = self
            .notification_sender
            .send(RelayPoolNotification::Shutdown);
    }

    // Disconnect and remove all relays
    pub async fn remove_all_relays(&self, force: bool) {
        // Acquire write lock
        let mut relays = self.atomic.relays.write().await;

        if force {
            // Make sure to disconnect all relays
            for relay in relays.values() {
                relay.disconnect();
            }

            // Clear map
            relays.clear();
        } else {
            // Drain the map to get owned keys and values
            let old_relays: Relays = mem::take(&mut *relays);

            for (url, relay) in old_relays {
                // Check if it can be removed
                if can_remove_relay(&relay) {
                    // Disconnect
                    relay.disconnect();
                } else {
                    // Re-insert into the map
                    relays.insert(url, relay);
                }
            }
        }
    }
}
