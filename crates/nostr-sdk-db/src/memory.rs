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

    async fn _query(
        &self,
        events: &HashMap<EventId, Event>,
        filters: Vec<Filter>,
    ) -> Result<Vec<Event>, DatabaseError> {
        let mut list: Vec<Event> = Vec::new();
        for event in events.values() {
            if filters.match_event(event) {
                list.push(event.clone());
            }
        }
        Ok(list)
    }

    async fn _save_event(
        &self,
        events: &mut HashMap<EventId, Event>,
        event: Event,
    ) -> Result<bool, DatabaseError> {
        self.event_id_seen(event.id, None).await?;

        if event.is_expired() || event.is_ephemeral() {
            tracing::warn!("Event {} not saved: expired or ephemeral", event.id);
            return Ok(false);
        }

        let mut should_insert: bool = true;

        if event.is_replaceable() {
            let filter: Filter = Filter::new()
                .author(event.pubkey.to_string())
                .kind(event.kind);
            let res: Vec<Event> = self._query(events, vec![filter]).await?;
            if let Some(ev) = res.into_iter().next() {
                if ev.created_at >= event.created_at {
                    should_insert = false;
                } else if ev.created_at < event.created_at {
                    events.remove(&ev.id);
                }
            }
        } else if event.is_parameterized_replaceable() {
            match event.identifier() {
                Some(identifier) => {
                    let filter: Filter = Filter::new()
                        .author(event.pubkey.to_string())
                        .kind(event.kind)
                        .identifier(identifier);
                    let res: Vec<Event> = self._query(events, vec![filter]).await?;
                    if let Some(ev) = res.into_iter().next() {
                        if ev.created_at >= event.created_at {
                            should_insert = false;
                        } else if ev.created_at < event.created_at {
                            events.remove(&ev.id);
                        }
                    }
                }
                None => should_insert = false,
            }
        }

        if should_insert {
            events.insert(event.id, event);
            Ok(true)
        } else {
            tracing::warn!("Event {} not saved: unknown", event.id);
            Ok(false)
        }
    }
}

#[async_trait]
impl NostrDatabase for MemoryDatabase {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        Backend::Memory
    }

    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        let mut events = self.events.write().await;
        self._save_event(&mut events, event.clone()).await
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

    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err> {
        let events = self.events.read().await;
        events
            .get(&event_id)
            .cloned()
            .ok_or(DatabaseError::NotFound)
    }

    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err> {
        let events = self.events.read().await;
        self._query(&events, filters).await
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
