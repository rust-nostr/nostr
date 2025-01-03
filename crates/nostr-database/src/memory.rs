// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Memory (RAM) Storage backend for Nostr apps

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use nostr::prelude::*;
use tokio::sync::RwLock;

use crate::events::NostrEventsDatabaseTransaction;
use crate::{
    Backend, DatabaseError, DatabaseEventResult, DatabaseEventStatus, DatabaseHelper, Events,
    NostrDatabase, NostrEventsDatabase, RejectedReason, SaveEventStatus,
};

/// Database options
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryDatabaseOptions {
    /// Store events (default: false)
    pub events: bool,
    /// Max events and IDs to store in memory (default: 35_000)
    ///
    /// `None` means no limits.
    pub max_events: Option<usize>,
}

impl Default for MemoryDatabaseOptions {
    fn default() -> Self {
        Self {
            events: false,
            max_events: Some(35_000),
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
    seen_event_ids: Arc<RwLock<SeenTracker>>,
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
            seen_event_ids: Arc::new(RwLock::new(SeenTracker::new(opts.max_events))),
            helper: match opts.max_events {
                Some(max) => DatabaseHelper::bounded(max),
                None => DatabaseHelper::unbounded(),
            },
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
        let mut seen_event_ids = self.seen_event_ids.write().await;
        seen_event_ids.clear();
        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrEventsDatabase for MemoryDatabase {
    async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, DatabaseError> {
        if self.opts.events {
            let DatabaseEventResult { status, .. } = self.helper.index_event(event).await;
            Ok(status)
        } else {
            // Mark it as seen
            let mut seen_event_ids = self.seen_event_ids.write().await;
            seen_event_ids.seen(event.id, None);

            Ok(SaveEventStatus::Rejected(RejectedReason::Other))
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
            let seen_event_ids = self.seen_event_ids.read().await;
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
        let mut seen_event_ids = self.seen_event_ids.write().await;
        seen_event_ids.seen(event_id, Some(relay_url));
        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError> {
        let seen_event_ids = self.seen_event_ids.read().await;
        Ok(seen_event_ids.get(event_id).cloned())
    }

    async fn event_by_id(&self, id: &EventId) -> Result<Option<Event>, DatabaseError> {
        Ok(self.helper.event_by_id(id).await)
    }

    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        Ok(self.helper.count(filters).await)
    }

    async fn begin_txn(&self) -> Result<Box<dyn NostrEventsDatabaseTransaction>, DatabaseError> {
        todo!()
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

#[derive(Debug)]
struct SeenTracker {
    ids: HashMap<EventId, HashSet<RelayUrl>>,
    capacity: Option<usize>,
    queue: VecDeque<EventId>,
}

impl SeenTracker {
    fn new(capacity: Option<usize>) -> Self {
        Self {
            ids: HashMap::new(),
            capacity,
            queue: VecDeque::new(),
        }
    }

    fn check_capacity(&mut self) {
        // Remove last item if queue > capacity
        if let Some(capacity) = self.capacity {
            if self.queue.len() >= capacity {
                if let Some(last) = self.queue.pop_back() {
                    self.ids.remove(&last);
                }
            }
        }
    }

    fn seen(&mut self, event_id: EventId, relay_url: Option<RelayUrl>) {
        match self.ids.get_mut(&event_id) {
            Some(set) => {
                if let Some(url) = relay_url {
                    set.insert(url);
                }
            }
            None => {
                self.check_capacity();

                let set: HashSet<RelayUrl> = match relay_url {
                    Some(url) => {
                        let mut set: HashSet<RelayUrl> = HashSet::with_capacity(1);
                        set.insert(url);
                        set
                    }
                    None => HashSet::new(),
                };
                self.ids.insert(event_id, set);
                self.queue.push_front(event_id);
            }
        }
    }

    #[inline]
    fn get(&self, id: &EventId) -> Option<&HashSet<RelayUrl>> {
        self.ids.get(id)
    }

    #[inline]
    fn contains(&self, id: &EventId) -> bool {
        self.ids.contains_key(id)
    }

    fn clear(&mut self) {
        self.ids.clear();
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seen_tracker_without_capacity() {
        let mut tracker = SeenTracker::new(None);

        let id0 = EventId::all_zeros();
        tracker.seen(id0, None);

        let id1 = EventId::from_byte_array([1u8; 32]);
        tracker.seen(id1, None);

        let id2 = EventId::from_byte_array([2u8; 32]);
        tracker.seen(id2, None);

        assert_eq!(tracker.ids.len(), 3);
        assert_eq!(tracker.queue.len(), 3);
        assert!(tracker.capacity.is_none());

        assert!(tracker.contains(&id0));
        assert!(tracker.queue.contains(&id0));

        assert!(tracker.contains(&id1));
        assert!(tracker.queue.contains(&id1));

        assert!(tracker.contains(&id2));
        assert!(tracker.queue.contains(&id2));
    }

    #[test]
    fn test_seen_tracker_with_capacity() {
        let mut tracker = SeenTracker::new(Some(2));

        let id0 = EventId::all_zeros();
        tracker.seen(id0, None);

        let id1 = EventId::from_byte_array([1u8; 32]);
        tracker.seen(id1, None);

        let id2 = EventId::from_byte_array([2u8; 32]);
        tracker.seen(id2, None);

        assert_eq!(tracker.ids.len(), 2);
        assert_eq!(tracker.queue.len(), 2);
        assert!(tracker.capacity.is_some());

        assert!(!tracker.contains(&id0));
        assert!(!tracker.queue.contains(&id0));

        assert!(tracker.contains(&id1));
        assert!(tracker.queue.contains(&id1));

        assert!(tracker.contains(&id2));
        assert!(tracker.queue.contains(&id2));
    }
}
