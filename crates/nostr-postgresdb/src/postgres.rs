use deadpool::managed::{Object, Pool};
use diesel::dsl::{Eq, Filter as DieselFilter, InnerJoin, IntoBoxed};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::QueryResult;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use nostr::event::*;
use nostr::filter::Filter;
use nostr_database::*;

use super::model::{EventDataDb, EventDb};
use super::schema::nostr::{event_tags, events};

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
        let event_id = event_id.to_hex();
        let res = events::table
            .select(EventDb::as_select())
            .filter(events::id.eq(event_id))
            .first(&mut self.get_connection().await?)
            .await
            .optional()
            .map_err(DatabaseError::backend)?;
        Ok(res)
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

/// sets the given default limit on a Nostr filter if not set
pub fn with_limit(filter: Filter, default_limit: usize) -> Filter {
    if filter.limit.is_none() {
        return filter.limit(default_limit);
    }
    filter
}

// filter type of a join query.
type QuerySetJoinType<'a> = IntoBoxed<
    'a,
    DieselFilter<
        InnerJoin<events::table, event_tags::table>,
        Eq<event_tags::event_id, diesel::expression::SqlLiteral<diesel::sql_types::VarChar>>,
    >,
    Pg,
>;

pub fn build_filter_query<'a>(filter: Filter) -> QuerySetJoinType<'a> {
    let mut query = events::table
        .distinct_on(events::id)
        .inner_join(event_tags::table)
        .into_boxed();

    if let Some(limit) = filter.limit {
        query = query.limit(limit as i64);
    }

    if !has_filters(&filter) {
        return query;
    }
    if let Some(ids) = filter.ids.clone() {
        let values = ids.iter().map(|id| id.to_hex()).collect::<Vec<_>>();
        query = query.filter(events::id.eq_any(values));
    }

    if let Some(authors) = filter.authors.clone() {
        let values = authors.iter().map(|a| a.to_hex()).collect::<Vec<_>>();
        query = query.filter(events::pubkey.eq_any(values));
    }

    if let Some(kinds) = filter.kinds.clone() {
        let values = kinds.iter().map(|k| k.as_u16() as i64).collect::<Vec<_>>();
        query = query.filter(events::kind.eq_any(values));
    }

    if let Some(since) = filter.since {
        query = query.filter(events::created_at.ge(since.as_u64() as i64));
    }

    if let Some(until) = filter.until {
        query = query.filter(events::created_at.le(until.as_u64() as i64));
    }

    if !filter.generic_tags.is_empty() {
        for (tag, values) in filter.generic_tags.into_iter() {
            let values = values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
            query = query.filter(
                event_tags::tag
                    .eq(tag.to_string())
                    .and(event_tags::tag_value.eq_any(values)),
            );
        }
    }

    query
}

// determine if the filter has any filters set
fn has_filters(filter: &Filter) -> bool {
    filter.ids.is_some()
        || filter.authors.is_some()
        || filter.kinds.is_some()
        || filter.since.is_some()
        || filter.until.is_some()
        || !filter.generic_tags.is_empty()
        || filter.limit.is_some()
}
