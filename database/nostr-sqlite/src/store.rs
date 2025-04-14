//! Nostr SQLite database

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use nostr_database::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{QueryBuilder, Sqlite, Transaction};
use sqlx::migrate::Migrator;
use tokio::sync::Mutex;

use crate::error::Error;
use crate::model::{EventDataDb, EventDb, EventPayloadAndDeletionStatus, EventTagDb};

const EVENTS_QUERY_LIMIT: usize = 10_000;

/// Nostr SQLite database
#[derive(Clone)]
pub struct NostrSqlite {
    pool: SqlitePool,
    fbb: Arc<Mutex<FlatBufferBuilder<'static>>>,
}

impl fmt::Debug for NostrSqlite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NostrSqlite")
            .field("pool", &self.pool)
            .finish()
    }
}

impl NostrSqlite {
    /// Connect to a SQL database
    pub async fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let opts: SqliteConnectOptions =
            SqliteConnectOptions::new().create_if_missing(true).filename(path);

        let pool: SqlitePool = SqlitePool::connect_with(opts).await?;

        // Run migrations
        let migrator: Migrator = sqlx::migrate!();
        migrator.run(&pool).await?;

        Ok(Self {
            pool,
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::new())),
        })
    }

    /// Returns true if successfully inserted
    async fn insert_event_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        event: &EventDb,
    ) -> Result<bool, Error> {
        let result = sqlx::query("INSERT OR IGNORE INTO events (id, pubkey, created_at, kind, payload) VALUES ($1, $2, $3, $4, $5)")
            .bind(&event.id)
            .bind(&event.pubkey)
            .bind(event.created_at)
            .bind(event.kind)
            .bind(&event.payload)
            .execute(&mut **tx)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn insert_tags_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        tags: &[EventTagDb],
    ) -> Result<(), Error> {
        for tag in tags.iter() {
            sqlx::query("INSERT OR IGNORE INTO event_tags (tag, tag_value, event_id) VALUES ($1, $2, $3)")
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

    async fn get_event_by_id(&self, id: &EventId) -> Result<Option<EventPayloadAndDeletionStatus>, Error> {
        let event: Option<EventPayloadAndDeletionStatus> = sqlx::query_as("SELECT payload, deleted FROM events WHERE id = $1")
            .bind(id.as_bytes().as_slice())
            .fetch_optional(&self.pool).await?;
        Ok(event)
    }
}

impl NostrDatabase for NostrSqlite {
    fn backend(&self) -> Backend {
        Backend::SQLite
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
            match self
                .get_event_by_id(event_id)
                .await
                .map_err(DatabaseError::backend)?
            {
                Some(e) if !e.deleted => Ok(Some(
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

            let mut sql = build_filter_query(&filter);

            let payloads: Vec<(Vec<u8>,)> = sql.build_query_as()
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

fn build_filter_query(filter: &Filter) -> QueryBuilder<Sqlite> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT DISTINCT e.payload
         FROM events e
         INNER JOIN event_tags et ON e.id = et.event_id
         WHERE e.deleted = 0",
    );

    // Add filters
    if let Some(ids) = &filter.ids {
        if !ids.is_empty() {
            query_builder.push(" AND e.id IN (");
            let mut separated = query_builder.separated(", ");
            for id in ids.iter() {
                separated.push_bind(id.as_bytes().as_slice());
            }
            query_builder.push(")");
        }
    }

    if let Some(authors) = &filter.authors {
        if !authors.is_empty() {
            query_builder.push(" AND e.pubkey IN (");
            let mut separated = query_builder.separated(", ");
            for author in authors.iter() {
                separated.push_bind(author.as_bytes().as_slice());
            }
            query_builder.push(")");
        }
    }

    if let Some(kinds) = &filter.kinds {
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
        for (tag, values) in &filter.generic_tags {
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

    query_builder
}

/// sets the given default limit on a Nostr filter if not set
fn with_limit(filter: Filter, default_limit: usize) -> Filter {
    match filter.limit {
        Some(..) => filter,
        None => filter.limit(default_limit),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use nostr_database_test_suite::database_unit_tests;

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
