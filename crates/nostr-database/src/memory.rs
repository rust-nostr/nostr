// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Memory (RAM) Storage backend for Nostr apps

use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;

use async_trait::async_trait;
use lru::LruCache;
use nostr::nips::nip01::Coordinate;
use nostr::{Event, EventId, Filter, Timestamp, Url};
use tokio::sync::Mutex;

use crate::{Backend, DatabaseError, DatabaseIndexes, EventIndexResult, NostrDatabase, Order};

/// Database options
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryDatabaseOptions {
    /// Store events (default: false)
    pub events: bool,
    /// Max events and IDs to store in memory (default: 100_000)
    ///
    /// `None` means no limits.
    pub max_events: Option<usize>,
}

impl Default for MemoryDatabaseOptions {
    fn default() -> Self {
        Self {
            events: false,
            max_events: Some(100_000),
        }
    }
}

impl MemoryDatabaseOptions {
    /// New default database options
    pub fn new() -> Self {
        Self::default()
    }
}

/// Memory Database (RAM)
#[derive(Debug)]
pub struct MemoryDatabase {
    opts: MemoryDatabaseOptions,
    seen_event_ids: Arc<Mutex<LruCache<EventId, HashSet<Url>>>>,
    events: Arc<Mutex<LruCache<EventId, Event>>>,
    indexes: DatabaseIndexes,
}

impl Default for MemoryDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryDatabase {
    /// New Memory database with default options
    pub fn new() -> Self {
        Self::with_opts(MemoryDatabaseOptions::default())
    }

    /// New Memory database
    pub fn with_opts(opts: MemoryDatabaseOptions) -> Self {
        Self {
            opts,
            seen_event_ids: Arc::new(Mutex::new(new_lru_cache(opts.max_events))),
            events: Arc::new(Mutex::new(new_lru_cache(opts.max_events))),
            indexes: DatabaseIndexes::new(),
        }
    }

    fn _event_id_seen(
        &self,
        seen_event_ids: &mut LruCache<EventId, HashSet<Url>>,
        event_id: EventId,
        relay_url: Url,
    ) {
        match seen_event_ids.get_mut(&event_id) {
            Some(set) => {
                set.insert(relay_url);
            }
            None => {
                let mut set = HashSet::with_capacity(1);
                set.insert(relay_url);
                seen_event_ids.put(event_id, set);
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrDatabase for MemoryDatabase {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        Backend::Memory
    }

    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        if self.opts.events {
            let EventIndexResult {
                to_store,
                to_discard,
            } = self.indexes.index_event(event).await;

            if to_store {
                let mut events = self.events.lock().await;

                events.put(event.id(), event.clone());

                for event_id in to_discard.into_iter() {
                    events.pop(&event_id);
                }

                Ok(true)
            } else {
                tracing::warn!("Event {} not saved: unknown", event.id());
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    async fn bulk_import(&self, events: BTreeSet<Event>) -> Result<(), Self::Err> {
        if self.opts.events {
            let events = self.indexes.bulk_import(events).await;

            let mut e = self.events.lock().await;

            for event in events.into_iter() {
                e.put(event.id(), event);
            }

            Ok(())
        } else {
            Err(DatabaseError::FeatureDisabled)
        }
    }

    async fn has_event_already_been_saved(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        if self.indexes.has_event_id_been_deleted(event_id).await {
            Ok(true)
        } else if self.opts.events {
            let events = self.events.lock().await;
            Ok(events.contains(event_id))
        } else {
            Ok(false)
        }
    }

    async fn has_event_already_been_seen(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        let seen_event_ids = self.seen_event_ids.lock().await;
        Ok(seen_event_ids.contains(event_id))
    }

    async fn has_event_id_been_deleted(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        Ok(self.indexes.has_event_id_been_deleted(event_id).await)
    }

    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: Timestamp,
    ) -> Result<bool, Self::Err> {
        Ok(self
            .indexes
            .has_coordinate_been_deleted(coordinate, timestamp)
            .await)
    }

    async fn event_id_seen(&self, event_id: EventId, relay_url: Url) -> Result<(), Self::Err> {
        let mut seen_event_ids = self.seen_event_ids.lock().await;
        self._event_id_seen(&mut seen_event_ids, event_id, relay_url);
        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err> {
        let mut seen_event_ids = self.seen_event_ids.lock().await;
        Ok(seen_event_ids.get(&event_id).cloned())
    }

    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err> {
        if self.opts.events {
            let mut events = self.events.lock().await;
            events
                .get(&event_id)
                .cloned()
                .ok_or(DatabaseError::NotFound)
        } else {
            Err(DatabaseError::FeatureDisabled)
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, Self::Err> {
        Ok(self.indexes.count(filters).await)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn query(&self, filters: Vec<Filter>, order: Order) -> Result<Vec<Event>, Self::Err> {
        if self.opts.events {
            Ok(self.indexes.query(filters, order).await)
        } else {
            Err(DatabaseError::FeatureDisabled)
        }
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
        if self.opts.events {
            Ok(self.indexes.negentropy_items(filter).await)
        } else {
            Err(DatabaseError::FeatureDisabled)
        }
    }

    async fn delete(&self, filter: Filter) -> Result<(), Self::Err> {
        let mut events = self.events.lock().await;

        match self.indexes.delete(filter).await {
            Some(ids) => {
                for id in ids.into_iter() {
                    events.pop(&id);
                }
            }
            None => {
                events.clear();
            }
        };

        Ok(())
    }

    async fn wipe(&self) -> Result<(), Self::Err> {
        // Clear indexes
        self.indexes.clear().await;

        // Clear
        let mut seen_event_ids = self.seen_event_ids.lock().await;
        seen_event_ids.clear();
        let mut events = self.events.lock().await;
        events.clear();
        Ok(())
    }
}

fn new_lru_cache<K, V>(size: Option<usize>) -> LruCache<K, V>
where
    K: Hash + Eq,
{
    match size {
        Some(size) => match NonZeroUsize::new(size) {
            Some(size) => LruCache::new(size),
            None => LruCache::unbounded(),
        },
        None => LruCache::unbounded(),
    }
}
