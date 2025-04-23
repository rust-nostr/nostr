use deadpool::managed::{Object, Pool};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::QueryResult;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use nostr::event::*;
use nostr::filter::Filter;
use nostr::types::Timestamp;
use nostr_database::*;
use prelude::BoxedFuture;

use super::model::{EventDataDb, EventDb};
use super::schema::postgres::{event_tags, events};
use crate::query::{build_filter_query, event_by_id, with_limit};

/// Shorthand for a database connection pool type
pub type PostgresConnectionPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub type PostgresConnection = Object<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone)]
pub struct NostrPostgres {
    pool: PostgresConnectionPool,
}

impl NostrPostgres {
    /// Create a new [`NostrPostgres`] instance
    pub async fn new<C>(connection_string: C) -> Result<Self, DatabaseError>
    where
        C: AsRef<str>,
    {
        crate::migrations::postgres::run_migrations(connection_string.as_ref())?;
        let pool = postgres_connection_pool(connection_string).await?;
        Ok(Self { pool })
    }

    pub(crate) async fn get_connection(&self) -> Result<PostgresConnection, DatabaseError> {
        self.pool.get().await.map_err(DatabaseError::backend)
    }

    pub(crate) async fn save(
        &self,
        event_data: EventDataDb,
    ) -> Result<SaveEventStatus, DatabaseError> {
        let mut db = self.get_connection().await?;
        let result: QueryResult<bool> = db
            .transaction(|c| {
                async move {
                    diesel::insert_into(events::table)
                        .values(&event_data.event)
                        .execute(c)
                        .await?;

                    diesel::insert_into(event_tags::table)
                        .values(&event_data.tags)
                        .execute(c)
                        .await?;

                    Ok(true)
                }
                .scope_boxed()
            })
            .await;

        match result {
            Ok(_) => Ok(SaveEventStatus::Success),
            Err(e) => match e {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    Ok(SaveEventStatus::Rejected(RejectedReason::Duplicate))
                }
                e => Err(DatabaseError::backend(e)),
            },
        }
    }

    pub(crate) async fn event_by_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<EventDb>, DatabaseError> {
        let res = event_by_id(event_id)
            .first(&mut self.get_connection().await?)
            .await
            .optional()
            .map_err(DatabaseError::backend)?;
        Ok(res)
    }
}

impl NostrEventsDatabase for NostrPostgres {
    /// Save [`Event`] into store
    ///
    /// **This method assumes that [`Event`] was already verified**
    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>> {
        Box::pin(async move { self.save(EventDataDb::try_from(event)?).await })
    }

    /// Check event status by ID
    ///
    /// Check if the event is saved, deleted or not existent.
    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        Box::pin(async move {
            let status = match self.event_by_id(event_id).await? {
                Some(e) if e.deleted => DatabaseEventStatus::Deleted,
                Some(_) => DatabaseEventStatus::Saved,
                None => DatabaseEventStatus::NotExistent,
            };
            Ok(status)
        })
    }

    /// Coordinate feature is not supported yet
    fn has_coordinate_been_deleted<'a>(
        &'a self,
        _coordinate: &'a nostr::nips::nip01::CoordinateBorrow<'a>,
        _timestamp: &'a Timestamp,
    ) -> BoxedFuture<'a, Result<bool, DatabaseError>> {
        Box::pin(async move { Ok(false) })
    }

    /// Get [`Event`] by [`EventId`]
    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            let event = match self.event_by_id(event_id).await? {
                Some(e) if !e.deleted => {
                    Some(Event::decode(&e.payload).map_err(DatabaseError::backend)?)
                }
                _ => None,
            };
            Ok(event)
        })
    }

    /// Count the number of events found with [`Filter`].
    ///
    /// Use `Filter::new()` or `Filter::default()` to count all events.
    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move {
            let res: i64 = build_filter_query(filter)
                .count()
                .get_result(&mut self.get_connection().await?)
                .await
                .map_err(DatabaseError::backend)?;
            Ok(res as usize)
        })
    }

    /// Query stored events.
    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        let filter = with_limit(filter, 10000);
        Box::pin(async move {
            let mut events = Events::new(&filter);
            let result = build_filter_query(filter.clone())
                .select(EventDb::as_select())
                .load(&mut self.get_connection().await?)
                .await
                .map_err(DatabaseError::backend)?;

            for item in result.into_iter() {
                if let Ok(event) = Event::decode(&item.payload) {
                    events.insert(event);
                }
            }
            Ok(events)
        })
    }

    /// Delete all events that match the [Filter]
    fn delete(&self, filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        let filter = with_limit(filter, 999);
        Box::pin(async move {
            let filter = build_filter_query(filter);
            diesel::update(events::table)
                .set(events::deleted.eq(true))
                .filter(events::id.eq_any(filter.select(events::id)))
                .execute(&mut self.get_connection().await?)
                .await
                .map_err(DatabaseError::backend)?;

            Ok(())
        })
    }
}

impl NostrDatabase for NostrPostgres {
    fn backend(&self) -> Backend {
        Backend::Custom("Postgres".to_string())
    }
}

/// Create a new [`NostrPostgres`] instance from an existing connection pool
impl From<PostgresConnectionPool> for NostrPostgres {
    fn from(pool: PostgresConnectionPool) -> Self {
        Self { pool }
    }
}

/// Create a connection pool for a Postgres database with the given connection string.
pub async fn postgres_connection_pool<C>(
    connection_string: C,
) -> Result<PostgresConnectionPool, DatabaseError>
where
    C: AsRef<str>,
{
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(connection_string.as_ref());
    let pool: PostgresConnectionPool = Pool::builder(config)
        .build()
        .map_err(|e| DatabaseError::Backend(Box::new(e)))?;
    Ok(pool)
}

impl std::fmt::Debug for NostrPostgres {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NostrPostgres")
            .field("pool", &self.pool.status())
            .finish()
    }
}

/// For now we want to avoid wiping the database
impl NostrDatabaseWipe for NostrPostgres {
    #[inline]
    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { Err(DatabaseError::NotSupported) })
    }
}
