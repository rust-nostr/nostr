// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use nostr::prelude::*;
use tokio::sync::RwLock;

// TODO: add support to DM relay list

#[derive(Debug, Clone)]
struct RelayListMetadata {
    pub map: HashMap<Url, Option<RelayMetadata>>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone)]
pub struct GossipGraph {
    /// Keep track of seen public keys and of their NIP-65
    public_keys: Arc<RwLock<HashMap<PublicKey, RelayListMetadata>>>,
}

impl GossipGraph {
    pub fn new() -> Self {
        Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update graph
    pub async fn update<I>(&self, events: I)
    where
        I: IntoIterator<Item = Event>,
    {
        let mut public_keys = self.public_keys.write().await;

        for event in events.into_iter().filter(|e| e.kind == Kind::RelayList) {
            public_keys
                .entry(event.pubkey)
                .and_modify(|m| {
                    // Update only if new metadata has more recent timestamp
                    if event.created_at >= m.timestamp {
                        *m = RelayListMetadata {
                            map: nip65::extract_relay_list(&event)
                                .map(|(u, m)| (u.clone(), *m))
                                .collect(),
                            timestamp: event.created_at,
                        };
                    }
                })
                .or_insert_with(|| RelayListMetadata {
                    map: nip65::extract_relay_list(&event)
                        .map(|(u, m)| (u.clone(), *m))
                        .collect(),
                    timestamp: event.created_at,
                });
        }
    }

    pub async fn get_nip65_relays<'a, I>(
        &self,
        public_keys: I,
        metadata: RelayMetadata,
    ) -> HashSet<Url>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let map = self.public_keys.read().await;

        let mut urls: HashSet<Url> = HashSet::new();

        for public_key in public_keys.into_iter() {
            if let Some(meta) = map.get(public_key) {
                for (url, m) in meta.map.iter() {
                    let insert: bool = match m {
                        Some(val) => val == &metadata,
                        None => true,
                    };

                    if insert {
                        urls.insert(url.clone());
                    }
                }
            }
        }

        urls
    }

    /// Get outbox (write) relays for public keys
    #[inline]
    pub async fn get_outbox_relays<'a, I>(&self, public_keys: I) -> HashSet<Url>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        self.get_nip65_relays(public_keys, RelayMetadata::Write)
            .await
    }

    /// Get inbox (read) relays for public keys
    #[inline]
    pub async fn get_inbox_relays<'a, I>(&self, public_keys: I) -> HashSet<Url>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        self.get_nip65_relays(public_keys, RelayMetadata::Read)
            .await
    }
}
