// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! SQLite Storage backend for Nostr SDK

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use std::sync::Arc;

pub extern crate nostr;
pub extern crate nostr_database as database;

use async_trait::async_trait;
use deadpool_sqlite::{Config, Object, Pool, Runtime};
use nostr::nips::nip01::Coordinate;
use nostr::{Event, EventId, Filter, Timestamp, Url};
use nostr_database::{
    Backend, DatabaseIndexes, EventIndexResult, FlatBufferBuilder, FlatBufferDecode,
    FlatBufferEncode, NostrDatabase, Order,
};
use rusqlite::config::DbConfig;
use tokio::sync::RwLock;

mod error;
mod migration;

pub use self::error::Error;
use self::migration::STARTUP_SQL;

const BATCH_SIZE: usize = 100;

/// SQLite Nostr Database
#[derive(Debug, Clone)]
pub struct SQLiteDatabase {
    db: Pool,
    indexes: DatabaseIndexes,
    fbb: Arc<RwLock<FlatBufferBuilder<'static>>>,
}

impl SQLiteDatabase {
    /// Open SQLite store
    pub async fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let cfg = Config::new(path.as_ref());
        let pool = cfg.create_pool(Runtime::Tokio1)?;

        // Acquire connection
        let conn = pool.get().await?;

        // Execute migrations
        migration::run(&conn).await?;

        let this = Self {
            db: pool,
            indexes: DatabaseIndexes::new(),
            fbb: Arc::new(RwLock::new(FlatBufferBuilder::with_capacity(70_000))),
        };

        // Build indexes
        this.build_indexes(&conn).await?;

        Ok(this)
    }

    async fn acquire(&self) -> Result<Object, Error> {
        Ok(self.db.get().await?)
    }

    #[tracing::instrument(skip_all)]
    async fn build_indexes(&self, conn: &Object) -> Result<(), Error> {
        let events = conn
            .interact(move |conn| {
                let mut stmt = conn.prepare_cached("SELECT event FROM events;")?;
                let mut rows = stmt.query([])?;
                let mut events = BTreeSet::new();
                while let Ok(Some(row)) = rows.next() {
                    let buf: Vec<u8> = row.get(0)?;
                    events.insert(Event::decode(&buf)?);
                }
                Ok::<BTreeSet<Event>, Error>(events)
            })
            .await??;

        // Build indexes
        let to_discard: Vec<EventId> = self.indexes.bulk_index(events).await.into_iter().collect();

        // Discard events
        if !to_discard.is_empty() {
            let conn = self.acquire().await?;
            conn.interact(move |conn| {
                for chunk in to_discard.chunks(BATCH_SIZE) {
                    let delete_query = format!(
                        "DELETE FROM events WHERE {};",
                        chunk
                            .iter()
                            .map(|id| format!("event_id = '{id}'"))
                            .collect::<Vec<_>>()
                            .join(" OR ")
                    );
                    conn.execute(&delete_query, [])?;
                }

                Ok::<(), Error>(())
            })
            .await??;
        }
        Ok(())
    }
}

#[async_trait]
impl NostrDatabase for SQLiteDatabase {
    type Err = Error;

    fn backend(&self) -> Backend {
        Backend::SQLite
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        // Index event
        let EventIndexResult {
            to_store,
            to_discard,
        } = self.indexes.index_event(event).await;

        if !to_discard.is_empty() {
            let conn = self.acquire().await?;
            let to_discard: Vec<EventId> = to_discard.into_iter().collect();
            conn.interact(move |conn| {
                for chunk in to_discard.chunks(BATCH_SIZE) {
                    let delete_query = format!(
                        "DELETE FROM events WHERE {};",
                        chunk
                            .iter()
                            .map(|id| format!("event_id = '{id}'"))
                            .collect::<Vec<_>>()
                            .join(" OR ")
                    );
                    conn.execute(&delete_query, [])?;
                }

                Ok::<(), Error>(())
            })
            .await??;
        }

        if to_store {
            // Acquire FlatBuffers Builder
            let mut fbb = self.fbb.write().await;

            // Encode
            let event_id: EventId = event.id();
            let value: Vec<u8> = event.encode(&mut fbb).to_vec();

            // Save event
            let conn = self.acquire().await?;
            conn.interact(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "INSERT OR IGNORE INTO events (event_id, event) VALUES (?, ?);",
                )?;
                stmt.execute((event_id.to_hex(), value))
            })
            .await??;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn bulk_import(&self, events: BTreeSet<Event>) -> Result<(), Self::Err> {
        // Acquire database conn lock
        let conn = self.acquire().await?;

        // Acquire FlatBuffers Builder
        let mut fbb = self.fbb.write().await;

        // Events to store
        let events = self.indexes.bulk_import(events).await;

        // Encode
        let events: Vec<(EventId, Vec<u8>)> = events
            .into_iter()
            .map(move |e| {
                let event_id: EventId = e.id();
                let value: Vec<u8> = e.encode(&mut fbb).to_vec();
                (event_id, value)
            })
            .collect();

        // Bulk save
        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            for (event_id, value) in events.into_iter() {
                tx.execute(
                    "INSERT OR IGNORE INTO events (event_id, event) VALUES (?, ?);",
                    (event_id.to_hex(), value),
                )?;
            }

            tx.commit()
        })
        .await??;

        Ok(())
    }

    async fn has_event_already_been_saved(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        if self.indexes.has_event_id_been_deleted(event_id).await {
            Ok(true)
        } else {
            let conn = self.acquire().await?;
            let event_id: String = event_id.to_hex();
            conn.interact(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT EXISTS(SELECT 1 FROM events WHERE event_id = ? LIMIT 1);",
                )?;
                let mut rows = stmt.query([event_id])?;
                let exists: u8 = match rows.next()? {
                    Some(row) => row.get(0)?,
                    None => 0,
                };
                Ok(exists == 1)
            })
            .await?
        }
    }

    async fn has_event_already_been_seen(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        let conn = self.acquire().await?;
        let event_id: String = event_id.to_hex();
        conn.interact(move |conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT EXISTS(SELECT 1 FROM event_seen_by_relays WHERE event_id = ? LIMIT 1);",
            )?;
            let mut rows = stmt.query([event_id])?;
            let exists: u8 = match rows.next()? {
                Some(row) => row.get(0)?,
                None => 0,
            };
            Ok(exists == 1)
        })
        .await?
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
        let conn = self.acquire().await?;
        conn.interact(move |conn| {
            let mut stmt = conn.prepare_cached(
                "INSERT OR IGNORE INTO event_seen_by_relays (event_id, relay_url) VALUES (?, ?);",
            )?;
            stmt.execute((event_id.to_hex(), relay_url.to_string()))
        })
        .await??;
        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err> {
        let conn = self.acquire().await?;
        conn.interact(move |conn| {
            let mut stmt = conn
                .prepare_cached("SELECT relay_url FROM event_seen_by_relays WHERE event_id = ?;")?;
            let mut rows = stmt.query([event_id.to_hex()])?;
            let mut relays = HashSet::new();
            while let Ok(Some(row)) = rows.next() {
                let url: String = row.get(0)?;
                relays.insert(Url::parse(&url)?);
            }
            Ok(Some(relays))
        })
        .await?
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err> {
        let conn = self.acquire().await?;
        conn.interact(move |conn| {
            let mut stmt = conn.prepare_cached("SELECT event FROM events WHERE event_id = ?;")?;
            let mut rows = stmt.query([event_id.to_hex()])?;
            let row = rows
                .next()?
                .ok_or_else(|| Error::NotFound("event".into()))?;
            let buf: Vec<u8> = row.get(0)?;
            Ok(Event::decode(&buf)?)
        })
        .await?
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, Self::Err> {
        Ok(self.indexes.count(filters).await)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn query(&self, filters: Vec<Filter>, order: Order) -> Result<Vec<Event>, Self::Err> {
        Ok(self.indexes.query(filters, order).await)
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
        Ok(self.indexes.negentropy_items(filter).await)
    }

    async fn delete(&self, filter: Filter) -> Result<(), Self::Err> {
        match self.indexes.delete(filter).await {
            Some(ids) => {
                let conn = self.acquire().await?;
                let ids: Vec<EventId> = ids.into_iter().collect();
                conn.interact(move |conn| {
                    for chunk in ids.chunks(BATCH_SIZE) {
                        let delete_query = format!(
                            "DELETE FROM events WHERE {};",
                            chunk
                                .iter()
                                .map(|id| format!("event_id = '{id}'"))
                                .collect::<Vec<_>>()
                                .join(" OR ")
                        );
                        conn.execute(&delete_query, [])?;
                    }

                    Ok::<(), Error>(())
                })
                .await??;
            }
            None => {
                let conn = self.acquire().await?;
                conn.interact(move |conn| conn.execute("DELETE FROM events;", []))
                    .await??;
            }
        };

        Ok(())
    }

    async fn wipe(&self) -> Result<(), Self::Err> {
        let conn = self.acquire().await?;

        conn.interact(|conn| {
            // Reset DB
            conn.set_db_config(DbConfig::SQLITE_DBCONFIG_RESET_DATABASE, true)?;
            conn.execute("VACUUM;", [])?;
            conn.set_db_config(DbConfig::SQLITE_DBCONFIG_RESET_DATABASE, false)?;

            // Execute migrations
            conn.execute_batch(STARTUP_SQL)?;

            Ok::<(), Error>(())
        })
        .await??;

        migration::run(&conn).await?;

        self.indexes.clear().await;

        Ok(())
    }
}
