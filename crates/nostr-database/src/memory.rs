// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Memory (RAM) Storage backend for Nostr apps

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use nostr::{Event, EventId, Filter, FiltersMatchEvent, Timestamp, Url};
use tokio::sync::RwLock;

use crate::{
    Backend, DatabaseError, DatabaseIndexes, DatabaseOptions, EventIndexResult, NostrDatabase,
};

/// Memory Database (RAM)
#[derive(Debug)]
pub struct MemoryDatabase {
    opts: DatabaseOptions,
    seen_event_ids: Arc<RwLock<HashMap<EventId, HashSet<Url>>>>,
    events: Arc<RwLock<HashMap<EventId, Event>>>,
    indexes: DatabaseIndexes,
}

// TODO: add queue field?

impl Default for MemoryDatabase {
    fn default() -> Self {
        Self::new(DatabaseOptions { events: false })
    }
}

impl MemoryDatabase {
    /// New Memory database
    pub fn new(opts: DatabaseOptions) -> Self {
        Self {
            opts,
            seen_event_ids: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
            indexes: DatabaseIndexes::new(),
        }
    }

    fn _event_id_seen(
        &self,
        seen_event_ids: &mut HashMap<EventId, HashSet<Url>>,
        event_id: EventId,
        relay_url: Url,
    ) {
        seen_event_ids
            .entry(event_id)
            .and_modify(|set| {
                set.insert(relay_url.clone());
            })
            .or_insert_with(|| {
                let mut set = HashSet::with_capacity(1);
                set.insert(relay_url);
                set
            });
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrDatabase for MemoryDatabase {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        Backend::Memory
    }

    fn opts(&self) -> DatabaseOptions {
        self.opts
    }

    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        if self.opts.events {
            let EventIndexResult {
                to_store,
                to_discard,
            } = self.indexes.index_event(event).await;

            if to_store {
                let mut events = self.events.write().await;

                events.insert(event.id, event.clone());

                for event_id in to_discard.into_iter() {
                    events.remove(&event_id);
                }

                Ok(true)
            } else {
                tracing::warn!("Event {} not saved: unknown", event.id);
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    async fn has_event_already_been_saved(&self, event_id: EventId) -> Result<bool, Self::Err> {
        if self.indexes.has_been_deleted(&event_id).await {
            Ok(true)
        } else if self.opts.events {
            let events = self.events.read().await;
            Ok(events.contains_key(&event_id))
        } else {
            Ok(false)
        }
    }

    async fn has_event_already_been_seen(&self, event_id: EventId) -> Result<bool, Self::Err> {
        let seen_event_ids = self.seen_event_ids.read().await;
        Ok(seen_event_ids.contains_key(&event_id))
    }

    async fn event_id_seen(&self, event_id: EventId, relay_url: Url) -> Result<(), Self::Err> {
        let mut seen_event_ids = self.seen_event_ids.write().await;
        self._event_id_seen(&mut seen_event_ids, event_id, relay_url);
        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err> {
        let seen_event_ids = self.seen_event_ids.read().await;
        Ok(seen_event_ids.get(&event_id).cloned())
    }

    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err> {
        if self.opts.events {
            let events = self.events.read().await;
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
    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err> {
        if self.opts.events {
            let ids = self.indexes.query(filters.clone()).await;
            let events = self.events.read().await;

            let mut list: Vec<Event> = Vec::new();
            for event_id in ids.into_iter() {
                if let Some(event) = events.get(&event_id) {
                    if filters.match_event(event) {
                        list.push(event.clone());
                    }
                }
            }
            Ok(list)
        } else {
            Err(DatabaseError::FeatureDisabled)
        }
    }

    async fn event_ids_by_filters(&self, filters: Vec<Filter>) -> Result<Vec<EventId>, Self::Err> {
        if self.opts.events {
            Ok(self.indexes.query(filters).await)
        } else {
            Err(DatabaseError::FeatureDisabled)
        }
    }

    async fn negentropy_items(
        &self,
        _filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
        Err(DatabaseError::NotSupported)
    }

    async fn wipe(&self) -> Result<(), Self::Err> {
        let mut seen_event_ids = self.seen_event_ids.write().await;
        seen_event_ids.clear();
        let mut events = self.events.write().await;
        events.clear();
        Ok(())
    }
}
