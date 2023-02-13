// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Store

use std::net::SocketAddr;
use std::path::Path;

use nostr::{Event, Url};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;

use crate::migration::{self, MigrationError, STARTUP_SQL};

pub(crate) type SqlitePool = r2d2::Pool<SqliteConnectionManager>;
pub(crate) type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Store error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Sqlite error
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// Sqlite Pool error
    #[error(transparent)]
    Pool(#[from] r2d2::Error),
    /// Migration error
    #[error(transparent)]
    Migration(#[from] MigrationError),
}

/// Store
#[derive(Debug, Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Drop for Store {
    fn drop(&mut self) {}
}

impl Store {
    /// Open new database
    pub fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let manager = SqliteConnectionManager::file(path.as_ref())
            .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)
            .with_init(|c| c.execute_batch(STARTUP_SQL));
        let pool = r2d2::Pool::new(manager)?;
        migration::run(&mut pool.get()?)?;
        Ok(Self { pool })
    }

    /// Close SQLite connection
    pub fn close(self) {
        drop(self);
    }

    /// Insert new relay
    pub fn insert_relay(&self, url: Url, proxy: Option<SocketAddr>) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO relays (url, proxy) VALUES (?, ?);",
            (url, proxy.map(|a| a.to_string())),
        )?;
        Ok(())
    }

    /// Get relays
    pub fn get_relays(&self, enabled: bool) -> Result<Vec<(Url, Option<SocketAddr>)>, Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT url, proxy FROM relays WHERE enabled = ?")?;
        let mut rows = stmt.query([enabled])?;

        let mut relays: Vec<(Url, Option<SocketAddr>)> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let url: Url = row.get(0)?;
            let proxy: Option<String> = row.get(1)?;
            relays.push((
                url,
                proxy
                    .map(|p| p.parse())
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap()),
            ));
        }
        Ok(relays)
    }

    /// Delete relay
    pub fn delete_relay(&self, url: Url) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM relays WHERE url = ?;", [url])?;
        Ok(())
    }

    /// Enable relay
    pub fn enable_relay(&self, url: Url) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute("UPDATE relays SET enabled = ? WHERE url = ?;", (1, url))?;
        Ok(())
    }

    /// Disbale relay
    pub fn disable_relay(&self, url: Url) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute("UPDATE relays SET enabled = ? WHERE url = ?;", (0, url))?;
        Ok(())
    }

    /// Insert new event
    pub fn insert_event(&self, event: Event) -> Result<(), Error> {
        let conn = self.pool.get()?;
        // Insert event
        conn.execute(
            "INSERT OR IGNORE INTO events (id, pubkey, created_at, kind, content, sig) VALUES (?, ?, ?, ?, ?, ?);",
            (event.id.to_hex(), &event.pubkey.to_string(), event.created_at.as_u64(), event.kind.as_u64(), event.content, event.sig.to_string()),
        )?;
        // Insert tags
        let mut stmt =
            conn.prepare("INSERT OR IGNORE INTO tags (event_id, kind, value) VALUES (?, ?, ?)")?;
        for tag in event.tags.into_iter() {
            let tag: Vec<String> = tag.as_vec();
            let kind = &tag[0];
            let value = tag.get(1..);
            stmt.execute((event.id.as_bytes(), kind, serde_json::json!(value)))?;
        }
        Ok(())
    }
}
