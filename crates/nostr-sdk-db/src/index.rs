// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Indexes

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Alphabet, Event, EventId, Filter, Kind, Timestamp};
use tokio::sync::RwLock;

type TagIndex = HashMap<EventId, HashMap<Alphabet, HashSet<String>>>;

/// Event Index Result
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventIndexResult {
    /// Handled event should be stored into database?
    pub to_store: bool,
    /// List of events that should be removed from database
    pub to_discard: HashSet<EventId>,
}

/// Events Indexes
#[derive(Debug, Clone, Default)]
pub struct DatabaseIndexes {
    ids_index: Arc<RwLock<HashMap<EventId, Timestamp>>>,
    kinds_index: Arc<RwLock<HashMap<Kind, HashSet<EventId>>>>,
    authors_index: Arc<RwLock<HashMap<XOnlyPublicKey, HashSet<EventId>>>>,
    created_at_index: Arc<RwLock<HashMap<Timestamp, HashSet<EventId>>>>, // TODO: remove this and use BTreeMap?
    tags_index: Arc<RwLock<TagIndex>>,
}

impl DatabaseIndexes {
    /// New empty indexes
    pub fn new() -> Self {
        Self::default()
    }

    /// Index [`Event`]
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn index_event(&self, event: &Event) -> EventIndexResult {
        // Check if it's expired or ephemeral
        if event.is_expired() || event.is_ephemeral() {
            return EventIndexResult::default();
        }

        let should_insert: bool = true;
        
        // TODO: check if it's a [parametrized] replaceable event

        if should_insert {
            // Index id
            let mut ids_index = self.ids_index.write().await;
            self.index_event_id(&mut ids_index, event).await;

            // Index kind
            let mut kinds_index = self.kinds_index.write().await;
            self.index_event_kind(&mut kinds_index, event).await;

            // Index author
            let mut authors_index = self.authors_index.write().await;
            self.index_event_author(&mut authors_index, event).await;

            // Index created at
            let mut created_at_index = self.created_at_index.write().await;
            self.index_event_created_at(&mut created_at_index, event)
                .await;

            // Index tags
            let mut tags_index = self.tags_index.write().await;
            self.index_event_tags(&mut tags_index, event).await;
        }

        EventIndexResult {
            to_store: should_insert,
            to_discard: HashSet::new(),
        }
    }

    /// Index id
    async fn index_event_id(&self, ids_index: &mut HashMap<EventId, Timestamp>, event: &Event) {
        ids_index.insert(event.id, event.created_at);
    }

    /// Index kind
    async fn index_event_kind(
        &self,
        kinds_index: &mut HashMap<Kind, HashSet<EventId>>,
        event: &Event,
    ) {
        kinds_index
            .entry(event.kind)
            .and_modify(|set| {
                set.insert(event.id);
            })
            .or_insert_with(|| {
                let mut set = HashSet::with_capacity(1);
                set.insert(event.id);
                set
            });
    }

    /// Index author
    async fn index_event_author(
        &self,
        authors_index: &mut HashMap<XOnlyPublicKey, HashSet<EventId>>,
        event: &Event,
    ) {
        authors_index
            .entry(event.pubkey)
            .and_modify(|set| {
                set.insert(event.id);
            })
            .or_insert_with(|| {
                let mut set = HashSet::with_capacity(1);
                set.insert(event.id);
                set
            });
    }

    /// Index created at
    async fn index_event_created_at(
        &self,
        created_at_index: &mut HashMap<Timestamp, HashSet<EventId>>,
        event: &Event,
    ) {
        created_at_index
            .entry(event.created_at)
            .and_modify(|set| {
                set.insert(event.id);
            })
            .or_insert_with(|| {
                let mut set = HashSet::with_capacity(1);
                set.insert(event.id);
                set
            });
    }

    /// Index tags
    async fn index_event_tags(
        &self,
        tags_index: &mut TagIndex,
        event: &Event,
    ) {
        if !event.tags.is_empty() {
            tags_index.insert(event.id, event.build_tags_index());
        }
    }

    /// Query
    #[tracing::instrument(skip_all)]
    pub async fn query(&self, filter: &Filter) -> HashSet<EventId> {
        let mut matching_event_ids = HashSet::new();

        let kinds_index = self.kinds_index.read().await;
        let authors_index = self.authors_index.read().await;
        let created_at_index = self.created_at_index.read().await;
        let tags_index = self.tags_index.read().await;

        if !filter.kinds.is_empty() {
            let temp = self.query_index(&kinds_index, &filter.kinds).await;
            intersect_or_extend(&mut matching_event_ids, &temp);
        }

        if !filter.authors.is_empty() {
            let temp = self.query_index(&authors_index, &filter.authors).await;
            intersect_or_extend(&mut matching_event_ids, &temp);
        }

        // TODO: check if since >= until

        if let Some(since) = filter.since {
            let mut temp = HashSet::new();
            for (timestamp, ids) in created_at_index.iter() {
                if *timestamp >= since {
                    temp.extend(ids);
                }
            }
            intersect_or_extend(&mut matching_event_ids, &temp);
        }

        if let Some(until) = filter.until {
            let mut temp = HashSet::new();
            for (timestamp, ids) in created_at_index.iter() {
                if *timestamp <= until {
                    temp.extend(ids);
                }
            }
            intersect_or_extend(&mut matching_event_ids, &temp);
        }

        if !filter.generic_tags.is_empty() {
            let mut temp = HashSet::new();
            for (id, idx) in tags_index.iter() {
                if filter.generic_tags.iter().all(|(tagname, set)| {
                    idx.get(tagname)
                        .map(|valset| valset.intersection(set).count() > 0)
                        .unwrap_or(false)
                }) {
                    temp.insert(*id);
                }
            }
            intersect_or_extend(&mut matching_event_ids, &temp);
        }

        // TODO: sort by timestamp and use limit

        matching_event_ids
    }

    async fn query_index<K>(
        &self,
        index: &HashMap<K, HashSet<EventId>>,
        keys: &HashSet<K>,
    ) -> HashSet<EventId>
    where
        K: Eq + Hash,
    {
        let mut result: HashSet<EventId> = HashSet::new();
        for key in keys.iter() {
            if let Some(ids) = index.get(key) {
                result.extend(ids);
            }
        }
        result
    }

    /// Clear indexes
    pub async fn clear(&self) {
        let mut kinds_index = self.kinds_index.write().await;
        kinds_index.clear();

        let mut authors_index = self.authors_index.write().await;
        authors_index.clear();

        let mut created_at_index = self.created_at_index.write().await;
        created_at_index.clear();
    }
}

fn intersect_or_extend(main: &mut HashSet<EventId>, second: &HashSet<EventId>) {
    if main.is_empty() {
        main.extend(second);
    } else {
        *main = main.intersection(second).copied().collect();
    }
}
