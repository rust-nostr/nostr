use std::collections::HashSet;
use std::path::Path;

use nostr::prelude::BoxedFuture;
use nostr::{Event, PublicKey, RelayUrl, TagKind, Timestamp};
use nostr_gossip::error::GossipError;
use nostr_gossip::{BestRelaySelection, GossipPublicKeyStatus, NostrGossip};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::error::Error;
use crate::flags::Flags;

/// Nostr Gossip SQLite store.
#[derive(Debug, Clone)]
pub struct NostrGossipSqlite {
    pool: SqlitePool,
    //best_relays_limit: usize,
}

impl NostrGossipSqlite {
    async fn new(opts: SqliteConnectOptions) -> Result<Self, Error> {
        // Create a connection pool.
        let pool: SqlitePool = SqlitePool::connect_with(opts).await?;

        // Run migrations
        let migrator: Migrator = sqlx::migrate!();
        migrator.run(&pool).await?;

        // Construct
        Ok(Self { pool })
    }

    /// Open a persistent database
    pub async fn open<P>(path: P) -> Result<Self, Error>
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

    // TODO: at the moment seems that the migrations don't work with the in-memory mode
    // /// Open an in-memory database
    // pub async fn in_memory() -> Result<Self, Error> {
    //     // Built options
    //     let opts: SqliteConnectOptions = SqliteConnectOptions::new().in_memory(true).shared_cache(true);
    //
    //     // Create instance
    //     Self::new(opts).await
    // }

    async fn process_event(
        &self,
        event: &Event,
        relay_url: Option<&RelayUrl>,
    ) -> Result<(), Error> {
        // Beings a new transaction
        let mut tx = self.pool.begin().await?;

        let pk_id: i32 = get_or_save_public_key(&mut tx, &event.pubkey).await?;

        // TODO: if NIP65 event, update relays

        // TODO: if NIP-17 event, update relays

        // TODO: extract hints from `p` tags. Only p tags or also additional ones?

        if let Some(relay_url) = relay_url {
            let relay_id: i32 = get_or_save_relay_url(&mut tx, relay_url).await?;

            let bitflag: Flags = Flags::HINT; // TODO: this should be something else, like RECEIVED?
            let now: u64 = Timestamp::now().as_u64();

            sqlx::query(
                r#"
        INSERT INTO relays_per_user (public_key_id, relay_id, bitflags, received_events, last_received_event)
        VALUES ($1, $2, $3, 1, $4)
        ON CONFLICT (public_key_id, relay_id)
        DO UPDATE SET
            bitflags = bitflags | excluded.bitflags,
            received_events = received_events + 1,
            last_received_event = excluded.last_received_event
        "#)
                .bind(pk_id)
                .bind(relay_id)
                .bind(bitflag.as_u16())
                .bind(now as i64)
                .execute(&mut *tx)
                .await?;
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }
}

async fn get_or_save_public_key<'a>(
    tx: &mut Transaction<'a, Sqlite>,
    public_key: &PublicKey,
) -> Result<i32, Error> {
    match get_id_by_public_key(tx, public_key).await? {
        Some(id) => Ok(id),
        None => save_public_key(tx, public_key).await,
    }
}

async fn get_id_by_public_key<'a>(
    tx: &mut Transaction<'a, Sqlite>,
    public_key: &PublicKey,
) -> Result<Option<i32>, Error> {
    let pk_id: Option<(i32,)> = sqlx::query_as("SELECT id FROM public_keys WHERE public_key = $1")
        .bind(public_key.as_bytes().as_slice())
        .fetch_optional(&mut **tx)
        .await?;
    Ok(pk_id.map(|(p,)| p))
}

async fn save_public_key<'a>(
    tx: &mut Transaction<'a, Sqlite>,
    public_key: &PublicKey,
) -> Result<i32, Error> {
    let pk_id: (i32,) = sqlx::query_as("INSERT INTO public_keys (public_key) VALUES ($1) ON CONFLICT (public_key) DO NOTHING RETURNING id")
        .bind(public_key.as_bytes().as_slice())
        .fetch_one(&mut **tx)
        .await?;
    Ok(pk_id.0)
}

async fn get_or_save_relay_url<'a>(
    tx: &mut Transaction<'a, Sqlite>,
    relay_url: &RelayUrl,
) -> Result<i32, Error> {
    match get_id_by_relay_url(tx, relay_url).await? {
        Some(id) => Ok(id),
        None => save_relay_url(tx, relay_url).await,
    }
}

async fn get_id_by_relay_url<'a>(
    tx: &mut Transaction<'a, Sqlite>,
    relay_url: &RelayUrl,
) -> Result<Option<i32>, Error> {
    let pk_id: Option<(i32,)> = sqlx::query_as("SELECT id FROM relays WHERE url = $1")
        .bind(relay_url.as_str_without_trailing_slash())
        .fetch_optional(&mut **tx)
        .await?;
    Ok(pk_id.map(|(p,)| p))
}

async fn save_relay_url<'a>(
    tx: &mut Transaction<'a, Sqlite>,
    relay_url: &RelayUrl,
) -> Result<i32, Error> {
    let pk_id: (i32,) = sqlx::query_as(
        "INSERT INTO relays (url) VALUES ($1) ON CONFLICT (url) DO NOTHING RETURNING id",
    )
    .bind(relay_url.as_str_without_trailing_slash())
    .fetch_one(&mut **tx)
    .await?;
    Ok(pk_id.0)
}

impl NostrGossip for NostrGossipSqlite {
    fn process<'a>(
        &'a self,
        event: &'a Event,
        relay_url: Option<&'a RelayUrl>,
    ) -> BoxedFuture<'a, Result<(), GossipError>> {
        Box::pin(async move {
            self.process_event(event, relay_url)
                .await
                .map_err(GossipError::backend)
        })
    }

    fn status(
        &self,
        public_key: &PublicKey,
    ) -> BoxedFuture<Result<GossipPublicKeyStatus, GossipError>> {
        todo!()
    }

    fn update_fetch_attempt(&self, public_key: PublicKey) -> BoxedFuture<Result<(), GossipError>> {
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

#[cfg(test)]
mod tests {
    use nostr::JsonUtil;
    use tempfile::TempDir;

    use super::*;

    async fn setup() -> (NostrGossipSqlite, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().join("test.db");

        let store = NostrGossipSqlite::open(path).await.unwrap();

        (store, temp_dir)
    }

    #[tokio::test]
    async fn test_process_event() {
        let (store, _temp_dir) = setup().await;

        let json = r#"{"id":"b7b1fb52ad8461a03e949820ae29a9ea07e35bcd79c95c4b59b0254944f62805","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644581,"kind":1,"tags":[],"content":"Text note","sig":"ed73a8a4e7c26cd797a7b875c634d9ecb6958c57733305fed23b978109d0411d21b3e182cb67c8ad750884e30ca383b509382ae6187b36e76ee76e6a142c4284"}"#;
        let event = Event::from_json(json).unwrap();

        // First process
        store.process(&event, None).await.unwrap();

        // Re-process the same event
        store.process(&event, None).await.unwrap();
    }
}
