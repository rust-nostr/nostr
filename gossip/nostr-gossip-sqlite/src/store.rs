//! Nostr gossip SQLite store.

use std::collections::HashSet;
use std::path::Path;

use nostr::nips::nip17;
use nostr::nips::nip65::{self, RelayMetadata};
use nostr::util::BoxedFuture;
use nostr::{Event, Kind, PublicKey, RelayUrl, TagKind, TagStandard, Timestamp};
use nostr_gossip::error::GossipError;
use nostr_gossip::{BestRelaySelection, GossipListKind, GossipPublicKeyStatus, NostrGossip};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Executor, Sqlite, SqlitePool, Transaction};

use crate::constant::{PUBKEY_METADATA_OUTDATED_AFTER, READ_WRITE_FLAGS};
use crate::error::Error;
use crate::flags::Flags;
use crate::model::ListRow;

/// Nostr Gossip SQLite store.
#[derive(Debug, Clone)]
pub struct NostrGossipSqlite {
    pool: SqlitePool,
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
            update_relay_per_user(&mut tx, pk_id, relay_url, Flags::RECEIVED).await?;
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
                        let last: Timestamp =
                            Timestamp::from_i64_secs(row.last_checked_at.unwrap_or(0));

                        if last + PUBKEY_METADATA_OUTDATED_AFTER < now {
                            Ok(GossipPublicKeyStatus::Outdated {
                                created_at: row.event_created_at.map(Timestamp::from_i64_secs),
                            })
                        } else {
                            Ok(GossipPublicKeyStatus::Updated)
                        }
                    }
                    None => Ok(GossipPublicKeyStatus::Outdated { created_at: None }),
                }
            }
            None => Ok(GossipPublicKeyStatus::Outdated { created_at: None }),
        }
    }

    async fn _update_fetch_attempt(
        &self,
        public_key: &PublicKey,
        list: GossipListKind,
    ) -> Result<(), Error> {
        // Beings a new transaction
        let mut tx = self.pool.begin().await?;

        // Save public key and get ID
        let pk_id: i32 = get_or_save_public_key(&mut tx, public_key).await?;

        let now: i64 = Timestamp::now().as_u64() as i64;

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
        .execute(&mut *tx)
        .await?;

        // Write changes
        tx.commit().await?;

        Ok(())
    }

    async fn _get_best_relays(
        &self,
        public_key: &PublicKey,
        selection: BestRelaySelection,
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
                    self.get_relays_by_flag(public_key, Flags::READ, read)
                        .await?,
                );

                // Get write relays
                relays.extend(
                    self.get_relays_by_flag(public_key, Flags::WRITE, write)
                        .await?,
                );

                // Get hint relays
                relays.extend(
                    self.get_relays_by_flag(public_key, Flags::HINT, hints)
                        .await?,
                );

                // Get most received relays
                relays.extend(
                    self.get_relays_by_flag(public_key, Flags::RECEIVED, most_received)
                        .await?,
                );
            }
            BestRelaySelection::Read { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, Flags::READ, limit)
                        .await?,
                );
            }
            BestRelaySelection::Write { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, Flags::WRITE, limit)
                        .await?,
                );
            }
            BestRelaySelection::PrivateMessage { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, Flags::PRIVATE_MESSAGE, limit)
                        .await?,
                );
            }
            BestRelaySelection::Hints { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, Flags::HINT, limit)
                        .await?,
                );
            }
            BestRelaySelection::MostReceived { limit } => {
                relays.extend(
                    self.get_relays_by_flag(public_key, Flags::RECEIVED, limit)
                        .await?,
                );
            }
        }

        Ok(relays)
    }

    async fn get_relays_by_flag(
        &self,
        public_key: &PublicKey,
        flag: Flags,
        limit: usize,
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

        let rows: Vec<(String,)> = sqlx::query_as(query)
            .bind(public_key.as_bytes().as_slice())
            .bind(flag)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        let mut relays = Vec::with_capacity(rows.len());
        for (url,) in rows.into_iter() {
            if let Ok(relay_url) = RelayUrl::parse(&url) {
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
    flags_to_remove: Flags,
) -> Result<(), Error> {
    sqlx::query("UPDATE relays_per_user SET bitflags = (bitflags & ~$1) WHERE public_key_id = $2")
        .bind(flags_to_remove)
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
    flags: Flags,
) -> Result<(), Error> {
    let relay_id: i32 = get_or_save_relay_url(tx, relay_url).await?;

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
        .bind(public_key_id)
        .bind(relay_id)
        .bind(flags)
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
        let bitflag: Flags = match metadata {
            Some(RelayMetadata::Read) => Flags::READ,
            Some(RelayMetadata::Write) => Flags::WRITE,
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
        .bind(bitflag)
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
    remove_flag_from_user_relays(tx, public_key_id, Flags::PRIVATE_MESSAGE).await?;

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
        .bind(Flags::PRIVATE_MESSAGE)
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
            update_relay_per_user(tx, p_tag_pk_id, relay_url, Flags::HINT).await?;
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

    fn get_best_relays<'a>(
        &'a self,
        public_key: &'a PublicKey,
        selection: BestRelaySelection,
    ) -> BoxedFuture<'a, Result<HashSet<RelayUrl>, GossipError>> {
        Box::pin(async move {
            self._get_best_relays(public_key, selection)
                .await
                .map_err(GossipError::backend)
        })
    }
}

#[cfg(test)]
mod tests {
    use nostr::{EventBuilder, JsonUtil, Keys, Tag};
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

    #[tokio::test]
    async fn test_process_nip65_relay_list() {
        let (store, _temp_dir) = setup().await;

        // NIP-65 relay list event with read and write relays
        let json = r#"{"id":"0a49bed4a1eb0973a68a0d43b7ca62781ffd4e052b91bbadef09e5cf756f6e68","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1759351841,"kind":10002,"tags":[["alt","Relay list to discover the user's content"],["r","wss://relay.damus.io/"],["r","wss://nostr.wine/"],["r","wss://nostr.oxtr.dev/"],["r","wss://relay.nostr.wirednet.jp/"]],"content":"","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let event = Event::from_json(json).unwrap();

        store.process(&event, None).await.unwrap();

        let public_key = event.pubkey;

        // Test Read selection
        let read_relays = store
            ._get_best_relays(&public_key, BestRelaySelection::Read { limit: 2 })
            .await
            .unwrap();

        assert_eq!(read_relays.len(), 2); // relay.damus.io and nos.lol

        // Test Write selection
        let write_relays = store
            ._get_best_relays(&public_key, BestRelaySelection::Write { limit: 2 })
            .await
            .unwrap();

        assert_eq!(write_relays.len(), 2); // relay.damus.io and relay.nostr.band
    }

    #[tokio::test]
    async fn test_process_nip17_inbox_relays() {
        let (store, _temp_dir) = setup().await;

        // NIP-17 inbox relays event
        let json = r#"{"id":"8d9b40907f80bd7d5014bdc6a2541227b92f4ae20cbff59792b4746a713da81e","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1756718818,"kind":10050,"tags":[["relay","wss://auth.nostr1.com/"],["relay","wss://nostr.oxtr.dev/"],["relay","wss://nip17.com"]],"content":"","sig":"05611df32f5c4e55bb8d74ab2840378b7707ad162f785a78f8bdaecee5b872667e4e43bcbbf3c6c638335c637f001155b48b7a7040ce2695660467be62f142d5"}"#;
        let event = Event::from_json(json).unwrap();

        store.process(&event, None).await.unwrap();

        let public_key = event.pubkey;

        // Test PrivateMessage selection
        let pm_relays = store
            ._get_best_relays(&public_key, BestRelaySelection::PrivateMessage { limit: 4 })
            .await
            .unwrap();

        assert_eq!(pm_relays.len(), 3); // inbox.nostr.wine and relay.primal.net
    }

    #[tokio::test]
    async fn test_process_hints_from_p_tags() {
        let (store, _temp_dir) = setup().await;

        let public_key =
            PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
                .unwrap();
        let relay_url = RelayUrl::parse("wss://hint.relay.io").unwrap();

        let keys = Keys::generate();
        let event = EventBuilder::text_note("test")
            .tag(Tag::from_standardized_without_cell(
                TagStandard::PublicKey {
                    public_key,
                    relay_url: Some(relay_url.clone()),
                    alias: None,
                    uppercase: false,
                },
            ))
            .sign_with_keys(&keys)
            .unwrap();

        store.process(&event, None).await.unwrap();

        let hint_relays = store
            ._get_best_relays(&public_key, BestRelaySelection::Hints { limit: 5 })
            .await
            .unwrap();

        assert_eq!(hint_relays.len(), 1);
        assert!(hint_relays.iter().any(|r| r == &relay_url));
    }

    #[tokio::test]
    async fn test_received_events_tracking() {
        let (store, _temp_dir) = setup().await;

        let keys = Keys::generate();
        let relay_url = RelayUrl::parse("wss://test.relay.io").unwrap();

        // Process multiple events from the same relay
        for i in 0..5 {
            let event = EventBuilder::text_note(format!("Test {i}"))
                .sign_with_keys(&keys)
                .unwrap();

            store.process(&event, Some(&relay_url)).await.unwrap();
        }

        // Test MostReceived selection
        let most_received = store
            ._get_best_relays(
                &keys.public_key,
                BestRelaySelection::MostReceived { limit: 10 },
            )
            .await
            .unwrap();

        assert_eq!(most_received.len(), 1);
        assert!(most_received.iter().any(|r| r == &relay_url));
    }

    #[tokio::test]
    async fn test_best_relays_all_selection() {
        let (store, _temp_dir) = setup().await;

        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();

        // Add NIP-65 relays
        let nip65_json = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000000","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1704644581,"kind":10002,"tags":[["r","wss://read.relay.io","read"],["r","wss://write.relay.io","write"]],"content":"","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let nip65_event = Event::from_json(nip65_json).unwrap();
        store.process(&nip65_event, None).await.unwrap();

        // Add event with hints
        let hint_json = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000001","pubkey":"bb4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644581,"kind":1,"tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","wss://hint.relay.io"]],"content":"Hint","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let hint_event = Event::from_json(hint_json).unwrap();
        store.process(&hint_event, None).await.unwrap();

        // Add received events
        let relay_url = RelayUrl::parse("wss://received.relay.io").unwrap();
        let received_json = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000002","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1704644581,"kind":1,"tags":[],"content":"Received","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let received_event = Event::from_json(received_json).unwrap();
        store
            .process(&received_event, Some(&relay_url))
            .await
            .unwrap();

        // Test All selection
        let all_relays = store
            ._get_best_relays(
                &public_key,
                BestRelaySelection::All {
                    read: 5,
                    write: 5,
                    hints: 5,
                    most_received: 5,
                },
            )
            .await
            .unwrap();

        // Should have relays from all categories (duplicates removed by HashSet)
        assert!(all_relays.len() >= 3);
        assert!(all_relays
            .iter()
            .any(|r| r.as_str() == "wss://read.relay.io"));
        assert!(all_relays
            .iter()
            .any(|r| r.as_str() == "wss://write.relay.io"));
        assert!(all_relays
            .iter()
            .any(|r| r.as_str() == "wss://hint.relay.io"));
        assert!(all_relays
            .iter()
            .any(|r| r.as_str() == "wss://received.relay.io"));
    }

    #[tokio::test]
    async fn test_status_tracking() {
        let (store, _temp_dir) = setup().await;

        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();

        // Initially should be outdated
        let status = store
            .get_status(&public_key, GossipListKind::Nip65)
            .await
            .unwrap();
        assert!(matches!(status, GossipPublicKeyStatus::Outdated { .. }));

        // Process a NIP-65 event
        let json = r#"{"id":"0a49bed4a1eb0973a68a0d43b7ca62781ffd4e052b91bbadef09e5cf756f6e68","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1759351841,"kind":10002,"tags":[["alt","Relay list to discover the user's content"],["r","wss://relay.damus.io/"],["r","wss://nostr.wine/"],["r","wss://nostr.oxtr.dev/"],["r","wss://relay.nostr.wirednet.jp/"]],"content":"","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let event = Event::from_json(json).unwrap();
        store.process(&event, None).await.unwrap();

        // Update fetch attempt
        store
            .update_fetch_attempt(&public_key, GossipListKind::Nip65)
            .await
            .unwrap();

        // Should now be updated
        let status = store
            .get_status(&public_key, GossipListKind::Nip65)
            .await
            .unwrap();
        assert!(matches!(status, GossipPublicKeyStatus::Updated));
    }

    #[tokio::test]
    async fn test_empty_results() {
        let (store, _temp_dir) = setup().await;

        // Random public key with no data
        let public_key =
            PublicKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        // Should return empty set
        let relays = store
            ._get_best_relays(&public_key, BestRelaySelection::Read { limit: 10 })
            .await
            .unwrap();

        assert_eq!(relays.len(), 0);
    }
}
