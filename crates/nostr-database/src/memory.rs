// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Memory (RAM) Storage backend for Nostr apps

use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use nostr::prelude::*;
use tokio::sync::RwLock;

use crate::{
    Backend, DatabaseError, DatabaseEventResult, DatabaseEventStatus, DatabaseHelper, Events,
    NostrDatabase, SaveEventStatus,
};

const MAX_EVENTS: usize = 35_000;

/// Database options
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryDatabaseOptions {
    /// Store events (default: false)
    pub events: bool,
    /// Max events and IDs to store in memory (default: 35_000)
    ///
    /// `None` means no limits.
    ///
    /// If `Some(0)` is passed, the default value will be used.
    pub max_events: Option<usize>,
}

impl Default for MemoryDatabaseOptions {
    fn default() -> Self {
        Self {
            events: false,
            max_events: Some(MAX_EVENTS),
        }
    }
}

impl MemoryDatabaseOptions {
    /// New default database options
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
enum InnerMemoryDatabase {
    /// Just an event ID tracker
    Tracker(Arc<RwLock<LruCache<EventId, ()>>>),
    /// A full in-memory events store
    Full(DatabaseHelper),
}

/// Memory Database (RAM)
#[derive(Debug, Clone)]
pub struct MemoryDatabase {
    inner: InnerMemoryDatabase,
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
    pub fn with_opts(mut opts: MemoryDatabaseOptions) -> Self {
        // Check if `Some(0)`
        if let Some(0) = opts.max_events {
            opts.max_events = Some(MAX_EVENTS);
        }

        // Check if event storing is allowed
        let inner: InnerMemoryDatabase = if opts.events {
            let helper: DatabaseHelper = match opts.max_events {
                Some(max) => DatabaseHelper::bounded(max),
                None => DatabaseHelper::unbounded(),
            };
            InnerMemoryDatabase::Full(helper)
        } else {
            let cache: LruCache<EventId, ()> = match opts.max_events {
                Some(max) if max > 0 => {
                    // SAFETY: checked above if > 0
                    let max: NonZeroUsize = NonZeroUsize::new(max).unwrap();
                    LruCache::new(max)
                }
                _ => LruCache::unbounded(),
            };
            InnerMemoryDatabase::Tracker(Arc::new(RwLock::new(cache)))
        };

        Self { inner }
    }
}

impl NostrDatabase for MemoryDatabase {
    fn backend(&self) -> Backend {
        Backend::Memory
    }

    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(tracker) => {
                    // Mark it as seen
                    let mut seen_event_ids = tracker.write().await;
                    seen_event_ids.put(event.id, ());

                    Ok(SaveEventStatus::Success)
                }
                InnerMemoryDatabase::Full(helper) => {
                    let DatabaseEventResult { status, .. } = helper.index_event(event).await;
                    Ok(status)
                }
            }
        })
    }

    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(tracker) => {
                    let seen_event_ids = tracker.read().await;

                    Ok(if seen_event_ids.contains(event_id) {
                        DatabaseEventStatus::Saved
                    } else {
                        DatabaseEventStatus::NotExistent
                    })
                }
                InnerMemoryDatabase::Full(helper) => {
                    if helper.has_event_id_been_deleted(event_id).await {
                        Ok(DatabaseEventStatus::Deleted)
                    } else if helper.has_event(event_id).await {
                        Ok(DatabaseEventStatus::Saved)
                    } else {
                        Ok(DatabaseEventStatus::NotExistent)
                    }
                }
            }
        })
    }

    fn has_coordinate_been_deleted<'a>(
        &'a self,
        coordinate: &'a CoordinateBorrow<'a>,
        timestamp: &'a Timestamp,
    ) -> BoxedFuture<'a, Result<bool, DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(..) => Ok(false),
                InnerMemoryDatabase::Full(helper) => Ok(helper
                    .has_coordinate_been_deleted(coordinate, timestamp)
                    .await),
            }
        })
    }

    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(..) => Ok(None),
                InnerMemoryDatabase::Full(helper) => Ok(helper.event_by_id(event_id).await),
            }
        })
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(..) => Ok(0),
                InnerMemoryDatabase::Full(helper) => Ok(helper.count(filter).await),
            }
        })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(..) => Ok(Events::new(&filter)),
                InnerMemoryDatabase::Full(helper) => Ok(helper.query(filter).await),
            }
        })
    }

    fn negentropy_items(
        &self,
        filter: Filter,
    ) -> BoxedFuture<Result<Vec<(EventId, Timestamp)>, DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(..) => Ok(Vec::new()),
                InnerMemoryDatabase::Full(helper) => Ok(helper.negentropy_items(filter).await),
            }
        })
    }

    fn delete(&self, filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(..) => Ok(()),
                InnerMemoryDatabase::Full(helper) => {
                    helper.delete(filter).await;
                    Ok(())
                }
            }
        })
    }

    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move {
            match &self.inner {
                InnerMemoryDatabase::Tracker(tracker) => {
                    let mut seen_event_ids = tracker.write().await;
                    seen_event_ids.clear();
                }
                InnerMemoryDatabase::Full(helper) => {
                    helper.clear().await;
                }
            }

            Ok(())
        })
    }
}
