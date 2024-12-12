// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::path::Path;
use std::sync::Arc;

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

#[derive(Debug, Clone)]
pub struct Store {
    pool: Pool,
    fbb: Arc<RwLock<FlatBufferBuilder<'static>>>,   
}

impl Store {
    pub async fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let conn = Connection::open(path)?;
        let pool: Pool = Pool::new(conn);

        // Execute migrations
        migration::run(&pool).await?;

        Ok(Self {
            pool,
            fbb: Arc::new(RwLock::new(FlatBufferBuilder::with_capacity(70_000))),
        })
    }
    
    pub async fn event_by_id(&self, id: &EventId) -> Result<Option<Event>, Error> {
        let event_id = id.to_bytes();
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
    
    pub async fn wipe(&self) -> Result<(), Error> {
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

        migration::run(&self.pool).await
    }
}

/// Find all events that match the filter
fn single_filter_query<'a>(
    conn: &mut Connection,
    filter: Filter,
) -> Result<Box<dyn Iterator<Item = DatabaseEvent<'a>> + 'a>, Error> {
    
}