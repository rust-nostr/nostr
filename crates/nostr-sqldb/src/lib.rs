mod migrations;
mod model;
mod postgres;
mod schema;
mod types;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use model::{EventDataDb, EventDb};
use nostr::event::*;
use nostr::filter::Filter;
use nostr::types::Timestamp;
use nostr::util::BoxedFuture;
use nostr_database::*;
use postgres::{build_filter_query, with_limit};

#[cfg(feature = "postgres")]
pub use postgres::{postgres_connection_pool, NostrPostgres};

#[cfg(feature = "postgres")]
pub use migrations::postgres::run_migrations;

#[cfg(feature = "postgres")]
use schema::postgres::{event_tags, events};

impl NostrDatabase for NostrPostgres {
    fn backend(&self) -> Backend {
        Backend::Custom("Postgres".to_string())
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
        _event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            let event = match self.event_by_id(_event_id).await? {
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

/// For now we want to avoid wiping the database
impl NostrDatabaseWipe for NostrPostgres {
    #[inline]
    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { Err(DatabaseError::NotSupported) })
    }
}
