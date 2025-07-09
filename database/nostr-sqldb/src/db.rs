// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr SQL

use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(feature = "sqlite")]
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
#[cfg(feature = "postgres")]
use sqlx::postgres::{PgConnectOptions, PgPool};
#[cfg(feature = "mysql")]
use sqlx::mysql::{MySql, MySqlPool};
use sqlx::{Transaction, QueryBuilder};
use nostr_database::prelude::*;
use tokio::sync::Mutex;

use crate::error::Error;
use crate::model::{EventDataDb, EventDb, EventTagDb};

const EVENTS_QUERY_LIMIT: usize = 10_000;

/// SQL backend
pub enum NostrSqlBackend {
    /// SQLite
    #[cfg(feature = "sqlite")]
    Sqlite {
        /// SQLite database path
        ///
        /// If no path is passed, an in-memory database will be created.
        path: Option<PathBuf>
    },
    /// Postgres
    #[cfg(feature = "postgres")]
    Postgres {
        /// Host
        host: String,
        /// Port
        port: u16,
        /// Username
        username: Option<String>,
        /// Password
        password: Option<String>,
        /// Database name
        database: String,
    }
}

impl NostrSqlBackend {
    /// New persistent SQLite database
    #[inline]
    #[cfg(feature = "sqlite")]
    pub fn sqlite<P>(path: P) -> Self
    where
        P: AsRef<Path>
    {
        Self::Sqlite { path: Some(path.as_ref().to_path_buf()) }
    }

    /// New in-memory SQLite database
    #[inline]
    #[cfg(feature = "sqlite")]
    pub fn sqlite_memory() -> Self {
        Self::Sqlite { path: None }
    }
}

#[derive(Debug, Clone)]
enum Db {
    #[cfg(feature = "sqlite")]
    Sqlite(SqlitePool),
    #[cfg(feature = "postgres")]
    Postgres(PgPool),
    #[cfg(feature = "mysql")]
    MySql(MySqlPool),
}

/// Nostr SQL database
#[derive(Clone)]
pub struct NostrSql {
    pool: Db,
    fbb: Arc<Mutex<FlatBufferBuilder<'static>>>,
}

impl fmt::Debug for NostrSql {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NostrSql")
            .field("pool", &self.pool)
            .finish()
    }
}

impl NostrSql {
    /// Connect to a SQL database
    pub async fn new(backend: NostrSqlBackend) -> Result<Self, Error> {
        let pool = match backend {
            #[cfg(feature = "sqlite")]
            NostrSqlBackend::Sqlite {path} => {
                let mut opts: SqliteConnectOptions = SqliteConnectOptions::new().create_if_missing(true);
                
                match path {
                    Some(path) => opts = opts.filename(path),
                    None => opts = opts.in_memory(true),
                };
                
                
                let pool: SqlitePool = SqlitePool::connect_with(opts).await?;

                sqlx::migrate!("migrations/sqlite").run(&pool).await?;

                Db::Sqlite(pool)
            }
            #[cfg(feature = "postgres")]
            NostrSqlBackend::Postgres {host, port, username, password, database } => {
                let mut opts: PgConnectOptions = PgConnectOptions::new_without_pgpass().host(&host).port(port).database(&database);
                
                if let Some(username) = username {
                    opts = opts.username(&username);
                }

                if let Some(password) = password {
                    opts = opts.password(&password);
                }

                let pool: PgPool = PgPool::connect_with(opts).await?;

                sqlx::migrate!("migrations/postgres").run(&pool).await?;

                Db::Postgres(pool)
            }
        };

        Ok(Self {
            pool,
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::new())),
        })
    }

    /// Returns true if successfully inserted
    async fn insert_event_tx(&self, tx: &mut Transaction<'_, Any>, event: &EventDb) -> Result<bool, Error> {
        let sql: &str = match self.kind {
            #[cfg(feature = "sqlite")]
            PoolKind::Sqlite => {
                "INSERT OR IGNORE INTO events (id, pubkey, created_at, kind, payload, deleted) VALUES (?, ?, ?, ?, ?, ?)"
            },
            #[cfg(feature = "postgres")]
            PoolKind::Postgres => {
                "INSERT INTO events (id, pubkey, created_at, kind, payload, deleted) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING"
            },
            #[cfg(feature = "mysql")]
            PoolKind::MySql => {
                "INSERT IGNORE INTO events (id, pubkey, created_at, kind, payload, deleted) VALUES (?, ?, ?, ?, ?, ?)"
            },
        };

        let result = sqlx::query(sql)
            .bind(&event.id)
            .bind(&event.pubkey)
            .bind(event.created_at)
            .bind(event.kind)
            .bind(&event.payload)
            .bind(event.deleted)
            .execute(&mut **tx)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn insert_tags_tx(&self, tx: &mut Transaction<'_, Any>, tags: &[EventTagDb]) -> Result<(), Error> {
        let sql: &str = match self.kind {
            #[cfg(feature = "sqlite")]
            PoolKind::Sqlite => {
                "INSERT OR IGNORE INTO event_tags (tag, tag_value, event_id) VALUES (?, ?, ?)"
            },
            #[cfg(feature = "postgres")]
            PoolKind::Postgres => {
                "INSERT INTO event_tags (tag, tag_value, event_id) VALUES (?, ?, ?) ON CONFLICT (tag, tag_value, event_id) DO NOTHING"
            },
            #[cfg(feature = "mysql")]
            PoolKind::MySql => {
                "INSERT IGNORE INTO event_tags (tag, tag_value, event_id) VALUES (?, ?, ?)"
            },
        };

        for tag in tags {
            sqlx::query(sql)
                .bind(&tag.tag)
                .bind(&tag.tag_value)
                .bind(&tag.event_id)
                .execute(&mut **tx)
                .await?;
        }

        Ok(())
    }


    async fn _save_event(&self, event: &Event) -> Result<SaveEventStatus, Error> {
        if event.kind.is_ephemeral() {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Ephemeral));
        }

        let mut tx = self.pool.begin().await?;

        // Convert event
        let data: EventDataDb = {
            let mut fbb = self.fbb.lock().await;
            EventDataDb::from_event(event, &mut fbb)
        };

        // TODO: check if event is deleted
        // TODO: check if is replaced

        // Insert event first
        let inserted: bool = self.insert_event_tx(&mut tx, &data.event).await?;

        // Check if the event has been inserted
        if inserted {
            // Insert tags
            if !data.tags.is_empty() {
                self.insert_tags_tx(&mut tx, &data.tags).await?;
            }

            // Commit transaction
            tx.commit().await?;

            Ok(SaveEventStatus::Success)
        } else {
            // Event has not been inserted, rollback transaction
            tx.rollback().await?;
            Ok(SaveEventStatus::Rejected(RejectedReason::Duplicate))
        }
    }

    async fn get_event_by_id(&self, id: &EventId) -> Result<Option<EventDb>, Error> {
        let query = sqlx::query_as::<Any, EventDb>("SELECT * FROM events WHERE id = ?")
            .bind(id.as_bytes().to_vec());
        Ok(query.fetch_optional(&self.pool).await?)
    }
}

impl NostrDatabase for NostrSql {
    fn backend(&self) -> Backend {
        match self.kind {
            #[cfg(feature = "sqlite")]
            PoolKind::Sqlite => Backend::SQLite,
            #[cfg(feature = "postgres")]
            PoolKind::Postgres => Backend::Custom(String::from("Postgres")),
            #[cfg(feature = "mysql")]
            PoolKind::MySql => Backend::Custom(String::from("MySQL")),
        }
    }

    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>> {
        Box::pin(async move {
            self._save_event(event).await.map_err(DatabaseError::backend)
        })
    }

    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        Box::pin(async move {
            match self.get_event_by_id(event_id).await.map_err(DatabaseError::backend)? {
                Some(e) if e.deleted => Ok(DatabaseEventStatus::Deleted),
                Some(_) => Ok(DatabaseEventStatus::Saved),
                None => Ok(DatabaseEventStatus::NotExistent),
            }
        })
    }

    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            match self.get_event_by_id(event_id).await.map_err(DatabaseError::backend)? {
                Some(e) if !e.deleted => {
                    Ok(Some(Event::decode(&e.payload).map_err(DatabaseError::backend)?))
                }
                _ => Ok(None),
            }
        })
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move {
            Ok(self.query(filter).await?.len())
        })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        Box::pin(async move {
            // Limit filter query
            let filter: Filter = with_limit(filter, EVENTS_QUERY_LIMIT);

            let mut events: Events = Events::new(&filter);

            let sql = build_filter_query(filter);

            let payloads: Vec<(Vec<u8>,)> = sqlx::query_as(&sql).fetch_all(&self.pool).await.map_err(DatabaseError::backend)?;

            for (payload,) in payloads.into_iter() {
                if let Ok(event) = Event::decode(&payload) {
                    events.insert(event);
                }
            }
            Ok(events)
        })
    }

    fn delete(&self, _filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        // Box::pin(async move {
        //     let filter = with_limit(filter, 999);
        //     let filter = build_filter_query(filter);
        //     diesel::update(events::table)
        //         .set(events::deleted.eq(true))
        //         .filter(events::id.eq_any(filter.select(events::id)))
        //         .execute(&mut self.get_connection().await?)
        //         .await
        //         .map_err(DatabaseError::backend)?;
        //
        //     Ok(())
        // })
        Box::pin(async move { Err(DatabaseError::NotSupported )})
    }

    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { Err(DatabaseError::NotSupported )})
    }
}

fn build_filter_query(filter: Filter) -> String {
    let mut query_builder: QueryBuilder<Any> = QueryBuilder::new(
        "SELECT DISTINCT e.payload
         FROM events e
         INNER JOIN event_tags et ON e.id = et.event_id
         WHERE e.deleted = 0"
    );

    // Add filters
    if let Some(ids) = filter.ids {
        if !ids.is_empty() {
            query_builder.push(" AND e.id IN (");
            let mut separated = query_builder.separated(", ");
            for id in ids.into_iter() {
                separated.push_bind(id.as_bytes().to_vec());
            }
            query_builder.push(")");
        }
    }

    if let Some(authors) = filter.authors {
        if !authors.is_empty() {
            query_builder.push(" AND e.pubkey IN (");
            let mut separated = query_builder.separated(", ");
            for author in authors {
                separated.push_bind(author.as_bytes().to_vec());
            }
            query_builder.push(")");
        }
    }

    if let Some(kinds) = filter.kinds {
        if !kinds.is_empty() {
            query_builder.push(" AND e.kind IN (");
            let mut separated = query_builder.separated(", ");
            for kind in kinds {
                separated.push_bind(kind.as_u16() as i64);
            }
            query_builder.push(")");
        }
    }

    if let Some(since) = filter.since {
        query_builder.push(" AND e.created_at >= ");
        query_builder.push_bind(since.as_u64() as i64);
    }

    if let Some(until) = filter.until {
        query_builder.push(" AND e.created_at <= ");
        query_builder.push_bind(until.as_u64() as i64);
    }

    if !filter.generic_tags.is_empty() {
        for (tag, values) in filter.generic_tags {
            if !values.is_empty() {
                query_builder.push(" AND EXISTS (
                    SELECT 1 FROM event_tags et2
                    WHERE et2.event_id = e.id
                    AND et2.tag = ");
                query_builder.push_bind(tag.to_string());
                query_builder.push(" AND et2.tag_value IN (");

                let mut separated = query_builder.separated(", ");
                for value in values {
                    separated.push_bind(value.to_string());
                }
                query_builder.push("))");
            }
        }
    }

    query_builder.push(" ORDER BY e.created_at DESC");

    if let Some(limit) = filter.limit {
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit as i64);
    }

    query_builder.into_sql()
}

/// sets the given default limit on a Nostr filter if not set
fn with_limit(filter: Filter, default_limit: usize) -> Filter {
    if filter.limit.is_none() {
        return filter.limit(default_limit);
    }
    filter
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::time::Duration;

    use tempfile::TempDir;

    use super::*;

    const EVENTS: [&str; 14] = [
        r#"{"id":"b7b1fb52ad8461a03e949820ae29a9ea07e35bcd79c95c4b59b0254944f62805","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644581,"kind":1,"tags":[],"content":"Text note","sig":"ed73a8a4e7c26cd797a7b875c634d9ecb6958c57733305fed23b978109d0411d21b3e182cb67c8ad750884e30ca383b509382ae6187b36e76ee76e6a142c4284"}"#,
        r#"{"id":"7296747d91c53f1d71778ef3e12d18b66d494a41f688ef244d518abf37c959b6","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644586,"kind":32121,"tags":[["d","id-1"]],"content":"Empty 1","sig":"8848989a8e808f7315e950f871b231c1dff7752048f8957d4a541881d2005506c30e85c7dd74dab022b3e01329c88e69c9d5d55d961759272a738d150b7dbefc"}"#,
        r#"{"id":"ec6ea04ba483871062d79f78927df7979f67545b53f552e47626cb1105590442","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644591,"kind":32122,"tags":[["d","id-1"]],"content":"Empty 2","sig":"89946113a97484850fe35fefdb9120df847b305de1216dae566616fe453565e8707a4da7e68843b560fa22a932f81fc8db2b5a2acb4dcfd3caba9a91320aac92"}"#,
        r#"{"id":"63b8b829aa31a2de870c3a713541658fcc0187be93af2032ec2ca039befd3f70","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644596,"kind":32122,"tags":[["d","id-2"]],"content":"","sig":"607b1a67bef57e48d17df4e145718d10b9df51831d1272c149f2ab5ad4993ae723f10a81be2403ae21b2793c8ed4c129e8b031e8b240c6c90c9e6d32f62d26ff"}"#,
        r#"{"id":"6fe9119c7db13ae13e8ecfcdd2e5bf98e2940ba56a2ce0c3e8fba3d88cd8e69d","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644601,"kind":32122,"tags":[["d","id-3"]],"content":"","sig":"d07146547a726fc9b4ec8d67bbbe690347d43dadfe5d9890a428626d38c617c52e6945f2b7144c4e0c51d1e2b0be020614a5cadc9c0256b2e28069b70d9fc26e"}"#,
        r#"{"id":"a82f6ebfc709f4e7c7971e6bf738e30a3bc112cfdb21336054711e6779fd49ef","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644606,"kind":32122,"tags":[["d","id-1"]],"content":"","sig":"96d3349b42ed637712b4d07f037457ab6e9180d58857df77eb5fa27ff1fd68445c72122ec53870831ada8a4d9a0b484435f80d3ff21a862238da7a723a0d073c"}"#,
        r#"{"id":"8ab0cb1beceeb68f080ec11a3920b8cc491ecc7ec5250405e88691d733185832","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644611,"kind":32122,"tags":[["d","id-1"]],"content":"Test","sig":"49153b482d7110e2538eb48005f1149622247479b1c0057d902df931d5cea105869deeae908e4e3b903e3140632dc780b3f10344805eab77bb54fb79c4e4359d"}"#,
        r#"{"id":"63dc49a8f3278a2de8dc0138939de56d392b8eb7a18c627e4d78789e2b0b09f2","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644616,"kind":5,"tags":[["a","32122:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:"]],"content":"","sig":"977e54e5d57d1fbb83615d3a870037d9eb5182a679ca8357523bbf032580689cf481f76c88c7027034cfaf567ba9d9fe25fc8cd334139a0117ad5cf9fe325eef"}"#,
        r#"{"id":"6975ace0f3d66967f330d4758fbbf45517d41130e2639b54ca5142f37757c9eb","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644621,"kind":5,"tags":[["a","32122:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:id-2"]],"content":"","sig":"9bb09e4759899d86e447c3fa1be83905fe2eda74a5068a909965ac14fcdabaed64edaeb732154dab734ca41f2fc4d63687870e6f8e56e3d9e180e4a2dd6fb2d2"}"#,
        r#"{"id":"33f5b4e6a38e107638c20f4536db35191d4b8651ba5a2cefec983b9ec2d65084","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704645586,"kind":0,"tags":[],"content":"{\"name\":\"Key A\"}","sig":"285d090f45a6adcae717b33771149f7840a8c27fb29025d63f1ab8d95614034a54e9f4f29cee9527c4c93321a7ebff287387b7a19ba8e6f764512a40e7120429"}"#,
        r#"{"id":"90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704645606,"kind":0,"tags":[],"content":"{\"name\":\"key-a\",\"display_name\":\"Key A\",\"lud16\":\"keya@ln.address\"}","sig":"ec8f49d4c722b7ccae102d49befff08e62db775e5da43ef51b25c47dfdd6a09dc7519310a3a63cbdb6ec6b3250e6f19518eb47be604edeb598d16cdc071d3dbc"}"#,
        r#"{"id":"a295422c636d3532875b75739e8dae3cdb4dd2679c6e4994c9a39c7ebf8bc620","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704646569,"kind":5,"tags":[["e","90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9"]],"content":"","sig":"d4dc8368a4ad27eef63cacf667345aadd9617001537497108234fc1686d546c949cbb58e007a4d4b632c65ea135af4fbd7a089cc60ab89b6901f5c3fc6a47b29"}"#, // Invalid event deletion
        r#"{"id":"999e3e270100d7e1eaa98fcfab4a98274872c1f2dfdab024f32e42a5a12d5b5e","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704646606,"kind":5,"tags":[["e","90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9"]],"content":"","sig":"4f3a33fd52784cea7ca8428fd35d94d65049712e9aa11a70b1a16a1fcd761c7b7e27afac325728b1c00dfa11e33e78b2efd0430a7e4b28f4ede5b579b3f32614"}"#,
        r#"{"id":"99a022e6d61c4e39c147d08a2be943b664e8030c0049325555ac1766429c2832","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1705241093,"kind":30333,"tags":[["d","multi-id"],["p","aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4"]],"content":"Multi-tags","sig":"0abfb2b696a7ed7c9e8e3bf7743686190f3f1b3d4045b72833ab6187c254f7ed278d289d52dfac3de28be861c1471421d9b1bfb5877413cbc81c84f63207a826"}"#,
    ];

    struct TempDatabase {
        db: NostrSql,
        // Needed to avoid the drop and deletion of temp folder
        _temp: TempDir,
    }

    impl Deref for TempDatabase {
        type Target = NostrSql;

        fn deref(&self) -> &Self::Target {
            &self.db
        }
    }

    impl TempDatabase {
        async fn new() -> Self {
            let path = tempfile::tempdir().unwrap();
            let backend = NostrSqlBackend::sqlite(path.path().join("temp.db"));
            Self {
                db: NostrSql::new(backend).await.unwrap(),
                _temp: path,
            }
        }

        // Return the number of added events
        async fn add_random_events(&self) -> usize {
            let keys_a = Keys::generate();
            let keys_b = Keys::generate();

            let events = vec![
                EventBuilder::text_note("Text Note A")
                    .sign_with_keys(&keys_a)
                    .unwrap(),
                EventBuilder::text_note("Text Note B")
                    .sign_with_keys(&keys_b)
                    .unwrap(),
                EventBuilder::metadata(
                    &Metadata::new().name("account-a").display_name("Account A"),
                )
                    .sign_with_keys(&keys_a)
                    .unwrap(),
                EventBuilder::metadata(
                    &Metadata::new().name("account-b").display_name("Account B"),
                )
                    .sign_with_keys(&keys_b)
                    .unwrap(),
                EventBuilder::new(Kind::Custom(33_333), "")
                    .tag(Tag::identifier("my-id-a"))
                    .sign_with_keys(&keys_a)
                    .unwrap(),
                EventBuilder::new(Kind::Custom(33_333), "")
                    .tag(Tag::identifier("my-id-b"))
                    .sign_with_keys(&keys_b)
                    .unwrap(),
            ];

            // Store
            for event in events.iter() {
                self.db.save_event(event).await.unwrap();
            }

            events.len()
        }

        async fn add_event(&self, builder: EventBuilder) -> (Keys, Event) {
            let keys = Keys::generate();
            let event = builder.sign_with_keys(&keys).unwrap();
            self.db.save_event(&event).await.unwrap();
            (keys, event)
        }

        async fn add_event_with_keys(
            &self,
            builder: EventBuilder,
            keys: &Keys,
        ) -> (Event, SaveEventStatus) {
            let event = builder.sign_with_keys(keys).unwrap();
            let status = self.db.save_event(&event).await.unwrap();
            (event, status)
        }

        async fn count_all(&self) -> usize {
            self.db.count(Filter::new()).await.unwrap()
        }
    }

    #[tokio::test]
    async fn test_event_by_id() {
        let db = TempDatabase::new().await;

        let added_events: usize = db.add_random_events().await;

        let (_keys, expected_event) = db.add_event(EventBuilder::text_note("Test")).await;

        let event = db.event_by_id(&expected_event.id).await.unwrap().unwrap();
        assert_eq!(event, expected_event);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1)
    }

    #[tokio::test]
    async fn test_replaceable_event() {
        let db = TempDatabase::new().await;

        let added_events: usize = db.add_random_events().await;

        let now = Timestamp::now();
        let metadata = Metadata::new()
            .name("my-account")
            .display_name("My Account");

        let (keys, expected_event) = db
            .add_event(
                EventBuilder::metadata(&metadata).custom_created_at(now - Duration::from_secs(120)),
            )
            .await;

        // Test event by ID
        let event = db.event_by_id(&expected_event.id).await.unwrap().unwrap();
        assert_eq!(event, expected_event);

        // Test filter query
        let events = db
            .query(Filter::new().author(keys.public_key).kind(Kind::Metadata))
            .await
            .unwrap();
        assert_eq!(events.to_vec(), vec![expected_event.clone()]);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1);

        // Replace previous event
        let (new_expected_event, status) = db
            .add_event_with_keys(
                EventBuilder::metadata(&metadata).custom_created_at(now),
                &keys,
            )
            .await;
        assert!(status.is_success());

        // Test event by ID (MUST be None because replaced)
        assert!(db.event_by_id(&expected_event.id).await.unwrap().is_none());

        // Test event by ID
        let event = db
            .event_by_id(&new_expected_event.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event, new_expected_event);

        // Test filter query
        let events = db
            .query(Filter::new().author(keys.public_key).kind(Kind::Metadata))
            .await
            .unwrap();
        assert_eq!(events.to_vec(), vec![new_expected_event]);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1);
    }

    #[tokio::test]
    async fn test_param_replaceable_event() {
        let db = TempDatabase::new().await;

        let added_events: usize = db.add_random_events().await;

        let now = Timestamp::now();

        let (keys, expected_event) = db
            .add_event(
                EventBuilder::new(Kind::Custom(33_333), "")
                    .tag(Tag::identifier("my-id-a"))
                    .custom_created_at(now - Duration::from_secs(120)),
            )
            .await;
        let coordinate = Coordinate::new(Kind::from(33_333), keys.public_key).identifier("my-id-a");

        // Test event by ID
        let event = db.event_by_id(&expected_event.id).await.unwrap().unwrap();
        assert_eq!(event, expected_event);

        // Test filter query
        let events = db.query(coordinate.clone().into()).await.unwrap();
        assert_eq!(events.to_vec(), vec![expected_event.clone()]);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1);

        // Replace previous event
        let (new_expected_event, status) = db
            .add_event_with_keys(
                EventBuilder::new(Kind::Custom(33_333), "Test replace")
                    .tag(Tag::identifier("my-id-a"))
                    .custom_created_at(now),
                &keys,
            )
            .await;
        assert!(status.is_success());

        // Test event by ID (MUST be None` because replaced)
        assert!(db.event_by_id(&expected_event.id).await.unwrap().is_none());

        // Test event by ID
        let event = db
            .event_by_id(&new_expected_event.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event, new_expected_event);

        // Test filter query
        let events = db.query(coordinate.into()).await.unwrap();
        assert_eq!(events.to_vec(), vec![new_expected_event]);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1);

        // Trey to add param replaceable event with older timestamp (MUSTN'T be stored)
        let (_, status) = db
            .add_event_with_keys(
                EventBuilder::new(Kind::Custom(33_333), "Test replace 2")
                    .tag(Tag::identifier("my-id-a"))
                    .custom_created_at(now - Duration::from_secs(2000)),
                &keys,
            )
            .await;
        assert!(!status.is_success());
    }

    #[tokio::test]
    async fn test_full_text_search() {
        let db = TempDatabase::new().await;

        let _added_events: usize = db.add_random_events().await;

        let events = db.query(Filter::new().search("Account A")).await.unwrap();
        assert_eq!(events.len(), 1);

        let events = db.query(Filter::new().search("account a")).await.unwrap();
        assert_eq!(events.len(), 1);

        let events = db.query(Filter::new().search("text note")).await.unwrap();
        assert_eq!(events.len(), 2);

        let events = db.query(Filter::new().search("notes")).await.unwrap();
        assert_eq!(events.len(), 0);

        let events = db.query(Filter::new().search("hola")).await.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_expected_query_result() {
        let db = TempDatabase::new().await;

        for event in EVENTS.into_iter() {
            let event = Event::from_json(event).unwrap();
            let _ = db.save_event(&event).await;
        }

        // Test expected output
        let expected_output = vec![
            Event::from_json(EVENTS[13]).unwrap(),
            Event::from_json(EVENTS[12]).unwrap(),
            // Event 11 is invalid deletion
            // Event 10 deleted by event 12
            // Event 9 replaced by event 10
            Event::from_json(EVENTS[8]).unwrap(),
            // Event 7 is an invalid deletion
            Event::from_json(EVENTS[6]).unwrap(),
            Event::from_json(EVENTS[5]).unwrap(),
            Event::from_json(EVENTS[4]).unwrap(),
            // Event 3 deleted by Event 8
            // Event 2 replaced by Event 6
            Event::from_json(EVENTS[1]).unwrap(),
            Event::from_json(EVENTS[0]).unwrap(),
        ];
        assert_eq!(
            db.query(Filter::new()).await.unwrap().to_vec(),
            expected_output
        );
        assert_eq!(db.count_all().await, 8);
    }

    #[tokio::test]
    async fn test_delete_events_with_filter() {
        let db = TempDatabase::new().await;

        let added_events: usize = db.add_random_events().await;

        assert_eq!(db.count_all().await, added_events);

        // Delete all kinds except text note
        let filter = Filter::new().kinds([Kind::Metadata, Kind::Custom(33_333)]);
        db.delete(filter).await.unwrap();

        assert_eq!(db.count_all().await, 2);
    }
}
