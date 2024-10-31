// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::sync::Arc;

use nostr_sdk::database;
use uniffi::Enum;

use crate::error::Result;
use crate::protocol::nips::nip01::Coordinate;
use crate::protocol::{Event, EventId, Filter, Timestamp};

#[derive(Enum)]
pub enum DatabaseEventStatus {
    Saved,
    Deleted,
    NotExistent,
}

impl From<DatabaseEventStatus> for database::DatabaseEventStatus {
    fn from(value: DatabaseEventStatus) -> Self {
        match value {
            DatabaseEventStatus::Saved => Self::Saved,
            DatabaseEventStatus::Deleted => Self::Deleted,
            DatabaseEventStatus::NotExistent => Self::NotExistent,
        }
    }
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait CustomNostrDatabase: Send + Sync {
    /// Name of backend
    fn backend(&self) -> String;

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    ///
    /// **This method assume that [`Event`] was already verified**
    async fn save_event(&self, event: Arc<Event>) -> Result<bool>;

    /// Check event status by ID
    ///
    /// Check if the event is saved, deleted or not existent.
    async fn check_id(&self, event_id: Arc<EventId>) -> Result<DatabaseEventStatus>;

    /// Check if event with [`Coordinate`] has been deleted before [`Timestamp`]
    async fn has_coordinate_been_deleted(
        &self,
        coordinate: Arc<Coordinate>,
        timestamp: Arc<Timestamp>,
    ) -> Result<bool>;

    /// Set [`EventId`] as seen by relay
    ///
    /// Useful for NIP65 (aka gossip)
    async fn event_id_seen(&self, event_id: Arc<EventId>, relay_url: String) -> Result<()>;

    /// Get list of relays that have seen the [`EventId`]
    async fn event_seen_on_relays(&self, event_id: Arc<EventId>) -> Result<Option<Vec<String>>>;

    /// Get event by ID
    async fn event_by_id(&self, event_id: Arc<EventId>) -> Result<Option<Arc<Event>>>;

    /// Count number of [`Event`] found by filters
    ///
    /// Use `Filter::new()` or `Filter::default()` to count all events.
    async fn count(&self, filters: Vec<Arc<Filter>>) -> Result<u64>;

    /// Query store with filters
    async fn query(&self, filters: Vec<Arc<Filter>>) -> Result<Vec<Arc<Event>>>;

    /// Delete all events that match the `Filter`
    async fn delete(&self, filter: Arc<Filter>) -> Result<()>;

    /// Wipe all data
    async fn wipe(&self) -> Result<()>;
}

pub(super) struct IntermediateCustomNostrDatabase {
    pub(super) inner: Arc<dyn CustomNostrDatabase>,
}

impl fmt::Debug for IntermediateCustomNostrDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntermediateCustomNostrDatabase").finish()
    }
}

mod inner {
    use std::collections::HashSet;
    use std::ops::Deref;
    use std::sync::Arc;

    use nostr_sdk::database::{DatabaseError, NostrDatabase};
    use nostr_sdk::prelude::*;

    use super::IntermediateCustomNostrDatabase;

    #[async_trait]
    #[allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive
    impl NostrDatabase for IntermediateCustomNostrDatabase {
        fn backend(&self) -> Backend {
            Backend::Custom(self.inner.backend())
        }

        async fn save_event(&self, event: &Event) -> Result<bool, DatabaseError> {
            self.inner
                .save_event(Arc::new(event.to_owned().into()))
                .await
                .map_err(DatabaseError::backend)
        }

        async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
            self.inner
                .check_id(Arc::new((*event_id).into()))
                .await
                .map(|s| s.into())
                .map_err(DatabaseError::backend)
        }

        async fn has_coordinate_been_deleted(
            &self,
            coordinate: &Coordinate,
            timestamp: &Timestamp,
        ) -> Result<bool, DatabaseError> {
            self.inner
                .has_coordinate_been_deleted(
                    Arc::new(coordinate.to_owned().into()),
                    Arc::new((*timestamp).into()),
                )
                .await
                .map_err(DatabaseError::backend)
        }

        async fn event_id_seen(
            &self,
            event_id: EventId,
            relay_url: Url,
        ) -> Result<(), DatabaseError> {
            self.inner
                .event_id_seen(Arc::new(event_id.into()), relay_url.to_string())
                .await
                .map_err(DatabaseError::backend)
        }

        async fn event_seen_on_relays(
            &self,
            event_id: &EventId,
        ) -> Result<Option<HashSet<Url>>, DatabaseError> {
            let res = self
                .inner
                .event_seen_on_relays(Arc::new((*event_id).into()))
                .await
                .map_err(DatabaseError::backend)?;
            Ok(res.map(|list| {
                list.into_iter()
                    .filter_map(|u| Url::parse(&u).ok())
                    .collect()
            }))
        }

        async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Event>, DatabaseError> {
            Ok(self
                .inner
                .event_by_id(Arc::new((*event_id).into()))
                .await
                .map_err(DatabaseError::backend)?
                .map(|e| e.as_ref().deref().clone()))
        }

        async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
            let filters = filters.into_iter().map(|f| Arc::new(f.into())).collect();
            let res = self
                .inner
                .count(filters)
                .await
                .map_err(DatabaseError::backend)?;
            Ok(res as usize)
        }

        async fn query(&self, filters: Vec<Filter>) -> Result<Events, DatabaseError> {
            let mut events = Events::new(&filters);

            let filters = filters.into_iter().map(|f| Arc::new(f.into())).collect();
            let res = self
                .inner
                .query(filters)
                .await
                .map_err(DatabaseError::backend)?;

            events.extend(res.into_iter().map(|e| e.as_ref().deref().clone()));
            Ok(events)
        }

        async fn negentropy_items(
            &self,
            filter: Filter,
        ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
            let filter = Arc::new(filter.into());
            let res = self
                .inner
                .query(vec![filter])
                .await
                .map_err(DatabaseError::backend)?;
            Ok(res
                .into_iter()
                .map(|e| (*e.id(), *e.created_at()))
                .collect())
        }

        async fn delete(&self, filter: Filter) -> Result<(), DatabaseError> {
            self.inner
                .delete(Arc::new(filter.into()))
                .await
                .map_err(DatabaseError::backend)
        }

        async fn wipe(&self) -> Result<(), DatabaseError> {
            self.inner.wipe().await.map_err(DatabaseError::backend)
        }
    }
}
