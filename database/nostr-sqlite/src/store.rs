//! Nostr SQLite database

use std::cmp::Ordering;
use std::fmt;
use std::path::Path;
use std::time::Duration;

use nostr_database::prelude::*;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use sqlx::types::Json;
use sqlx::{Executor, QueryBuilder, Sqlite, Transaction};

use crate::error::Error;
use crate::model::{extract_tags, EventDb};

const EVENTS_QUERY_LIMIT: usize = 10_000;

#[derive(Clone, Copy)]
enum SqlSelectClause {
    Select,
    Count,
    Delete,
}

/// Nostr SQLite database
#[derive(Clone)]
pub struct NostrSqlite {
    pool: SqlitePool,
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
        let opts: SqliteConnectOptions = SqliteConnectOptions::new()
            .busy_timeout(Duration::from_secs(60))
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true)
            .filename(path);

        let pool: SqlitePool = SqlitePool::connect_with(opts).await?;

        // Run migrations
        let migrator: Migrator = sqlx::migrate!();
        migrator.run(&pool).await?;

        Ok(Self { pool })
    }

    /// Returns true if successfully inserted
    async fn insert_event_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        event: &Event,
    ) -> Result<bool, Error> {
        let result = sqlx::query("INSERT OR IGNORE INTO events (id, pubkey, created_at, kind, content, tags, sig) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(event.id.as_bytes().as_slice())
            .bind(event.pubkey.as_bytes().as_slice())
            .bind(event.created_at.as_secs() as i64)
            .bind(event.kind.as_u16())
            .bind(&event.content)
            .bind(Json(&event.tags))
            .bind(event.sig.as_ref().as_slice())
            .execute(&mut **tx)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn handle_deletion_event(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        event: &Event,
    ) -> Result<bool, Error> {
        for id in event.tags.event_ids() {
            if let Some(pubkey) = self.get_pubkey_of_event_by_id(tx, id).await? {
                // Author must match
                if pubkey != event.pubkey {
                    return Ok(true);
                }

                // Mark the event ID as deleted (for NIP-09 deletion events)
                self.mark_event_as_deleted(tx, id).await?;

                // Remove event from store
                self.remove_event(tx, id).await?;
            }
        }

        for coordinate in event.tags.coordinates() {
            // Author must match
            if coordinate.public_key != event.pubkey {
                return Ok(true);
            }

            // Mark deleted
            self.mark_coordinate_deleted(tx, &coordinate.borrow(), event.created_at)
                .await?;

            // Remove events (up to the created_at of the deletion event)
            if coordinate.kind.is_replaceable() {
                self.remove_replaceable(tx, coordinate, &event.created_at)
                    .await?;
            } else if coordinate.kind.is_addressable() {
                self.remove_addressable(tx, coordinate, event.created_at)
                    .await?;
            }
        }

        Ok(false)
    }

    async fn _save_event(&self, event: &Event) -> Result<SaveEventStatus, Error> {
        if event.kind.is_ephemeral() {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Ephemeral));
        }

        let mut tx = self.pool.begin().await?;

        // Already exists
        if self.has_event(&mut *tx, &event.id).await? {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Duplicate));
        }

        // Reject event if ID was deleted
        if self.event_is_deleted(&mut *tx, &event.id).await? {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
        }

        // Reject event if the public key was vanished
        if self.pubkey_is_vanished(&mut tx, &event.pubkey).await? {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Vanished));
        }

        // Reject event if ADDR was deleted after it's created_at date
        // (non-parameterized or parameterized)
        if let Some(coordinate) = event.coordinate() {
            let timestamp: Option<Timestamp> = self
                .when_is_coordinate_deleted(&mut tx, &coordinate)
                .await?;

            if let Some(time) = timestamp {
                if event.created_at <= time {
                    return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
                }
            }
        }

        // Remove replaceable events being replaced
        if event.kind.is_replaceable() {
            // Find existing replaceable event
            let existing: Option<EventDb> =
                sqlx::query_as("SELECT * FROM events WHERE pubkey = $1 AND kind = $2 LIMIT 1")
                    .bind(event.pubkey.as_bytes().as_slice())
                    .bind(event.kind.as_u16())
                    .fetch_optional(&mut *tx)
                    .await?;

            if let Some(stored) = existing {
                // Check if new event should replace stored
                if has_event_been_replaced(&stored, event) {
                    // New event is older or same timestamp with higher ID - reject it
                    return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                }

                // Delete the old event (CASCADE will delete tags too)
                sqlx::query("DELETE FROM events WHERE id = $1")
                    .bind(stored.id.as_slice())
                    .execute(&mut *tx)
                    .await?;
            }
        }

        // Remove addressable events being replaced
        if event.kind.is_addressable() {
            if let Some(identifier) = event.tags.identifier() {
                let existing: Option<EventDb> = sqlx::query_as(
                    "SELECT e.* FROM events e
                 INNER JOIN event_tags t ON e.id = t.event_id
                 WHERE e.pubkey = ?1 AND e.kind = ?2
                 AND t.tag_name = 'd' AND t.tag_value = ?3
                 LIMIT 1",
                )
                .bind(event.pubkey.as_bytes().as_slice())
                .bind(event.kind.as_u16())
                .bind(identifier)
                .fetch_optional(&mut *tx)
                .await?;

                if let Some(stored) = existing {
                    if has_event_been_replaced(&stored, event) {
                        return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                    }

                    // Delete the old addressable event
                    sqlx::query("DELETE FROM events WHERE id = $1")
                        .bind(stored.id.as_slice())
                        .execute(&mut *tx)
                        .await?;
                }
            }
        }

        // Handle deletion events
        if event.kind == Kind::EventDeletion {
            let invalid: bool = self.handle_deletion_event(&mut tx, event).await?;

            if invalid {
                tx.rollback().await?;
                return Ok(SaveEventStatus::Rejected(RejectedReason::InvalidDelete));
            }
        }

        if event.kind == Kind::RequestToVanish {
            // For now, handling `ALL_RELAYS` only
            if let Some(TagStandard::AllRelays) = event.tags.find_standardized(TagKind::Relay) {
                self.handle_request_to_vanish(&mut tx, &event.pubkey)
                    .await?;
            }
        }

        // Insert event first
        let inserted: bool = self.insert_event_tx(&mut tx, event).await?;

        // Check if the event has been inserted
        if inserted {
            // Insert tags
            for tag in extract_tags(event) {
                sqlx::query("INSERT OR IGNORE INTO event_tags(event_id, tag_name, tag_value) VALUES ($1, $2, $3)")
                    .bind(tag.event_id)
                    .bind(tag.tag_name.as_str())
                    .bind(tag.tag_value)
                    .execute(&mut *tx)
                    .await?;
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

    async fn mark_event_as_deleted(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        id: &EventId,
    ) -> Result<(), Error> {
        sqlx::query("INSERT OR IGNORE INTO deleted_ids(event_id) VALUES ($1)")
            .bind(id.as_bytes().as_slice())
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn mark_coordinate_deleted(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        coordinate: &CoordinateBorrow<'_>,
        deleted_at: Timestamp,
    ) -> Result<(), Error> {
        sqlx::query("INSERT OR IGNORE INTO deleted_coordinates(pubkey, kind, identifier, deleted_at) VALUES ($1, $2, $3, $4)")
            .bind(coordinate.public_key.as_bytes().as_slice())
            .bind(coordinate.kind.as_u16())
            .bind(coordinate.identifier.unwrap_or_default())
            .bind(deleted_at.as_secs() as i64)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn event_is_deleted<'a, E>(&self, executor: E, id: &EventId) -> Result<bool, Error>
    where
        E: Executor<'a, Database = Sqlite>,
    {
        let is_deleted: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM deleted_ids WHERE event_id = $1)")
                .bind(id.as_bytes().as_slice())
                .fetch_one(executor)
                .await?;
        Ok(is_deleted)
    }

    async fn when_is_coordinate_deleted(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        coordinate: &CoordinateBorrow<'_>,
    ) -> Result<Option<Timestamp>, Error> {
        let timestamp: Option<(i64,)> = sqlx::query_as("SELECT deleted_at FROM deleted_coordinates WHERE pubkey = $1 AND kind = $2 AND identifier = $3")
            .bind(coordinate.public_key.as_bytes().as_slice())
            .bind(coordinate.kind.as_u16())
            .bind(coordinate.identifier.unwrap_or_default())
            .fetch_optional(&mut **tx)
            .await?;

        match timestamp {
            Some((timestamp,)) => Ok(Some(timestamp.try_into()?)),
            None => Ok(None),
        }
    }

    async fn pubkey_is_vanished(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        pubkey: &PublicKey,
    ) -> Result<bool, Error> {
        let is_vanished: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM vanished_public_keys WHERE pubkey = $1)",
        )
        .bind(pubkey.as_bytes().as_slice())
        .fetch_one(&mut **tx)
        .await?;
        Ok(is_vanished)
    }

    async fn mark_pubkey_as_vanished(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        pubkey: &PublicKey,
    ) -> Result<(), Error> {
        sqlx::query("INSERT OR IGNORE INTO vanished_public_keys(pubkey) VALUES ($1)")
            .bind(pubkey.as_bytes().as_slice())
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn handle_request_to_vanish(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        pubkey: &PublicKey,
    ) -> Result<(), Error> {
        self.mark_pubkey_as_vanished(tx, pubkey).await?;

        // Delete all user events
        sqlx::query("DELETE FROM events where pubkey = $1")
            .bind(pubkey.as_bytes().as_slice())
            .execute(&mut **tx)
            .await?;

        // Delete all gift wraps that mention the public key
        sqlx::query(
            r#"
        DELETE FROM events
        WHERE id IN (
            SELECT e.id
            FROM events AS e
            INNER JOIN event_tags AS et
                ON e.id = et.event_id
            WHERE
                e.kind = 1059 AND
                et.tag_name = 'p' AND
                et.tag_value = $1
        )
        "#,
        )
        .bind(pubkey.to_hex())
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn has_event<'a, E>(&self, executor: E, id: &EventId) -> Result<bool, Error>
    where
        E: Executor<'a, Database = Sqlite>,
    {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
            .bind(id.as_bytes().as_slice())
            .fetch_one(executor)
            .await?;
        Ok(exists)
    }

    async fn get_pubkey_of_event_by_id(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        id: &EventId,
    ) -> Result<Option<PublicKey>, Error> {
        let pubkey: Option<(Vec<u8>,)> = sqlx::query_as("SELECT pubkey FROM events WHERE id = $1")
            .bind(id.as_bytes().as_slice())
            .fetch_optional(&mut **tx)
            .await?;
        match pubkey {
            Some((pk,)) => Ok(Some(PublicKey::from_slice(&pk)?)),
            None => Ok(None),
        }
    }

    async fn get_event_by_id(&self, id: &EventId) -> Result<Option<Event>, Error> {
        let event: Option<EventDb> = sqlx::query_as("SELECT * FROM events WHERE id = $1")
            .bind(id.as_bytes().as_slice())
            .fetch_optional(&self.pool)
            .await?;
        match event {
            Some(event) => Ok(Some(event.to_event()?)),
            None => Ok(None),
        }
    }

    async fn remove_event(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        id: &EventId,
    ) -> Result<(), Error> {
        sqlx::query("DELETE FROM events where id = $1")
            .bind(id.as_bytes().as_slice())
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    /// Remove all replaceable events with the matching author-kind
    /// Kind must be a replaceable (not parameterized replaceable) event kind
    async fn remove_replaceable(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        coordinate: &Coordinate,
        until: &Timestamp,
    ) -> Result<(), Error> {
        sqlx::query(
            "DELETE FROM events
         WHERE pubkey = $1 AND kind = $2 AND created_at <= $3",
        )
        .bind(coordinate.public_key.as_bytes().as_slice())
        .bind(coordinate.kind.as_u16())
        .bind(until.as_secs() as i64)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Remove all parameterized-replaceable events with the matching author-kind-identifier
    /// Kind must be a parameterized-replaceable (addressable) event kind
    async fn remove_addressable(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        coordinate: &Coordinate,
        until: Timestamp,
    ) -> Result<(), Error> {
        sqlx::query(
            "DELETE FROM events
         WHERE id IN (
             SELECT e.id FROM events e
             INNER JOIN event_tags t ON e.id = t.event_id
             WHERE e.pubkey = $1 AND e.kind = $2
             AND t.tag_name = 'd' AND t.tag_value = $3
             AND e.created_at <= $4
         )",
        )
        .bind(coordinate.public_key.as_bytes().as_slice())
        .bind(coordinate.kind.as_u16())
        .bind(&coordinate.identifier)
        .bind(until.as_secs() as i64)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

impl NostrDatabase for NostrSqlite {
    fn backend(&self) -> Backend {
        Backend::SQLite
    }

    fn features(&self) -> Features {
        Features {
            persistent: true,
            event_expiration: false,
            full_text_search: true,
            request_to_vanish: true,
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
            if self
                .event_is_deleted(&self.pool, event_id)
                .await
                .map_err(DatabaseError::backend)?
            {
                Ok(DatabaseEventStatus::Deleted)
            } else if self
                .has_event(&self.pool, event_id)
                .await
                .map_err(DatabaseError::backend)?
            {
                Ok(DatabaseEventStatus::Saved)
            } else {
                Ok(DatabaseEventStatus::NotExistent)
            }
        })
    }

    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            self.get_event_by_id(event_id)
                .await
                .map_err(DatabaseError::backend)
        })
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move {
            // Limit filter query
            let filter: Filter = with_limit(filter, EVENTS_QUERY_LIMIT);

            let mut sql: QueryBuilder<Sqlite> = build_filter(&filter, SqlSelectClause::Count);

            let count: (i64,) = sql
                .build_query_as()
                .fetch_one(&self.pool)
                .await
                .map_err(DatabaseError::backend)?;

            Ok(count.0 as usize)
        })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        Box::pin(async move {
            // Limit filter query
            let filter: Filter = with_limit(filter, EVENTS_QUERY_LIMIT);

            let mut events: Events = Events::new(&filter);

            let mut sql: QueryBuilder<Sqlite> = build_filter(&filter, SqlSelectClause::Select);

            let row_events: Vec<EventDb> = sql
                .build_query_as()
                .fetch_all(&self.pool)
                .await
                .map_err(DatabaseError::backend)?;

            for event in row_events.into_iter() {
                if let Ok(event) = event.to_event() {
                    events.insert(event);
                }
            }

            Ok(events)
        })
    }

    // TODO: impl negentropy_items deserializing only ids and timestamps

    fn delete(&self, filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move {
            let mut sql: QueryBuilder<Sqlite> = build_filter(&filter, SqlSelectClause::Delete);

            sql.build()
                .execute(&self.pool)
                .await
                .map_err(DatabaseError::backend)?;

            Ok(())
        })
    }

    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move {
            // Delete all data (CASCADE will handle event_tags)
            sqlx::query("DELETE FROM events")
                .execute(&self.pool)
                .await
                .map_err(DatabaseError::backend)?;

            sqlx::query("DELETE FROM deleted_ids")
                .execute(&self.pool)
                .await
                .map_err(DatabaseError::backend)?;

            sqlx::query("DELETE FROM deleted_coordinates")
                .execute(&self.pool)
                .await
                .map_err(DatabaseError::backend)?;

            // Vacuum to reclaim space
            sqlx::query("VACUUM")
                .execute(&self.pool)
                .await
                .map_err(DatabaseError::backend)?;

            Ok(())
        })
    }
}

fn has_event_been_replaced(stored: &EventDb, event: &Event) -> bool {
    match stored.created_at.cmp(&(event.created_at.as_secs() as i64)) {
        Ordering::Greater => true, // Stored is newer, reject incoming event
        Ordering::Equal => {
            // NIP-01: When timestamps are identical, keep the event with the LOWEST ID
            // Return true if stored.id < event.id (stored should be kept, reject incoming)
            // Return false if event.id < stored.id (incoming should replace stored)
            stored.id.as_slice() < event.id.as_bytes().as_slice()
        }
        Ordering::Less => false, // Stored is older, accept incoming event (will replace stored)
    }
}

fn build_filter(filter: &Filter, select_clause: SqlSelectClause) -> QueryBuilder<Sqlite> {
    // If no filters, simple query without JOIN
    if filter.is_empty() {
        let mut query_builder = QueryBuilder::new(match select_clause {
            SqlSelectClause::Select => "SELECT * FROM events",
            SqlSelectClause::Count => "SELECT COUNT(*) FROM events",
            SqlSelectClause::Delete => "DELETE FROM events",
        });

        if let SqlSelectClause::Select | SqlSelectClause::Count = select_clause {
            query_builder.push(" ORDER BY created_at DESC");
            if let Some(limit) = filter.limit {
                query_builder.push(" LIMIT ");
                query_builder.push_bind(limit as i64);
            }
        }

        return query_builder;
    }

    let mut query_builder: QueryBuilder<Sqlite> = match select_clause {
        SqlSelectClause::Select => QueryBuilder::new("SELECT DISTINCT e.*"),
        SqlSelectClause::Count => QueryBuilder::new("SELECT COUNT(DISTINCT e.id)"),
        // For DELETE, we need to use a subquery because SQLite doesn't support DELETE with JOIN directly
        SqlSelectClause::Delete => {
            let mut query_builder = QueryBuilder::new(
                "DELETE FROM events WHERE id IN (SELECT DISTINCT e.id FROM events e",
            );

            // Only JOIN if we have tag filters
            if !filter.generic_tags.is_empty() {
                query_builder.push(" INNER JOIN event_tags et ON e.id = et.event_id");
            }

            query_builder.push(" WHERE 1=1");

            // Add all the filter conditions
            add_filter_conditions(filter, &mut query_builder);

            query_builder.push(")"); // Close the subquery

            return query_builder;
        }
    };

    query_builder.push(" FROM events e");

    // Only JOIN if we have tag filters
    if !filter.generic_tags.is_empty() {
        query_builder.push(" INNER JOIN event_tags et ON e.id = et.event_id");
    }

    query_builder.push(" WHERE 1=1");

    // Add all the filter conditions
    add_filter_conditions(filter, &mut query_builder);

    // Only add ORDER BY and LIMIT for SELECT queries
    query_builder.push(" ORDER BY e.created_at DESC");

    if let Some(limit) = filter.limit {
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit as i64);
    }

    query_builder
}

// Extract filter conditions to avoid duplication
fn add_filter_conditions<'a>(filter: &'a Filter, query_builder: &mut QueryBuilder<'a, Sqlite>) {
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

    // Search filter (if exists)
    if let Some(search) = &filter.search {
        query_builder.push(" AND (e.content LIKE ");
        query_builder.push_bind(format!("%{}%", search));
        query_builder.push(" OR EXISTS (SELECT 1 FROM event_tags et_search WHERE et_search.event_id = e.id AND et_search.tag_value LIKE ");
        query_builder.push_bind(format!("%{}%", search));
        query_builder.push("))");
    }

    if !filter.generic_tags.is_empty() {
        for (tag, values) in &filter.generic_tags {
            if !values.is_empty() {
                query_builder.push(
                    " AND EXISTS (
                    SELECT 1 FROM event_tags et2
                    WHERE et2.event_id = e.id
                    AND et2.tag_name = ",
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
                db: NostrSqlite::open(path.path().join("temp.db"))
                    .await
                    .unwrap(),
                _temp: path,
            }
        }
    }

    database_unit_tests!(TempDatabase, TempDatabase::new);
}
