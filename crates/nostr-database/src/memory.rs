// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Memory (RAM) Storage backend for Nostr apps

use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use lru::LruCache;
use nostr::prelude::*;
use tokio::sync::Mutex;

use crate::collections::new_lru_cache;
use crate::{
    Backend, DatabaseError, DatabaseEventResult, DatabaseEventStatus, DatabaseHelper, Events,
    NostrDatabase, NostrEventsDatabase,
};

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
#[derive(Debug, Clone)]
pub struct MemoryDatabase {
    opts: MemoryDatabaseOptions,
    seen_event_ids: Arc<Mutex<LruCache<EventId, HashSet<RelayUrl>>>>,
    helper: DatabaseHelper,
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
            helper: match opts.max_events {
                Some(max) => DatabaseHelper::bounded(max),
                None => DatabaseHelper::unbounded(),
            },
        }
    }

    fn _event_id_seen(
        &self,
        seen_event_ids: &mut LruCache<EventId, HashSet<RelayUrl>>,
        event_id: EventId,
        relay_url: Option<RelayUrl>,
    ) {
        match seen_event_ids.get_mut(&event_id) {
            Some(set) => {
                if let Some(url) = relay_url {
                    set.insert(url);
                }
            }
            None => {
                let mut set: HashSet<RelayUrl> = HashSet::new();

                if let Some(url) = relay_url {
                    set.insert(url);
                }

                seen_event_ids.put(event_id, set);
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrDatabase for MemoryDatabase {
    fn backend(&self) -> Backend {
        Backend::Memory
    }

    async fn wipe(&self) -> Result<(), DatabaseError> {
        // Clear helper
        self.helper.clear().await;

        // Clear
        let mut seen_event_ids = self.seen_event_ids.lock().await;
        seen_event_ids.clear();
        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrEventsDatabase for MemoryDatabase {
    async fn save_event(&self, event: &Event) -> Result<bool, DatabaseError> {
        if self.opts.events {
            let DatabaseEventResult { to_store, .. } = self.helper.index_event(event).await;
            Ok(to_store)
        } else {
            // Mark it as seen
            let mut seen_event_ids = self.seen_event_ids.lock().await;
            self._event_id_seen(&mut seen_event_ids, event.id, None);

            Ok(false)
        }
    }

    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
        if self.opts.events {
            if self.helper.has_event_id_been_deleted(event_id).await {
                Ok(DatabaseEventStatus::Deleted)
            } else if self.helper.has_event(event_id).await {
                Ok(DatabaseEventStatus::Saved)
            } else {
                Ok(DatabaseEventStatus::NotExistent)
            }
        } else {
            let seen_event_ids = self.seen_event_ids.lock().await;
            Ok(if seen_event_ids.contains(event_id) {
                DatabaseEventStatus::Saved
            } else {
                DatabaseEventStatus::NotExistent
            })
        }
    }

    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: &Timestamp,
    ) -> Result<bool, DatabaseError> {
        Ok(self
            .helper
            .has_coordinate_been_deleted(coordinate, timestamp)
            .await)
    }

    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: RelayUrl,
    ) -> Result<(), DatabaseError> {
        let mut seen_event_ids = self.seen_event_ids.lock().await;
        self._event_id_seen(&mut seen_event_ids, event_id, Some(relay_url));
        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError> {
        let mut seen_event_ids = self.seen_event_ids.lock().await;
        Ok(seen_event_ids.get(event_id).cloned())
    }

    async fn event_by_id(&self, id: &EventId) -> Result<Option<Event>, DatabaseError> {
        Ok(self.helper.event_by_id(id).await)
    }

    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        Ok(self.helper.count(filters).await)
    }

    async fn query(&self, filters: Vec<Filter>) -> Result<Events, DatabaseError> {
        Ok(self.helper.query(filters).await)
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        Ok(self.helper.negentropy_items(filter).await)
    }

    async fn delete(&self, filter: Filter) -> Result<(), DatabaseError> {
        self.helper.delete(filter).await;
        Ok(())
    }
}
