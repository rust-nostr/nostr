// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! SQLite Storage backend for Nostr SDK

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

use std::collections::HashSet;
use std::path::Path;

use async_trait::async_trait;
use nostr_database::prelude::*;

mod store;

use self::store::Store;

/// SQLite Nostr Database
#[derive(Debug, Clone)]
pub struct SQLiteDatabase {
    db: Store,
    // TODO: Temporary use memory database to store seen event IDs
    // until decide if continue to store them in `NostrDatabase`
    // or somewhere else
    temp: MemoryDatabase,
}

impl SQLiteDatabase {
    /// Open or create a new database
    pub async fn open<P>(path: P) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            db: Store::open(path).await.map_err(DatabaseError::backend)?,
            temp: MemoryDatabase::with_opts(MemoryDatabaseOptions {
                events: false,
                max_events: Some(100_000),
            }),
        })
    }
}

#[async_trait]
impl NostrDatabase for SQLiteDatabase {
    fn backend(&self) -> Backend {
        Backend::SQLite
    }

    async fn wipe(&self) -> Result<(), DatabaseError> {
        self.db.wipe().await.map_err(DatabaseError::backend)
    }
}

#[async_trait]
impl NostrEventsDatabase for SQLiteDatabase {
    #[inline]
    async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, DatabaseError> {
        todo!()
    }

    #[inline]
    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
        todo!()
    }

    #[inline]
    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: &Timestamp,
    ) -> Result<bool, DatabaseError> {
        todo!()
    }

    #[inline]
    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: RelayUrl,
    ) -> Result<(), DatabaseError> {
        self.temp.event_id_seen(event_id, relay_url).await
    }

    #[inline]
    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError> {
        self.temp.event_seen_on_relays(event_id).await
    }

    #[inline]
    async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Event>, DatabaseError> {
        self.db.event_by_id(event_id).await.map_err(DatabaseError::backend)
    }

    #[inline]
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        todo!()
    }

    #[inline]
    async fn query(&self, filters: Vec<Filter>) -> Result<Events, DatabaseError> {
        todo!()
    }

    #[inline]
    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        todo!()
    }

    async fn delete(&self, filter: Filter) -> Result<(), DatabaseError> {
        todo!()
    }
}
