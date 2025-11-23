// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! LMDB storage backend for nostr apps
//!
//! Fork of [Pocket](https://github.com/mikedilger/pocket) database.

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(clippy::mutable_key_type)]

use std::path::{Path, PathBuf};

use nostr_database::prelude::*;

pub mod prelude;
mod store;

use self::store::Store;

// 64-bit
#[cfg(target_pointer_width = "64")]
const MAP_SIZE: usize = 1024 * 1024 * 1024 * 32; // 32GB

// 32-bit
#[cfg(target_pointer_width = "32")]
const MAP_SIZE: usize = 0xFFFFF000; // 4GB (2^32-4096)

#[allow(missing_docs)]
#[deprecated(since = "0.45.0", note = "Use NostrLmdb instead")]
pub type NostrLMDB = NostrLmdb;

/// Nostr LMDB database builder
#[derive(Debug, Clone)]
pub struct NostrLmdbBuilder {
    /// Database path
    pub path: PathBuf,
    /// Custom map size
    ///
    /// By default, the following map size is used:
    /// - 32GB for 64-bit arch
    /// - 4GB for 32-bit arch
    pub map_size: Option<usize>,
    /// Maximum number of reader threads
    ///
    /// Defaults to 126 if not set
    pub max_readers: Option<u32>,
    /// Number of additional databases to allocate beyond the 9 internal ones
    ///
    /// Defaults to 0 if not set
    pub additional_dbs: Option<u32>,
}

impl NostrLmdbBuilder {
    /// New LMDb builder
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            map_size: None,
            max_readers: None,
            additional_dbs: None,
        }
    }

    /// Map size
    ///
    /// By default, the following map size is used:
    /// - 32GB for 64-bit arch
    /// - 4GB for 32-bit arch
    pub fn map_size(mut self, map_size: usize) -> Self {
        self.map_size = Some(map_size);
        self
    }

    /// Maximum number of reader threads
    ///
    /// Defaults to 126 if not set
    pub fn max_readers(mut self, max_readers: u32) -> Self {
        self.max_readers = Some(max_readers);
        self
    }

    /// Number of additional databases to allocate beyond the 9 internal ones
    ///
    /// Defaults to 0 if not set
    pub fn additional_dbs(mut self, additional_dbs: u32) -> Self {
        self.additional_dbs = Some(additional_dbs);
        self
    }

    /// Build
    pub async fn build(self) -> Result<NostrLmdb, DatabaseError> {
        let map_size: usize = self.map_size.unwrap_or(MAP_SIZE);
        let max_readers: u32 = self.max_readers.unwrap_or(126);
        let additional_dbs: u32 = self.additional_dbs.unwrap_or(0);
        let db: Store = Store::open(self.path, map_size, max_readers, additional_dbs)
            .await
            .map_err(DatabaseError::backend)?;
        Ok(NostrLmdb { db })
    }
}

/// LMDB Nostr Database
#[derive(Debug)]
pub struct NostrLmdb {
    db: Store,
}

impl NostrLmdb {
    /// Open LMDB database
    #[inline]
    pub async fn open<P>(path: P) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        Self::builder(path).build().await
    }

    /// Get a new builder
    #[inline]
    pub fn builder<P>(path: P) -> NostrLmdbBuilder
    where
        P: AsRef<Path>,
    {
        NostrLmdbBuilder::new(path)
    }
}

impl NostrDatabase for NostrLmdb {
    #[inline]
    fn backend(&self) -> Backend {
        Backend::LMDB
    }

    fn features(&self) -> Features {
        Features {
            persistent: true,
            event_expiration: false,
            full_text_search: true,
        }
    }

    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>> {
        Box::pin(async move {
            self.db
                .save_event(event)
                .await
                .map_err(DatabaseError::backend)
        })
    }

    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        Box::pin(async move {
            self.db
                .check_id(*event_id)
                .await
                .map_err(DatabaseError::backend)
        })
    }

    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            self.db
                .get_event_by_id(*event_id)
                .await
                .map_err(DatabaseError::backend)
        })
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move { self.db.count(filter).await.map_err(DatabaseError::backend) })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        Box::pin(async move { self.db.query(filter).await.map_err(DatabaseError::backend) })
    }

    fn negentropy_items(
        &self,
        filter: Filter,
    ) -> BoxedFuture<Result<Vec<(EventId, Timestamp)>, DatabaseError>> {
        Box::pin(async move {
            self.db
                .negentropy_items(filter)
                .await
                .map_err(DatabaseError::backend)
        })
    }

    fn delete(&self, filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { self.db.delete(filter).await.map_err(DatabaseError::backend) })
    }

    #[inline]
    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { self.db.wipe().await.map_err(DatabaseError::backend) })
    }
}

#[cfg(test)]
mod tests {
    use nostr_database_test_suite::database_unit_tests;
    use tempfile::TempDir;

    use super::*;

    struct TempDatabase {
        db: NostrLmdb,
        // Needed to avoid the drop and deletion of temp folder
        _temp: TempDir,
    }

    impl Deref for TempDatabase {
        type Target = NostrLmdb;

        fn deref(&self) -> &Self::Target {
            &self.db
        }
    }

    impl TempDatabase {
        async fn new() -> Self {
            let path = tempfile::tempdir().unwrap();
            Self {
                db: NostrLmdb::open(&path).await.unwrap(),
                _temp: path,
            }
        }
    }

    database_unit_tests!(TempDatabase, TempDatabase::new);
}
