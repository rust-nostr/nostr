// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Multi-storage

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use nostr::nips::nip01::Coordinate;
use nostr::{Event, EventId, Filter, RelayUrl, Timestamp};

use crate::events::NostrEventsDatabase;
use crate::{
    Backend, DatabaseError, DatabaseEventStatus, Events, NostrDatabase, NostrDatabaseWipe,
    SaveEventStatus,
};

/// Multi-storage database
///
/// This struct allows using different types of backends.
// TODO: should be this the `NostrDatabase`? If yes, remove the `NostrDatabase` trait
#[derive(Debug, Clone)]
pub struct MultiStorageDatabase {
    /// Events storage
    pub events: Arc<dyn NostrEventsDatabase>,
    // TODO: add future traits here
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrDatabase for MultiStorageDatabase {
    fn backend(&self) -> Backend {
        Backend::MultiBackend
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrEventsDatabase for MultiStorageDatabase {
    async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, DatabaseError> {
        self.events.save_event(event).await
    }

    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
        self.events.check_id(event_id).await
    }

    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: &Timestamp,
    ) -> Result<bool, DatabaseError> {
        self.events
            .has_coordinate_been_deleted(coordinate, timestamp)
            .await
    }

    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: RelayUrl,
    ) -> Result<(), DatabaseError> {
        self.events.event_id_seen(event_id, relay_url).await
    }

    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError> {
        self.events.event_seen_on_relays(event_id).await
    }

    async fn event_by_id(&self, id: &EventId) -> Result<Option<Event>, DatabaseError> {
        self.events.event_by_id(id).await
    }

    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        self.events.count(filters).await
    }

    async fn query(&self, filters: Vec<Filter>) -> Result<Events, DatabaseError> {
        self.events.query(filters).await
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        self.events.negentropy_items(filter).await
    }

    async fn delete(&self, filter: Filter) -> Result<(), DatabaseError> {
        self.events.delete(filter).await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrDatabaseWipe for MultiStorageDatabase {
    async fn wipe(&self) -> Result<(), DatabaseError> {
        self.events.wipe().await?;
        Ok(())
    }
}
