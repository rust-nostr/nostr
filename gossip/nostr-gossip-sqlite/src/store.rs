use std::collections::HashSet;
use std::path::Path;

use nostr::prelude::BoxedFuture;
use nostr::{Event, PublicKey, RelayUrl};
use nostr_gossip::error::GossipError;
use nostr_gossip::{BestRelaySelection, NostrGossip};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;

/// Nostr Gossip SQLite store.
#[derive(Debug, Clone)]
pub struct NostrGossipSqlite {
    pool: SqlitePool,
    //best_relays_limit: usize,
}

impl NostrGossipSqlite {
    async fn new(opts: SqliteConnectOptions) -> Result<Self, GossipError> {
        // Create a connection pool.
        let pool: SqlitePool = SqlitePool::connect_with(opts)
            .await
            .map_err(GossipError::backend)?;

        // Run migrations
        let migrator: Migrator = sqlx::migrate!();
        migrator.run(&pool).await.map_err(GossipError::backend)?;

        // Construct
        Ok(Self { pool })
    }

    /// Open a persistent database
    pub async fn open<P>(path: P) -> Result<Self, GossipError>
    where
        P: AsRef<Path>,
    {
        // Built options
        let opts: SqliteConnectOptions = SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(path);

        // Create instance
        Self::new(opts).await
    }

    /// Open an in-memory database
    pub async fn in_memory() -> Result<Self, GossipError> {
        // Built options
        let opts: SqliteConnectOptions = SqliteConnectOptions::new().in_memory(true);

        // Create instance
        Self::new(opts).await
    }
}

impl NostrGossip for NostrGossipSqlite {
    fn process<'a>(
        &'a self,
        event: &'a Event,
        relay_url: Option<&'a RelayUrl>,
    ) -> BoxedFuture<'a, Result<(), GossipError>> {
        todo!()
    }

    fn get_best_relays<'a>(
        &'a self,
        public_key: &'a PublicKey,
        selection: BestRelaySelection,
    ) -> BoxedFuture<'a, Result<HashSet<RelayUrl>, GossipError>> {
        todo!()
    }
}
