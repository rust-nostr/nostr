use std::path::Path;
use std::sync::Arc;

use sqlx::{Sqlite, SqlitePool};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use tokio::sync::Mutex;
use nostr::{Event, EventId, Filter};
use nostr::prelude::BoxedFuture;
use nostr_database::{Backend, DatabaseError, DatabaseEventStatus, Events, FlatBufferBuilder, NostrDatabase, SaveEventStatus};

use crate::db::NostrSql;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct NostrSqlite {
    db: NostrSql<Sqlite>
}

impl NostrSqlite {
    /// Open SQLite database
    pub async fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        // Build SQLite connection options
        let opts: SqliteConnectOptions =
            SqliteConnectOptions::new().create_if_missing(true).filename(path);

        // Connect to SQLite database
        let pool: SqlitePool = SqlitePool::connect_with(opts).await?;

        // Run migrations
        let migrator: Migrator = sqlx::migrate!("migrations/sqlite");
        migrator.run(&pool).await?;

        Ok(Self {
            db: NostrSql::new(pool),
        })
    }
}

impl NostrDatabase for NostrSqlite {
    fn backend(&self) -> Backend {
        self.db.backend()
    }

    fn save_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>> {
        self.db.save_event(event)
    }

    fn check_id<'a>(&'a self, event_id: &'a EventId) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        self.db.check_id(event_id)
    }

    fn event_by_id<'a>(&'a self, event_id: &'a EventId) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        self.db.event_by_id(event_id)
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        self.db.count(filter)
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        self.db.query(filter)
    }

    fn delete(&self, filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        self.db.delete(filter)
    }

    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        self.db.wipe()
    }
}

#[cfg(test)]
mod tests {
    use nostr_database_test_suite::database_unit_tests;
    use tempfile::TempDir;

    use super::*;

    struct TempDatabase {
        db: NostrSqlite,
        // Needed to avoid the drop and deletion of temp folder
        _temp: TempDir,
    }

    impl Deref for TempDatabase {
        type Target = NostrSqlite;

        fn deref(&self) -> &Self::Target {
            &self.db
        }
    }

    impl TempDatabase {
        async fn new() -> Self {
            let path = tempfile::tempdir().unwrap();
            Self {
                db: NostrSqlite::open(path.path().join("temp.db")).await.unwrap(),
                _temp: path,
            }
        }
    }

    database_unit_tests!(TempDatabase, TempDatabase::new);
}
