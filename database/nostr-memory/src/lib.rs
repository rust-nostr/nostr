//! Memory (RAM) Storage backend for Nostr apps

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`
#![doc = include_str!("../README.md")]

use core::num::NonZeroUsize;

use nostr::prelude::*;
use nostr_database::prelude::*;
use tokio::sync::RwLock;

pub mod builder;
pub mod error;
pub mod prelude;
mod store;

use self::builder::MemoryDatabaseBuilder;
use self::error::Error;
use self::store::{DatabaseEventResult, MemoryStore};

/// Memory Database (RAM)
#[derive(Debug)]
pub struct MemoryDatabase {
    store: RwLock<MemoryStore>,
}

impl MemoryDatabase {
    /// Unbounded database helper
    #[inline]
    pub fn unbounded() -> Result<Self, Error> {
        Self::builder().build()
    }

    /// Bounded database helper
    #[inline]
    pub fn bounded(max: NonZeroUsize) -> Result<Self, Error> {
        Self::builder().max_events(max).build()
    }

    /// Get a new builder.
    #[inline]
    pub fn builder() -> MemoryDatabaseBuilder {
        MemoryDatabaseBuilder::default()
    }

    // TODO: at the moment we are not using the Result, but will be needed in the future and we want to avoid breaking changes.
    fn from_builder(builder: MemoryDatabaseBuilder) -> Result<Self, Error> {
        let store: MemoryStore = match builder.max_events {
            Some(max) => MemoryStore::bounded(max),
            None => MemoryStore::unbounded(),
        };

        Ok(Self {
            store: RwLock::new(store),
        })
    }
}

impl NostrDatabase for MemoryDatabase {
    fn backend(&self) -> Backend {
        Backend::Memory
    }

    fn features(&self) -> Features {
        Features {
            persistent: false,
            event_expiration: false,
            full_text_search: true,
            request_to_vanish: false,
        }
    }

    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>> {
        Box::pin(async move {
            let mut store = self.store.write().await;
            let DatabaseEventResult { status, .. } = store.index_event(event);
            Ok(status)
        })
    }

    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        Box::pin(async move {
            let store = self.store.read().await;

            if store.has_event_id_been_deleted(event_id) {
                Ok(DatabaseEventStatus::Deleted)
            } else if store.has_event(event_id) {
                Ok(DatabaseEventStatus::Saved)
            } else {
                Ok(DatabaseEventStatus::NotExistent)
            }
        })
    }

    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            let store = self.store.read().await;
            Ok(store.event_by_id(event_id).cloned())
        })
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move {
            let store = self.store.read().await;
            Ok(store.count(filter))
        })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        Box::pin(async move {
            let store = self.store.read().await;
            let mut events = Events::new(&filter);
            events.extend(store.query(filter).cloned());
            Ok(events)
        })
    }

    fn negentropy_items(
        &self,
        filter: Filter,
    ) -> BoxedFuture<Result<Vec<(EventId, Timestamp)>, DatabaseError>> {
        Box::pin(async move {
            let store = self.store.read().await;
            Ok(store.negentropy_items(filter))
        })
    }

    fn delete(&self, filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move {
            let mut store = self.store.write().await;
            store.delete(filter);
            Ok(())
        })
    }

    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move {
            let mut store = self.store.write().await;
            store.clear();

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use nostr_database_test_suite::database_unit_tests;

    use super::*;

    struct TestDatabase {
        inner: MemoryDatabase,
    }

    impl Deref for TestDatabase {
        type Target = MemoryDatabase;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl TestDatabase {
        async fn new() -> Self {
            Self {
                inner: MemoryDatabase::unbounded().unwrap(),
            }
        }
    }

    database_unit_tests!(TestDatabase, TestDatabase::new);
}
