use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use async_utility::{task, time};
use nostr::PublicKey;
use nostr_gossip::{GossipListKind, GossipPublicKeyStatus, OutdatedPublicKey};
use tokio::sync::RwLock;

use super::Gossip;
use crate::client::{Client, Error, WeakClient};

#[derive(Debug)]
pub(super) struct GossipBackgroundRefresher {
    background_refresher_spawned: AtomicBool,
    // Round-robin cursors for background refresh selection.
    // We keep one per gossip kind so each list advances independently.
    nip65_refresh_cursor: AtomicUsize,
    nip17_refresh_cursor: AtomicUsize,
    nip65_seen_public_keys: RwLock<BTreeSet<PublicKey>>,
    nip17_seen_public_keys: RwLock<BTreeSet<PublicKey>>,
}

impl GossipBackgroundRefresher {
    pub(super) fn new() -> Self {
        Self {
            background_refresher_spawned: AtomicBool::new(false),
            nip65_refresh_cursor: AtomicUsize::new(0),
            nip17_refresh_cursor: AtomicUsize::new(0),
            nip65_seen_public_keys: RwLock::new(BTreeSet::new()),
            nip17_seen_public_keys: RwLock::new(BTreeSet::new()),
        }
    }

    #[cfg(test)]
    pub(super) fn is_background_refresher_spawned(&self) -> bool {
        self.background_refresher_spawned.load(Ordering::SeqCst)
    }

    pub(super) async fn track_public_keys<I>(&self, kind: &GossipListKind, public_keys: I)
    where
        I: IntoIterator<Item = PublicKey>,
    {
        match kind {
            GossipListKind::Nip65 => {
                let mut set = self.nip65_seen_public_keys.write().await;
                set.extend(public_keys);
            }
            GossipListKind::Nip17 => {
                let mut set = self.nip17_seen_public_keys.write().await;
                set.extend(public_keys);
            }
        }
    }

    pub(super) async fn tracked_public_keys(&self, kind: GossipListKind) -> BTreeSet<PublicKey> {
        match kind {
            GossipListKind::Nip65 => self.nip65_seen_public_keys.read().await.clone(),
            GossipListKind::Nip17 => self.nip17_seen_public_keys.read().await.clone(),
        }
    }

    /// Select up to `limit` tracked public keys using a round-robin window.
    pub(super) async fn next_tracked_public_keys_for_refresh(
        &self,
        kind: GossipListKind,
        limit: NonZeroUsize,
    ) -> Vec<PublicKey> {
        let limit: usize = limit.get();

        let tracked_public_keys: Vec<PublicKey> =
            self.tracked_public_keys(kind).await.into_iter().collect();

        if tracked_public_keys.is_empty() {
            return Vec::new();
        }

        let cursor: usize = match kind {
            GossipListKind::Nip65 => self
                .nip65_refresh_cursor
                .fetch_add(limit, Ordering::Relaxed),
            GossipListKind::Nip17 => self
                .nip17_refresh_cursor
                .fetch_add(limit, Ordering::Relaxed),
        };

        let start: usize = cursor % tracked_public_keys.len();
        let count: usize = tracked_public_keys.len().min(limit);
        let mut selected: Vec<PublicKey> = Vec::with_capacity(count);

        for offset in 0..count {
            let idx: usize = (start + offset) % tracked_public_keys.len();
            selected.push(tracked_public_keys[idx]);
        }

        selected
    }
}

impl Client {
    pub(in crate::client) fn spawn_gossip_background_refresher(&self) {
        let Some(background_refresh) = self.config().gossip_config.background_refresh else {
            return;
        };

        // Check if gossip is available
        match self.gossip() {
            // Gossip available
            Some(gossip) => {
                // Mark is as spawned and get the old value
                let is_spawned: bool = gossip
                    .refresher()
                    .background_refresher_spawned
                    .swap(true, Ordering::SeqCst);

                // If already spawned, return immediately
                if is_spawned {
                    return;
                }
            }
            // Gossip not available, return.
            None => return,
        }

        // Make a weak reference to the client.
        let weak: WeakClient = self.weak_clone();

        task::spawn(async move {
            tracing::info!("Background gossip refresher started.");

            // At this moment there are no relays in the pool yet, so sleep for a while
            #[cfg(not(test))]
            time::sleep(Duration::from_secs(60)).await;

            loop {
                // Check if we can upgrade the client.
                let Some(client) = weak.upgrade() else {
                    tracing::warn!("Can't upgrade the client, stopping background refresher...");
                    break;
                };

                // Check if the client has been shutdown.
                if client.is_shutdown() {
                    tracing::warn!("Client has been shutdown, stopping background refresher...");
                    break;
                }

                // Get gossip instance
                // SAFETY: we checked above that gossip is available.
                let gossip: &Gossip = client.gossip().expect("Client must have a gossip instance");

                // Sleep a bit for simulating an update
                #[cfg(test)]
                time::sleep(Duration::from_secs(3)).await;

                // Refresh NIP-65
                if let Err(e) = client
                    .refresh_gossip_list_kind_in_background(
                        gossip,
                        GossipListKind::Nip65,
                        background_refresh.max_public_keys_per_round,
                    )
                    .await
                {
                    tracing::error!(
                        error = %e,
                        "Failed to refresh NIP-65 public keys in background"
                    );
                }

                // Refresh NIP-17
                if let Err(e) = client
                    .refresh_gossip_list_kind_in_background(
                        gossip,
                        GossipListKind::Nip17,
                        background_refresh.max_public_keys_per_round,
                    )
                    .await
                {
                    tracing::error!(
                        error = %e,
                        "Failed to refresh NIP-17 public keys in background"
                    );
                }

                // IMPORTANT: drop the upgraded strong `Client` reference *before* sleeping!
                //
                // The task holds only a `WeakClient` across rounds and upgrades it at the top of each loop.
                // If this explicit drop is removed, `client` stays alive until the end of the iteration
                // (including the sleep), keeping one extra strong ref.
                // That can delay the final client shutdown-on-drop by up to one full interval.
                //
                // Keep this drop here!
                drop(client);

                // Sleep for the next round.
                time::sleep(background_refresh.interval).await;
            }

            tracing::info!("Background gossip refresher stopped.");
        });
    }

    async fn refresh_gossip_list_kind_in_background(
        &self,
        gossip: &Gossip,
        kind: GossipListKind,
        limit: NonZeroUsize,
    ) -> Result<(), Error> {
        let public_keys: BTreeSet<PublicKey> = self
            .select_background_refresh_public_keys(gossip, kind, limit)
            .await?;

        if public_keys.is_empty() {
            return Ok(());
        }

        tracing::debug!(kind = ?kind, limit = %limit, "Refreshing gossip list kind in background...");

        self.sync_gossip_public_keys(gossip, public_keys, &[kind])
            .await
    }

    async fn select_background_refresh_public_keys(
        &self,
        gossip: &Gossip,
        kind: GossipListKind,
        limit: NonZeroUsize,
    ) -> Result<BTreeSet<PublicKey>, Error> {
        let tracked_public_keys: Vec<PublicKey> = gossip
            .refresher()
            .next_tracked_public_keys_for_refresh(kind, limit)
            .await;

        let mut selected: BTreeSet<PublicKey> = BTreeSet::new();

        for public_key in tracked_public_keys {
            if selected.contains(&public_key) {
                continue;
            }

            let status: GossipPublicKeyStatus = gossip.store().status(&public_key, kind).await?;

            // Add if it's missing or outdated.
            if !status.is_updated() {
                selected.insert(public_key);
            }
        }

        if selected.len() < limit.get() {
            // Here instead of taking the remaining number of outdated keys, we use the limit directly.
            // This is because this may select also outdated keys that are already being tracked, resulting in an update with fewer keys than the limit.
            let outdated_public_keys: BTreeSet<OutdatedPublicKey> =
                gossip.store().outdated_public_keys(kind, limit).await?;

            // get the primitive type
            let limit: usize = limit.get();

            for pk in outdated_public_keys {
                if selected.len() >= limit {
                    break;
                }

                selected.insert(pk.public_key);
            }
        }

        Ok(selected)
    }
}
