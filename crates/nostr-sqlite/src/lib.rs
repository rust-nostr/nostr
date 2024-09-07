// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! SQLite Storage backend for Nostr SDK

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use std::sync::Arc;

pub extern crate nostr;
pub extern crate nostr_database as database;

use async_trait::async_trait;
use nostr_database::prelude::*;
use rusqlite::config::DbConfig;
use rusqlite::Connection;
use tokio::sync::RwLock;

mod error;
mod migration;
mod pool;

use self::error::Error;
use self::migration::STARTUP_SQL;
use self::pool::Pool;

/// SQLite Nostr Database
#[deprecated(since = "0.35.0", note = "Use LMDB or other backend instead")]
#[derive(Debug, Clone)]
pub struct SQLiteDatabase {
    pool: Pool,
    helper: DatabaseHelper,
    fbb: Arc<RwLock<FlatBufferBuilder<'static>>>,
}

#[allow(deprecated)]
impl SQLiteDatabase {
    async fn new<P>(path: P, helper: DatabaseHelper) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        let conn = Connection::open(path).map_err(DatabaseError::backend)?;
        let pool: Pool = Pool::new(conn);

        // Execute migrations
        migration::run(&pool).await?;

        let this = Self {
            pool,
            helper,
            fbb: Arc::new(RwLock::new(FlatBufferBuilder::with_capacity(70_000))),
        };

        this.bulk_load().await?;

        Ok(this)
    }

    /// Open database with **unlimited** capacity
    #[inline]
    pub async fn open<P>(path: P) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        Self::new(path, DatabaseHelper::unbounded()).await
    }

    /// Open database with **limited** capacity
    #[inline]
    pub async fn open_bounded<P>(path: P, max_capacity: usize) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        Self::new(path, DatabaseHelper::bounded(max_capacity)).await
    }

    #[tracing::instrument(skip_all)]
    async fn bulk_load(&self) -> Result<(), DatabaseError> {
        let events = self
            .pool
            .interact(move |conn| {
                // Query
                let mut stmt = conn.prepare("SELECT event FROM events;")?;
                let mut rows = stmt.query([])?;

                // Decode
                let mut events = BTreeSet::new();
                while let Ok(Some(row)) = rows.next() {
                    let buf: &[u8] = row.get_ref(0)?.as_bytes()?;
                    let event = Event::decode(buf)?;
                    events.insert(event);
                }
                Ok::<BTreeSet<Event>, Error>(events)
            })
            .await??;

        // Build indexes
        let to_discard: HashSet<EventId> = self.helper.bulk_load(events).await;

        // Discard events
        if !to_discard.is_empty() {
            self.pool
                .interact(move |conn| {
                    let mut stmt = conn.prepare_cached("DELETE FROM events WHERE event_id = ?;")?;
                    for id in to_discard.into_iter() {
                        stmt.execute([id.to_hex()])?;
                    }
                    Ok::<(), Error>(())
                })
                .await??;
        }
        Ok(())
    }
}

#[async_trait]
#[allow(deprecated)]
impl NostrDatabase for SQLiteDatabase {
    fn backend(&self) -> Backend {
        Backend::SQLite
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn save_event(&self, event: &Event) -> Result<bool, DatabaseError> {
        // Index event
        let DatabaseEventResult {
            to_store,
            to_discard,
        } = self.helper.index_event(event).await;

        if !to_discard.is_empty() {
            self.pool
                .interact(move |conn| {
                    let mut stmt = conn.prepare_cached("DELETE FROM events WHERE event_id = ?;")?;
                    for id in to_discard.into_iter() {
                        stmt.execute([id.to_hex()])?;
                    }
                    Ok::<(), Error>(())
                })
                .await??;
        }

        if to_store {
            // Acquire FlatBuffers Builder
            let mut fbb = self.fbb.write().await;

            // Encode
            let event_id: EventId = event.id;
            let value: Vec<u8> = event.encode(&mut fbb).to_vec();

            // Save event
            self.pool
                .interact(move |conn| {
                    let mut stmt = conn.prepare_cached(
                        "INSERT OR IGNORE INTO events (event_id, event) VALUES (?, ?);",
                    )?;
                    stmt.execute((event_id.to_hex(), value))
                })
                .await?
                .map_err(DatabaseError::backend)?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
        if self.helper.has_event_id_been_deleted(event_id).await {
            Ok(DatabaseEventStatus::Deleted)
        } else {
            let event_id: String = event_id.to_hex();
            self.pool
                .interact(move |conn| {
                    let mut stmt = conn
                        .prepare_cached(
                            "SELECT EXISTS(SELECT 1 FROM events WHERE event_id = ? LIMIT 1);",
                        )
                        .map_err(DatabaseError::backend)?;
                    let mut rows = stmt.query([event_id]).map_err(DatabaseError::backend)?;
                    let exists: u8 = match rows.next().map_err(DatabaseError::backend)? {
                        Some(row) => row.get(0).map_err(DatabaseError::backend)?,
                        None => 0,
                    };
                    Ok(if exists == 1 {
                        DatabaseEventStatus::Saved
                    } else {
                        DatabaseEventStatus::NotExistent
                    })
                })
                .await?
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
        relay_url: Url,
    ) -> std::result::Result<(), DatabaseError> {
        self.pool
            .interact(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "INSERT OR IGNORE INTO event_seen_by_relays (event_id, relay_url) VALUES (?, ?);",
                )?;
                stmt.execute((event_id.to_hex(), relay_url.to_string()))
            })
            .await?.map_err(DatabaseError::backend)?;
        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<Url>>, DatabaseError> {
        let event_id: String = event_id.to_hex();
        self.pool
            .interact(move |conn| {
                let mut stmt = conn
                    .prepare_cached(
                        "SELECT relay_url FROM event_seen_by_relays WHERE event_id = ?;",
                    )
                    .map_err(DatabaseError::backend)?;
                let mut rows = stmt.query([event_id]).map_err(DatabaseError::backend)?;
                let mut relays = HashSet::new();
                while let Ok(Some(row)) = rows.next() {
                    let url: &str = row
                        .get_ref(0)
                        .map_err(DatabaseError::backend)?
                        .as_str()
                        .map_err(DatabaseError::backend)?;
                    relays.insert(Url::parse(url).map_err(DatabaseError::backend)?);
                }
                Ok(Some(relays))
            })
            .await?
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Event>, DatabaseError> {
        let event_id: String = event_id.to_hex();
        self.pool
            .interact(move |conn| {
                let mut stmt = conn
                    .prepare_cached("SELECT event FROM events WHERE event_id = ?;")
                    .map_err(DatabaseError::backend)?;
                let mut rows = stmt.query([event_id]).map_err(DatabaseError::backend)?;
                match rows.next().map_err(DatabaseError::backend)? {
                    Some(row) => {
                        let buf: &[u8] = row
                            .get_ref(0)
                            .map_err(DatabaseError::backend)?
                            .as_bytes()
                            .map_err(DatabaseError::backend)?;
                        Ok(Some(Event::decode(buf).map_err(DatabaseError::backend)?))
                    }
                    None => Ok(None),
                }
            })
            .await?
    }

    #[inline]
    #[tracing::instrument(skip_all, level = "trace")]
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        Ok(self.helper.count(filters).await)
    }

    #[inline]
    #[tracing::instrument(skip_all)]
    async fn query(&self, filters: Vec<Filter>, order: Order) -> Result<Vec<Event>, DatabaseError> {
        Ok(self.helper.query(filters, order).await)
    }

    #[inline]
    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        Ok(self.helper.negentropy_items(filter).await)
    }

    async fn delete(&self, filter: Filter) -> Result<(), DatabaseError> {
        match self.helper.delete(filter).await {
            Some(ids) => {
                self.pool
                    .interact(move |conn| {
                        let mut stmt =
                            conn.prepare_cached("DELETE FROM events WHERE event_id = ?;")?;
                        for id in ids.into_iter() {
                            stmt.execute([id.to_hex()])?;
                        }
                        Ok::<(), Error>(())
                    })
                    .await??;
            }
            None => {
                self.pool
                    .interact(move |conn| conn.execute("DELETE FROM events;", []))
                    .await?
                    .map_err(DatabaseError::backend)?;
            }
        };

        Ok(())
    }

    async fn wipe(&self) -> Result<(), DatabaseError> {
        self.pool
            .interact(|conn| {
                // Reset DB
                conn.set_db_config(DbConfig::SQLITE_DBCONFIG_RESET_DATABASE, true)?;
                conn.execute("VACUUM;", [])?;
                conn.set_db_config(DbConfig::SQLITE_DBCONFIG_RESET_DATABASE, false)?;

                // Execute migrations
                conn.execute_batch(STARTUP_SQL)?;

                Ok::<(), Error>(())
            })
            .await??;

        migration::run(&self.pool).await?;

        self.helper.clear().await;

        Ok(())
    }
}
