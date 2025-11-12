// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr SQL

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use nostr_database::prelude::*;
use sqlx::migrate::Migrator;
#[cfg(feature = "mysql")]
use sqlx::mysql::MySqlConnectOptions;
#[cfg(feature = "postgres")]
use sqlx::postgres::PgConnectOptions;
#[cfg(feature = "sqlite")]
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{
    Any, AnyConnection, AnyPool, ConnectOptions, Database, Pool, QueryBuilder, Sqlite, Transaction,
    Type,
};
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
        path: Option<PathBuf>,
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
    },
}

impl NostrSqlBackend {
    /// New persistent SQLite database
    #[inline]
    #[cfg(feature = "sqlite")]
    pub fn sqlite<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self::Sqlite {
            path: Some(path.as_ref().to_path_buf()),
        }
    }

    /// New in-memory SQLite database
    #[inline]
    #[cfg(feature = "sqlite")]
    pub fn sqlite_memory() -> Self {
        Self::Sqlite { path: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PoolKind {
    #[cfg(feature = "sqlite")]
    Sqlite,
    #[cfg(feature = "postgres")]
    Postgres,
    #[cfg(feature = "mysql")]
    MySql,
}

/// Nostr SQL database
#[derive(Clone)]
pub struct NostrSql {
    pool: AnyPool,
    kind: PoolKind,
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
        // Install drivers
        sqlx::any::install_default_drivers();

        let (uri, kind) = match backend {
            #[cfg(feature = "sqlite")]
            NostrSqlBackend::Sqlite { path } => {
                let mut opts: SqliteConnectOptions =
                    SqliteConnectOptions::new().create_if_missing(true);

                match path {
                    Some(path) => opts = opts.filename(path),
                    None => opts = opts.in_memory(true),
                };

                (opts.to_url_lossy(), PoolKind::Sqlite)
            }
            #[cfg(feature = "postgres")]
            NostrSqlBackend::Postgres {
                host,
                port,
                username,
                password,
                database,
            } => {
                let mut opts: PgConnectOptions = PgConnectOptions::new_without_pgpass()
                    .host(&host)
                    .port(port)
                    .database(&database);

                if let Some(username) = username {
                    opts = opts.username(&username);
                }

                if let Some(password) = password {
                    opts = opts.password(&password);
                }

                (opts.to_url_lossy(), PoolKind::Postgres)
            }
        };

        let pool: AnyPool = AnyPool::connect(uri.as_str()).await?;

        let migrator: Migrator = match kind {
            #[cfg(feature = "sqlite")]
            PoolKind::Sqlite => sqlx::migrate!("migrations/sqlite"),
            #[cfg(feature = "postgres")]
            PoolKind::Postgres => sqlx::migrate!("migrations/postgres"),
            #[cfg(feature = "mysql")]
            PoolKind::MySql => sqlx::migrate!("migrations/mysql"),
        };
        migrator.run(&pool).await?;

        Ok(Self {
            pool,
            kind,
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::new())),
        })
    }

    /// Returns true if successfully inserted
    async fn insert_event_tx(
        &self,
        tx: &mut Transaction<'_, Any>,
        event: &EventDb,
    ) -> Result<bool, Error> {
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

    async fn insert_tags_tx(
        &self,
        tx: &mut Transaction<'_, Any>,
        tags: &[EventTagDb],
    ) -> Result<(), Error> {
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
        let event: Option<EventDb> = sqlx::query_as(
            "SELECT id, pubkey, created_at, kind, payload, deleted FROM events WHERE id = ?",
        )
        .bind(id.as_bytes().to_vec())
        .fetch_optional(&self.pool)
        .await?;
        Ok(event)
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
            self._save_event(event)
                .await
                .map_err(DatabaseError::backend)
        })
    }

    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        Box::pin(async move {
            match self
                .get_event_by_id(event_id)
                .await
                .map_err(DatabaseError::backend)?
            {
                Some(e) if e.is_deleted() => Ok(DatabaseEventStatus::Deleted),
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
            match self
                .get_event_by_id(event_id)
                .await
                .map_err(DatabaseError::backend)?
            {
                Some(e) if !e.is_deleted() => Ok(Some(
                    Event::decode(&e.payload).map_err(DatabaseError::backend)?,
                )),
                _ => Ok(None),
            }
        })
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move { Ok(self.query(filter).await?.len()) })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        Box::pin(async move {
            // Limit filter query
            let filter: Filter = with_limit(filter, EVENTS_QUERY_LIMIT);

            let mut events: Events = Events::new(&filter);

            let sql = build_filter_query(filter);

            let payloads: Vec<(Vec<u8>,)> = sqlx::query_as(&sql)
                .fetch_all(&self.pool)
                .await
                .map_err(DatabaseError::backend)?;

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
        Box::pin(async move { Err(DatabaseError::NotSupported) })
    }

    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { Err(DatabaseError::NotSupported) })
    }
}

fn build_filter_query(filter: Filter) -> String {
    let mut query_builder: QueryBuilder<Any> = QueryBuilder::new(
        "SELECT DISTINCT e.payload
         FROM events e
         INNER JOIN event_tags et ON e.id = et.event_id
         WHERE e.deleted = 0",
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
        query_builder.push_bind(since.as_secs() as i64);
    }

    if let Some(until) = filter.until {
        query_builder.push(" AND e.created_at <= ");
        query_builder.push_bind(until.as_secs() as i64);
    }

    if !filter.generic_tags.is_empty() {
        for (tag, values) in filter.generic_tags {
            if !values.is_empty() {
                query_builder.push(
                    " AND EXISTS (
                    SELECT 1 FROM event_tags et2
                    WHERE et2.event_id = e.id
                    AND et2.tag = ",
                );
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
    use nostr_database_test_suite::database_unit_tests;
    use tempfile::TempDir;

    use super::*;

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
            let backend = NostrSqlBackend::sqlite(path.path().join("test.db"));
            Self {
                db: NostrSql::new(backend).await.unwrap(),
                _temp: path,
            }
        }
    }

    database_unit_tests!(TempDatabase, TempDatabase::new);
}
