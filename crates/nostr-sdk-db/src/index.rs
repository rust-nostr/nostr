// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Indexes

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Alphabet, Event, EventId, Filter, Kind, Timestamp};
use tokio::sync::RwLock;

type Mapping = HashMap<SmallerIdentifier, EventId>;
type KindIndex = HashMap<Kind, HashSet<MappingIdentifier>>;
type AuthorIndex = HashMap<XOnlyPublicKey, HashSet<MappingIdentifier>>;
type CreatedAtIndex = BTreeMap<Timestamp, HashSet<MappingIdentifier>>;
type TagIndex = HashMap<Alphabet, HashMap<MappingIdentifier, HashSet<String>>>;

/// Event Index Result
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventIndexResult {
    /// Handled event should be stored into database?
    pub to_store: bool,
    /// List of events that should be removed from database
    pub to_discard: HashSet<EventId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SmallerIdentifier([u8; 8]);

impl SmallerIdentifier {
    pub fn new(sid: [u8; 8]) -> Self {
        Self(sid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
struct MappingIdentifier {
    pub timestamp: Timestamp,
    pub sid: SmallerIdentifier,
}

impl PartialOrd for MappingIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MappingIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        let timestamp_cmp = other.timestamp.cmp(&self.timestamp);
        if timestamp_cmp != Ordering::Equal {
            return timestamp_cmp;
        }

        self.sid.cmp(&other.sid)
    }
}

/// Database Indexes
#[derive(Debug, Clone, Default)]
pub struct DatabaseIndexes {
    counter: Arc<AtomicU64>,
    mapping: Arc<RwLock<Mapping>>,
    kinds_index: Arc<RwLock<KindIndex>>,
    authors_index: Arc<RwLock<AuthorIndex>>,
    created_at_index: Arc<RwLock<CreatedAtIndex>>,
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

        let mut should_insert: bool = true;
        let mut to_discard = HashSet::new();

        let mut mapping = self.mapping.write().await;
        let mut kinds_index = self.kinds_index.write().await;
        let mut authors_index = self.authors_index.write().await;

        if event.is_replaceable() {
            // Query event
            let mut matching_sids: BTreeSet<MappingIdentifier> = BTreeSet::new();
            let mut kinds = HashSet::with_capacity(1);
            let mut authors = HashSet::with_capacity(1);
            kinds.insert(event.kind);
            authors.insert(event.pubkey);
            let temp = self.query_index(&kinds_index, &kinds);
            intersect_or_extend(&mut matching_sids, &temp);
            let temp = self.query_index(&authors_index, &authors);
            intersect_or_extend(&mut matching_sids, &temp);

            if let Some(mid) = matching_sids.into_iter().next() {
                match mapping.get(&mid.sid) {
                    Some(event_id) => {
                        if mid.timestamp >= event.created_at {
                            should_insert = false;
                        } else if mid.timestamp < event.created_at {
                            to_discard.insert(*event_id);
                        }
                    }
                    None => tracing::warn!("Event ID not found for {mid:?}"),
                }
            }
        } else if event.is_parameterized_replaceable() {
            match event.identifier() {
                Some(_identifier) => {
                    should_insert = false;
                    /* let filter: Filter = Filter::new()
                        .author(event.pubkey)
                        .kind(event.kind)
                        .identifier(identifier);
                    let res: Vec<Event> = self._query(events, vec![filter]).await?;
                    if let Some(ev) = res.into_iter().next() {
                        if ev.created_at >= event.created_at {
                            should_insert = false;
                        } else if ev.created_at < event.created_at {
                            events.remove(&ev.id);
                        }
                    } */
                }
                None => should_insert = false,
            }
        }

        if should_insert {
            let mapping_id = MappingIdentifier {
                sid: self.next_sid(),
                timestamp: event.created_at,
            };

            mapping.insert(mapping_id.sid, event.id);

            // Index kind
            self.index_event_kind(&mut kinds_index, mapping_id, event);

            // Index author
            self.index_event_author(&mut authors_index, mapping_id, event);

            // Index created at
            let mut created_at_index = self.created_at_index.write().await;
            self.index_event_created_at(&mut created_at_index, mapping_id, event);

            // Index tags
            let mut tags_index = self.tags_index.write().await;
            self.index_event_tags(&mut tags_index, mapping_id, event);
        }

        EventIndexResult {
            to_store: should_insert,
            to_discard,
        }
    }

    fn next_sid(&self) -> SmallerIdentifier {
        let next_id = self.counter.fetch_add(1, AtomicOrdering::SeqCst);
        SmallerIdentifier::new(next_id.to_be_bytes())
    }

    /// Index kind
    fn index_event_kind(&self, kinds_index: &mut KindIndex, mid: MappingIdentifier, event: &Event) {
        kinds_index
            .entry(event.kind)
            .and_modify(|set| {
                set.insert(mid);
            })
            .or_insert_with(|| {
                let mut set = HashSet::with_capacity(1);
                set.insert(mid);
                set
            });
    }

    /// Index author
    fn index_event_author(
        &self,
        authors_index: &mut AuthorIndex,
        mid: MappingIdentifier,
        event: &Event,
    ) {
        authors_index
            .entry(event.pubkey)
            .and_modify(|set| {
                set.insert(mid);
            })
            .or_insert_with(|| {
                let mut set = HashSet::with_capacity(1);
                set.insert(mid);
                set
            });
    }

    /// Index created at
    fn index_event_created_at(
        &self,
        created_at_index: &mut CreatedAtIndex,
        mid: MappingIdentifier,
        event: &Event,
    ) {
        created_at_index
            .entry(event.created_at)
            .and_modify(|set| {
                set.insert(mid);
            })
            .or_insert_with(|| {
                let mut set = HashSet::with_capacity(1);
                set.insert(mid);
                set
            });
    }

    /// Index tags
    fn index_event_tags(&self, tags_index: &mut TagIndex, mid: MappingIdentifier, event: &Event) {
        for (a, set) in event.build_tags_index().into_iter() {
            tags_index
                .entry(a)
                .and_modify(|map| {
                    map.insert(mid, set.clone());
                })
                .or_insert_with(|| {
                    let mut map = HashMap::with_capacity(1);
                    map.insert(mid, set);
                    map
                });
        }
    }

    /// Query
    #[tracing::instrument(skip_all)]
    pub async fn query(&self, filters: Vec<Filter>) -> HashSet<EventId> {
        let mut matching_event_ids: HashSet<EventId> = HashSet::new();

        let kinds_index = self.kinds_index.read().await;
        let authors_index = self.authors_index.read().await;
        let created_at_index = self.created_at_index.read().await;
        let tags_index = self.tags_index.read().await;
        let mapping = self.mapping.read().await;

        for filter in filters.into_iter() {
            if !filter.ids.is_empty() {
                matching_event_ids.extend(filter.ids.iter().copied());
                continue;
            }

            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            let mut matching_sids: BTreeSet<MappingIdentifier> = BTreeSet::new();

            if !filter.kinds.is_empty() {
                let temp = self.query_index(&kinds_index, &filter.kinds);
                intersect_or_extend(&mut matching_sids, &temp);
            }

            if !filter.authors.is_empty() {
                let temp = self.query_index(&authors_index, &filter.authors);
                intersect_or_extend(&mut matching_sids, &temp);
            }

            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                let mut temp = BTreeSet::new();
                for ids in created_at_index.range(since..=until).map(|(_, ids)| ids) {
                    temp.extend(ids);
                }
                intersect_or_extend(&mut matching_sids, &temp);
            } else {
                if let Some(since) = filter.since {
                    let mut temp = BTreeSet::new();
                    for (_, ids) in created_at_index.range(since..) {
                        temp.extend(ids);
                    }
                    intersect_or_extend(&mut matching_sids, &temp);
                }

                if let Some(until) = filter.until {
                    let mut temp = BTreeSet::new();
                    for (_, ids) in created_at_index.range(..=until) {
                        temp.extend(ids);
                    }
                    intersect_or_extend(&mut matching_sids, &temp);
                }
            }

            if !filter.generic_tags.is_empty() {
                let mut temp = BTreeSet::new();

                for (tagname, set) in filter.generic_tags.iter() {
                    if let Some(tag_map) = tags_index.get(tagname) {
                        for (id, tag_values) in tag_map {
                            if set.iter().all(|value| tag_values.contains(value)) {
                                temp.insert(*id);
                            }
                        }
                    }
                }

                intersect_or_extend(&mut matching_sids, &temp);
            }

            let limit: usize = filter.limit.unwrap_or(matching_sids.len());
            let mut ids: Vec<EventId> = Vec::with_capacity(limit);

            for mid in matching_sids.into_iter().take(limit) {
                match mapping.get(&mid.sid) {
                    Some(event_id) => ids.push(*event_id),
                    None => tracing::warn!("Event ID not found for {mid:?}"),
                }
            }

            matching_event_ids.extend(ids);
        }

        matching_event_ids
    }

    fn query_index<K>(
        &self,
        index: &HashMap<K, HashSet<MappingIdentifier>>,
        keys: &HashSet<K>,
    ) -> BTreeSet<MappingIdentifier>
    where
        K: Eq + Hash,
    {
        let mut result: BTreeSet<MappingIdentifier> = BTreeSet::new();
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

fn intersect_or_extend<T>(main: &mut BTreeSet<T>, other: &BTreeSet<T>)
where
    T: Eq + Ord + Copy,
{
    if main.is_empty() {
        main.extend(other);
    } else {
        *main = main.intersection(other).copied().collect();
    }
}
