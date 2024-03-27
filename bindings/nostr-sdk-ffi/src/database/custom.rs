// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::sync::Arc;

use nostr_ffi::nips::nip01::Coordinate;
use nostr_ffi::{Event, EventId, Filter, Timestamp};

use crate::error::Result;

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

    /// Check if [`Event`] has already been saved
    async fn has_event_already_been_saved(&self, event_id: Arc<EventId>) -> Result<bool>;

    /// Check if [`EventId`] has already been seen
    async fn has_event_already_been_seen(&self, event_id: Arc<EventId>) -> Result<bool>;

    /// Check if [`EventId`] has been deleted
    async fn has_event_id_been_deleted(&self, event_id: Arc<EventId>) -> Result<bool>;

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

    // TODO: for some reason this method cause issues with `#[uniffi::export(with_foreign)]`
    // `*const c_void` cannot be sent between threads safely [E0277] Help: within
    // `uniffi_core::oneshot::OneshotInner<ForeignFutureResult<*const c_void>>`,
    // the trait `std::marker::Send` is not implemented for `*const c_void`, which is required by
    // `{async block@bindings/nostr-sdk-ffi/src/database/custom.rs:13:1: 13:32}: std::marker::Send`
    // Note: required because it appears within the type `ForeignFutureResult<*const c_void>`
    // Note: required because it appears within the type `std::option::Option<ForeignFutureResult<*const c_void>>`
    // Note: required because it appears within the type `uniffi_core::oneshot::OneshotInner<ForeignFutureResult<*const c_void>>`
    // Note: required for `std::sync::Mutex<uniffi_core::oneshot::OneshotInner<ForeignFutureResult<*const c_void>>>` to implement `Sync`
    // Note: required for `Arc<std::sync::Mutex<uniffi_core::oneshot::OneshotInner<ForeignFutureResult<*const c_void>>>>` to implement `std::marker::Send`
    // Note: required because it appears within the type `uniffi_core::oneshot::Receiver<ForeignFutureResult<*const c_void>>`
    // Note: required because it captures the following types: `ForeignFuture`,
    // `uniffi_core::oneshot::Receiver<ForeignFutureResult<<std::result::Result<Arc<nostr_ffi::Event>, NostrSdkError> as LiftReturn<UniFfiTag>>::ReturnType>>`
    // Note: required because it's used within this `async` fn body
    // Note: required because it captures the following types: `impl std::future::Future<Output = std::result::Result<Arc<nostr_ffi::Event>, NostrSdkError>>`
    // Note: required because it's used within this `async` block
    // Note: required for the cast from `Pin<Box<{async block@bindings/nostr-sdk-ffi/src/database/custom.rs:13:1: 13:32}>>` to `Pin<Box<dyn std::future::Future<Output = std::result::Result<Arc<nostr_ffi::Event>, NostrSdkError>> + std::marker::Send>>`
    //
    // // Get [`Event`] by [`EventId`]
    // async fn event_by_id(&self, event_id: Arc<EventId>) -> Result<Arc<Event>>;

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
    use std::collections::{BTreeSet, HashSet};
    use std::ops::Deref;
    use std::sync::Arc;

    use nostr_sdk::database::{DatabaseError, NostrDatabase, Order};
    use nostr_sdk::prelude::*;

    use super::IntermediateCustomNostrDatabase;

    #[async_trait]
    impl NostrDatabase for IntermediateCustomNostrDatabase {
        type Err = DatabaseError;

        fn backend(&self) -> Backend {
            Backend::Custom(self.inner.backend())
        }

        async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
            self.inner
                .save_event(Arc::new(event.to_owned().into()))
                .await
                .map_err(DatabaseError::backend)
        }

        async fn bulk_import(&self, _events: BTreeSet<Event>) -> Result<(), Self::Err> {
            Ok(())
        }

        async fn has_event_already_been_saved(
            &self,
            event_id: &EventId,
        ) -> Result<bool, Self::Err> {
            self.inner
                .has_event_already_been_saved(Arc::new((*event_id).into()))
                .await
                .map_err(DatabaseError::backend)
        }

        async fn has_event_already_been_seen(&self, event_id: &EventId) -> Result<bool, Self::Err> {
            self.inner
                .has_event_already_been_seen(Arc::new((*event_id).into()))
                .await
                .map_err(DatabaseError::backend)
        }

        async fn has_event_id_been_deleted(&self, event_id: &EventId) -> Result<bool, Self::Err> {
            self.inner
                .has_event_id_been_deleted(Arc::new((*event_id).into()))
                .await
                .map_err(DatabaseError::backend)
        }

        async fn has_coordinate_been_deleted(
            &self,
            coordinate: &Coordinate,
            timestamp: Timestamp,
        ) -> Result<bool, Self::Err> {
            self.inner
                .has_coordinate_been_deleted(
                    Arc::new(coordinate.to_owned().into()),
                    Arc::new(timestamp.into()),
                )
                .await
                .map_err(DatabaseError::backend)
        }

        async fn event_id_seen(&self, event_id: EventId, relay_url: Url) -> Result<(), Self::Err> {
            self.inner
                .event_id_seen(Arc::new(event_id.into()), relay_url.to_string())
                .await
                .map_err(DatabaseError::backend)
        }

        async fn event_seen_on_relays(
            &self,
            event_id: EventId,
        ) -> Result<Option<HashSet<Url>>, Self::Err> {
            let res = self
                .inner
                .event_seen_on_relays(Arc::new(event_id.into()))
                .await
                .map_err(DatabaseError::backend)?;
            Ok(res.map(|list| {
                list.into_iter()
                    .filter_map(|u| Url::parse(&u).ok())
                    .collect()
            }))
        }

        async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err> {
            // TODO: use event_by_id directly
            let filter = Filter::new().id(event_id).limit(1);
            let events = self.query(vec![filter], Order::Desc).await?;
            events.first().cloned().ok_or(DatabaseError::NotFound)
        }

        async fn count(&self, filters: Vec<Filter>) -> Result<usize, Self::Err> {
            let filters = filters.into_iter().map(|f| Arc::new(f.into())).collect();
            let res = self
                .inner
                .count(filters)
                .await
                .map_err(DatabaseError::backend)?;
            Ok(res as usize)
        }

        async fn query(
            &self,
            filters: Vec<Filter>,
            _order: Order,
        ) -> Result<Vec<Event>, Self::Err> {
            let filters = filters.into_iter().map(|f| Arc::new(f.into())).collect();
            let res = self
                .inner
                .query(filters)
                .await
                .map_err(DatabaseError::backend)?;
            Ok(res
                .into_iter()
                .map(|e| e.as_ref().deref().clone())
                .collect())
        }

        async fn event_ids_by_filters(
            &self,
            filters: Vec<Filter>,
            _order: Order,
        ) -> Result<Vec<EventId>, Self::Err> {
            let filters = filters.into_iter().map(|f| Arc::new(f.into())).collect();
            let res = self
                .inner
                .query(filters)
                .await
                .map_err(DatabaseError::backend)?;
            Ok(res.into_iter().map(|e| *e.id()).collect())
        }

        async fn negentropy_items(
            &self,
            filter: Filter,
        ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
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

        async fn delete(&self, filter: Filter) -> Result<(), Self::Err> {
            self.inner
                .delete(Arc::new(filter.into()))
                .await
                .map_err(DatabaseError::backend)
        }

        async fn wipe(&self) -> Result<(), Self::Err> {
            self.inner.wipe().await.map_err(DatabaseError::backend)
        }
    }
}
