// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Database

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use nostr::{Event, EventId, Filter, FiltersMatchEvent, Url};
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
    events: Arc<RwLock<HashMap<EventId, Event>>>,
    // TODO: add messages queue? (messages not sent)
}

impl MemoryDatabase {
    /// New Memory database
    pub fn new() -> Self {
        Self::default()
    }

    fn _event_id_seen(
        &self,
        seen_event_ids: &mut HashMap<EventId, HashSet<Url>>,
        event_id: EventId,
        relay_url: Option<Url>,
    ) {
        seen_event_ids
            .entry(event_id)
            .and_modify(|set| {
                if let Some(relay_url) = &relay_url {
                    set.insert(relay_url.clone());
                }
            })
            .or_insert_with(|| match relay_url {
                Some(relay_url) => {
                    let mut set = HashSet::with_capacity(1);
                    set.insert(relay_url);
                    set
                }
                None => HashSet::with_capacity(0),
            });
    }
}

#[async_trait]
impl NostrDatabase for MemoryDatabase {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        Backend::Memory
    }

    async fn save_event(&self, event: &Event) -> Result<(), Self::Err> {
        let mut events = self.events.write().await;
        events.insert(event.id, event.clone());
        Ok(())
    }

    async fn save_events(&self, list: Vec<Event>) -> Result<(), Self::Err> {
        let mut events = self.events.write().await;
        for event in list.into_iter() {
            events.insert(event.id, event);
        }
        Ok(())
    }

    async fn has_event_already_been_seen(&self, event_id: EventId) -> Result<bool, Self::Err> {
        let seen_event_ids = self.seen_event_ids.read().await;
        Ok(seen_event_ids.contains_key(&event_id))
    }

    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: Option<Url>,
    ) -> Result<(), Self::Err> {
        let mut seen_event_ids = self.seen_event_ids.write().await;
        self._event_id_seen(&mut seen_event_ids, event_id, relay_url);
        Ok(())
    }

    async fn event_ids_seen(
        &self,
        event_ids: Vec<EventId>,
        relay_url: Option<Url>,
    ) -> Result<(), Self::Err> {
        let mut seen_event_ids = self.seen_event_ids.write().await;
        for event_id in event_ids.into_iter() {
            self._event_id_seen(&mut seen_event_ids, event_id, relay_url.clone());
        }

        Ok(())
    }

    async fn event_recently_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err> {
        let seen_event_ids = self.seen_event_ids.read().await;
        Ok(seen_event_ids.get(&event_id).cloned())
    }

    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err> {
        let events = self.events.read().await;
        let mut list: Vec<Event> = Vec::new();
        for event in events.values() {
            if filters.match_event(event) {
                list.push(event.clone());
            }
        }
        Ok(list)
    }

    async fn event_ids_by_filters(&self, filters: Vec<Filter>) -> Result<Vec<EventId>, Self::Err> {
        let events = self.events.read().await;
        let mut list: Vec<EventId> = Vec::new();
        for event in events.values() {
            if filters.match_event(event) {
                list.push(event.id);
            }
        }
        Ok(list)
    }
}
