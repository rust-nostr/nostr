// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Database

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use nostr::{Event, EventId, Filter, Url};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::{Backend, DatabaseError, NostrDatabase};

/// Memory Database Error
#[derive(Debug, Error)]
pub enum Error {}

impl From<Error> for DatabaseError {
    fn from(e: Error) -> Self {
        DatabaseError::backend(e)
    }
}

/// Memory Database (RAM)
#[derive(Debug, Default)]
pub struct MemoryDatabase {
    seen_event_ids: Arc<RwLock<HashMap<EventId, HashSet<Url>>>>,
}

impl MemoryDatabase {
    /// New Memory database
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl NostrDatabase for MemoryDatabase {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        Backend::Memory
    }

    async fn save_event(&self, _event: &Event) -> Result<(), Self::Err> {
        Ok(())
    }

    async fn save_event_id_seen_by_relay(
        &self,
        event_id: EventId,
        relay_url: Url,
    ) -> Result<(), Self::Err> {
        let mut seen_event_ids = self.seen_event_ids.write().await;
        seen_event_ids
            .entry(event_id)
            .and_modify(|set| {
                set.insert(relay_url.clone());
            })
            .or_insert_with(|| {
                let mut set = HashSet::with_capacity(1);
                set.insert(relay_url);
                set
            });
        Ok(())
    }

    async fn event_recently_seen_on_relays(
        &self,
        _event_id: EventId,
    ) -> Result<Vec<Url>, Self::Err> {
        todo!()
    }

    async fn query(&self, _filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err> {
        Ok(Vec::new())
    }

    async fn event_ids_by_filters(&self, _filters: Vec<Filter>) -> Result<Vec<EventId>, Self::Err> {
        Ok(Vec::new())
    }
}
