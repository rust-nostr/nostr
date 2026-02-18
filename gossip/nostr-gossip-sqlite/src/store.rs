//! Nostr gossip SQLite store.

use std::cmp;
use std::collections::{BTreeSet, HashSet};
use std::num::NonZeroUsize;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use nostr::nips::nip17;
use nostr::nips::nip65::{self, RelayMetadata};
use nostr::util::BoxedFuture;
use nostr::{Event, Kind, PublicKey, RelayUrl, TagKind, TagStandard, Timestamp};
use nostr_gossip::error::GossipError;
use nostr_gossip::flags::GossipFlags;
use nostr_gossip::{
    BestRelaySelection, GossipAllowedRelays, GossipListKind, GossipPublicKeyStatus, NostrGossip,
    OutdatedPublicKey,
};
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Executor, Sqlite, SqlitePool, Transaction};
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::constant::{READ_WRITE_FLAGS, RELAYS_QUERY_LIMIT, TTL_OUTDATED};
use crate::error::Error;
use crate::model::ListRow;

struct SqlTx<'a> {
    tx: Transaction<'a, Sqlite>,
    _permit: SemaphorePermit<'a>,
}

impl<'a> Deref for SqlTx<'a> {
    type Target = Transaction<'a, Sqlite>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl DerefMut for SqlTx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tx
    }
}

impl SqlTx<'_> {
    #[inline]
    async fn commit(self) -> Result<(), Error> {
        Ok(self.tx.commit().await?)
    }
}

/// Nostr Gossip SQLite store.
#[derive(Debug, Clone)]
pub struct NostrGossipSqlite {
    pool: SqlitePool,
    write_semaphore: Arc<Semaphore>,
}

impl NostrGossipSqlite {
    async fn new(opts: SqliteConnectOptions) -> Result<Self, Error> {
        // Create a connection pool.
        let pool: SqlitePool = SqlitePool::connect_with(opts).await?;

        // Run migrations
        let migrator: Migrator = sqlx::migrate!();
        migrator.run(&pool).await?;

        // Construct
        Ok(Self {
            pool,
            // Limit concurrent writes to 1
            write_semaphore: Arc::new(Semaphore::new(1)),
        })
    }

    /// Open a persistent database
    pub async fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        // Built options
        let opts: SqliteConnectOptions = SqliteConnectOptions::new()
            .busy_timeout(Duration::from_secs(60))
            .journal_mode(SqliteJournalMode::Wal)
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

    async fn write_tx(&self) -> Result<SqlTx, Error> {
        // Acquire permit to write
        let permit = self.write_semaphore.acquire().await?;

        // Being transaction
        let tx: Transaction<Sqlite> = self.pool.begin().await?;

        Ok(SqlTx {
            tx,
            _permit: permit,
        })
    }

    async fn process_event(
        &self,
        event: &Event,
        relay_url: Option<&RelayUrl>,
    ) -> Result<(), Error> {
        // Beings a new transaction
        let mut tx = self.write_tx().await?;

        // Save public key and get ID
        let pk_id: i32 = get_or_save_public_key(&mut tx, &event.pubkey).await?;

        // Check the event kind
        match &event.kind {
            // Extract NIP-65 relays
            Kind::RelayList => {
                update_nip65_relays(&mut tx, pk_id, nip65::extract_relay_list(event)).await?
            }
            // Extract NIP-17 relays
            Kind::InboxRelays => {
                update_nip17_relays(&mut tx, pk_id, nip17::extract_relay_list(event)).await?
            }
            // Extract hints
            _ => update_hints(&mut tx, event).await?,
        }

        if let Some(relay_url) = relay_url {
            update_relay_per_user(&mut tx, pk_id, relay_url, GossipFlags::RECEIVED).await?;
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }

    async fn get_status(
        &self,
        public_key: &PublicKey,
        list: GossipListKind,
    ) -> Result<GossipPublicKeyStatus, Error> {
        // Get public key ID
        match get_id_by_public_key(&self.pool, public_key).await? {
            Some(pk_id) => {
                let row: Option<ListRow> = sqlx::query_as(
                    "SELECT event_created_at, last_checked_at FROM lists WHERE public_key_id = $1 AND event_kind = $2",
                )
                    .bind(pk_id)
                    .bind(list.to_event_kind().as_u16())
                    .fetch_optional(&self.pool)
                    .await?;

                match row {
                    Some(row) => {
                        let now: Timestamp = Timestamp::now();
                        let last: i64 = row.last_checked_at.unwrap_or(0);
                        let last: Timestamp = last.try_into()?;

                        if last + TTL_OUTDATED < now {
                            Ok(GossipPublicKeyStatus::Outdated {
                                created_at: match row.event_created_at {
                                    Some(t) => Some(t.try_into()?),
                                    None => None,
                                },
                            })
                        } else {
                            Ok(GossipPublicKeyStatus::Updated)
                        }
                    }
                    None => Ok(GossipPublicKeyStatus::Missing),
                }
            }
            None => Ok(GossipPublicKeyStatus::Missing),
        }
    }

    async fn _update_fetch_attempt(
        &self,
        public_key: &PublicKey,
        list: GossipListKind,
    ) -> Result<(), Error> {
        // Beings a new transaction
        let mut tx = self.write_tx().await?;

        // Save public key and get ID
        let pk_id: i32 = get_or_save_public_key(&mut tx, public_key).await?;

        let now: i64 = Timestamp::now().as_secs() as i64;

        sqlx::query(
            r#"
            INSERT INTO lists (public_key_id, event_kind, last_checked_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (public_key_id, event_kind)
            DO UPDATE SET last_checked_at = excluded.last_checked_at
            "#,
        )
        .bind(pk_id)
        .bind(list.to_event_kind().as_u16())
        .bind(now)
        .execute(&mut **tx)
        .await?;

        // Write changes
        tx.commit().await?;

        Ok(())
    }

    async fn get_outdated_public_keys(
        &self,
        list: GossipListKind,
        limit: NonZeroUsize,
    ) -> Result<BTreeSet<OutdatedPublicKey>, Error> {
        let now: i64 = Timestamp::now().as_secs() as i64;
        let threshold: i64 = now.saturating_sub(TTL_OUTDATED.as_secs() as i64);

        let rows: Vec<(Vec<u8>, Option<i64>)> = sqlx::query_as(
            r#"
            SELECT pk.public_key, l.last_checked_at
            FROM lists l
            INNER JOIN public_keys pk ON l.public_key_id = pk.id
            WHERE l.event_kind = $1
              AND COALESCE(l.last_checked_at, 0) > 0
              AND l.last_checked_at < $2
            ORDER BY l.last_checked_at ASC
            LIMIT $3
            "#,
        )
        .bind(list.to_event_kind().as_u16())
        .bind(threshold)
        .bind(limit.get() as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut public_keys: BTreeSet<OutdatedPublicKey> = BTreeSet::new();

        for (pk, timestamp) in rows.into_iter() {
            let last: i64 = timestamp.unwrap_or(0);

            if let (Ok(pk), Ok(last)) = (PublicKey::from_slice(&pk), last.try_into()) {
                public_keys.insert(OutdatedPublicKey::new(pk, last));
            }
        }

        Ok(public_keys)
    }

    async fn _get_best_relays(
        &self,
        public_key: &PublicKey,
        selection: BestRelaySelection,
        allowed: GossipAllowedRelays,
    ) -> Result<HashSet<RelayUrl>, Error> {
        let mut relays: HashSet<RelayUrl> = HashSet::new();

        match selection {
            BestRelaySelection::All {
                read,
                write,
                hints,
                most_received,
            } => {
                // Get read relays
                relays.extend(
                    self.get_relays_by_flag(public_key, GossipFlags::READ, allowed, read)
                        .await?,
                );

                // Get write relays
                relays.extend(
                    self.get_relays_by_flag(public_key, GossipFlags::WRITE, allowed, write)
                        .await?,
                );

                // Get hint relays
                relays.extend(
                    self.get_relays_by_flag(public_key, GossipFlags::HINT, allowed, hints)
                        .await?,
                );

                // Get most received relays
                relays.extend(
                    self.get_relays_by_flag(
                        public_key,
                        GossipFlags::RECEIVED,
                        allowed,
                        most_received,
                    )
                    .await?,
                );
            }
            BestRelaySelection::Read { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, GossipFlags::READ, allowed, limit)
                        .await?,
                );
            }
            BestRelaySelection::Write { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, GossipFlags::WRITE, allowed, limit)
                        .await?,
                );
            }
            BestRelaySelection::PrivateMessage { limit } => {
                relays.extend(
                    self.get_relays_by_flag(
                        public_key,
                        GossipFlags::PRIVATE_MESSAGE,
                        allowed,
                        limit,
                    )
                    .await?,
                );
            }
            BestRelaySelection::Hints { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, GossipFlags::HINT, allowed, limit)
                        .await?,
                );
            }
            BestRelaySelection::MostReceived { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, GossipFlags::RECEIVED, allowed, limit)
                        .await?,
                );
            }
        }

        Ok(relays)
    }

    async fn get_relays_by_flag(
        &self,
        public_key: &PublicKey,
        flag: GossipFlags,
        allowed: GossipAllowedRelays,
        limit: u8,
    ) -> Result<Vec<RelayUrl>, Error> {
        let query = r#"
            SELECT r.url
            FROM relays_per_user rpu
            INNER JOIN relays r ON rpu.relay_id = r.id
            INNER JOIN public_keys pk ON rpu.public_key_id = pk.id
            WHERE pk.public_key = $1 AND (rpu.bitflags & $2) = $2
            ORDER BY rpu.received_events DESC, rpu.last_received_event DESC
            LIMIT $3
        "#;

        let query_limit: u8 = cmp::max(limit, RELAYS_QUERY_LIMIT);

        let rows: Vec<(String,)> = sqlx::query_as(query)
            .bind(public_key.as_bytes().as_slice())
            .bind(flag.as_u32())
            .bind(query_limit)
            .fetch_all(&self.pool)
            .await?;

        let mut relays = Vec::with_capacity(rows.len());
        for (url,) in rows.into_iter() {
            if relays.len() >= limit as usize {
                break;
            }

            if let Ok(relay_url) = RelayUrl::parse(&url) {
                // Check if the relay is allowed by the allowed relays filter
                if !allowed.is_allowed(&relay_url) {
                    continue;
                }

                relays.push(relay_url);
            }
        }

        Ok(relays)
    }
}

async fn get_or_save_public_key(
    tx: &mut Transaction<'_, Sqlite>,
    public_key: &PublicKey,
) -> Result<i32, Error> {
    match get_id_by_public_key(&mut **tx, public_key).await? {
        Some(id) => Ok(id),
        None => save_public_key(tx, public_key).await,
    }
}

async fn get_id_by_public_key<'a, E>(
    executor: E,
    public_key: &PublicKey,
) -> Result<Option<i32>, Error>
where
    E: Executor<'a, Database = Sqlite>,
{
    let pk_id: Option<(i32,)> = sqlx::query_as("SELECT id FROM public_keys WHERE public_key = $1")
        .bind(public_key.as_bytes().as_slice())
        .fetch_optional(executor)
        .await?;
    Ok(pk_id.map(|(p,)| p))
}

async fn save_public_key(
    tx: &mut Transaction<'_, Sqlite>,
    public_key: &PublicKey,
) -> Result<i32, Error> {
    let pk_id: (i32,) = sqlx::query_as("INSERT INTO public_keys (public_key) VALUES ($1) ON CONFLICT (public_key) DO NOTHING RETURNING id")
        .bind(public_key.as_bytes().as_slice())
        .fetch_one(&mut **tx)
        .await?;
    Ok(pk_id.0)
}

async fn get_or_save_relay_url(
    tx: &mut Transaction<'_, Sqlite>,
    relay_url: &RelayUrl,
) -> Result<i32, Error> {
    match get_id_by_relay_url(tx, relay_url).await? {
        Some(id) => Ok(id),
        None => save_relay_url(tx, relay_url).await,
    }
}

async fn get_id_by_relay_url(
    tx: &mut Transaction<'_, Sqlite>,
    relay_url: &RelayUrl,
) -> Result<Option<i32>, Error> {
    let pk_id: Option<(i32,)> = sqlx::query_as("SELECT id FROM relays WHERE url = $1")
        .bind(relay_url.as_str_without_trailing_slash())
        .fetch_optional(&mut **tx)
        .await?;
    Ok(pk_id.map(|(p,)| p))
}

async fn save_relay_url(
    tx: &mut Transaction<'_, Sqlite>,
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

async fn remove_flag_from_user_relays(
    tx: &mut Transaction<'_, Sqlite>,
    public_key_id: i32,
    flags_to_remove: GossipFlags,
) -> Result<(), Error> {
    sqlx::query("UPDATE relays_per_user SET bitflags = (bitflags & ~$1) WHERE public_key_id = $2")
        .bind(flags_to_remove.as_u32())
        .bind(public_key_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Add relay per user or update the received events and bitflags.
async fn update_relay_per_user(
    tx: &mut Transaction<'_, Sqlite>,
    public_key_id: i32,
    relay_url: &RelayUrl,
    flags: GossipFlags,
) -> Result<(), Error> {
    let relay_id: i32 = get_or_save_relay_url(tx, relay_url).await?;

    let now: u64 = Timestamp::now().as_secs();

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
        .bind(public_key_id)
        .bind(relay_id)
        .bind(flags.as_u32())
        .bind(now as i64)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

async fn update_nip65_relays<'a, I>(
    tx: &mut Transaction<'_, Sqlite>,
    public_key_id: i32,
    iter: I,
) -> Result<(), Error>
where
    I: IntoIterator<Item = (&'a RelayUrl, &'a Option<RelayMetadata>)>,
{
    // Remove all READ and WRITE flags from the relays of the public key
    remove_flag_from_user_relays(tx, public_key_id, READ_WRITE_FLAGS).await?;

    // Extract relay list
    for (relay_url, metadata) in iter {
        // Save relay and get ID
        let relay_id: i32 = get_or_save_relay_url(tx, relay_url).await?;

        // New bitflag for the relay
        let bitflag: GossipFlags = match metadata {
            Some(RelayMetadata::Read) => GossipFlags::READ,
            Some(RelayMetadata::Write) => GossipFlags::WRITE,
            None => READ_WRITE_FLAGS,
        };

        // Update bitflag
        sqlx::query(
            r#"
                    INSERT INTO relays_per_user (public_key_id, relay_id, bitflags)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (public_key_id, relay_id)
                    DO UPDATE SET
                        bitflags = bitflags | excluded.bitflags
                    "#,
        )
        .bind(public_key_id)
        .bind(relay_id)
        .bind(bitflag.as_u32())
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn update_nip17_relays<'a, I>(
    tx: &mut Transaction<'_, Sqlite>,
    public_key_id: i32,
    iter: I,
) -> Result<(), Error>
where
    I: IntoIterator<Item = &'a RelayUrl>,
{
    // Remove all PRIVATE_MESSAGE flag from the relays of the public key
    remove_flag_from_user_relays(tx, public_key_id, GossipFlags::PRIVATE_MESSAGE).await?;

    // Extract relay list
    for relay_url in iter {
        let relay_id: i32 = get_or_save_relay_url(tx, relay_url).await?;

        sqlx::query(
            r#"
                    INSERT INTO relays_per_user (public_key_id, relay_id, bitflags)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (public_key_id, relay_id)
                    DO UPDATE SET
                        bitflags = bitflags | excluded.bitflags
                    "#,
        )
        .bind(public_key_id)
        .bind(relay_id)
        .bind(GossipFlags::PRIVATE_MESSAGE.as_u32())
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn update_hints(tx: &mut Transaction<'_, Sqlite>, event: &Event) -> Result<(), Error> {
    for tag in event.tags.filter_standardized(TagKind::p()) {
        if let TagStandard::PublicKey {
            public_key,
            relay_url: Some(relay_url),
            ..
        } = tag
        {
            let p_tag_pk_id: i32 = get_or_save_public_key(tx, public_key).await?;
            update_relay_per_user(tx, p_tag_pk_id, relay_url, GossipFlags::HINT).await?;
        }
    }

    Ok(())
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

    fn status<'a>(
        &'a self,
        public_key: &'a PublicKey,
        list: GossipListKind,
    ) -> BoxedFuture<'a, Result<GossipPublicKeyStatus, GossipError>> {
        Box::pin(async move {
            self.get_status(public_key, list)
                .await
                .map_err(GossipError::backend)
        })
    }

    fn update_fetch_attempt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        list: GossipListKind,
    ) -> BoxedFuture<'a, Result<(), GossipError>> {
        Box::pin(async move {
            self._update_fetch_attempt(public_key, list)
                .await
                .map_err(GossipError::backend)
        })
    }

    fn outdated_public_keys(
        &self,
        list: GossipListKind,
        limit: NonZeroUsize,
    ) -> BoxedFuture<Result<BTreeSet<OutdatedPublicKey>, GossipError>> {
        Box::pin(async move {
            self.get_outdated_public_keys(list, limit)
                .await
                .map_err(GossipError::backend)
        })
    }

    fn get_best_relays<'a>(
        &'a self,
        public_key: &'a PublicKey,
        selection: BestRelaySelection,
        allowed: GossipAllowedRelays,
    ) -> BoxedFuture<'a, Result<HashSet<RelayUrl>, GossipError>> {
        Box::pin(async move {
            self._get_best_relays(public_key, selection, allowed)
                .await
                .map_err(GossipError::backend)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use nostr_gossip_test_suite::gossip_unit_tests;
    use tempfile::TempDir;

    use super::*;

    #[derive(Debug)]
    struct NostrGossipSqliteUnitTest {
        store: NostrGossipSqlite,
        _temp_dir: TempDir,
    }

    impl Deref for NostrGossipSqliteUnitTest {
        type Target = NostrGossipSqlite;

        fn deref(&self) -> &Self::Target {
            &self.store
        }
    }

    async fn setup() -> NostrGossipSqliteUnitTest {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().join("test.db");

        let store = NostrGossipSqlite::open(path).await.unwrap();

        NostrGossipSqliteUnitTest {
            store,
            _temp_dir: temp_dir,
        }
    }

    gossip_unit_tests!(NostrGossipSqliteUnitTest, setup);
}
