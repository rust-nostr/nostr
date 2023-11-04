// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Indexes

use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Alphabet, Event, EventId, Filter, Kind, Timestamp};
use tokio::sync::RwLock;

//type Mapping = HashMap<SmallerIdentifier, EventId>;
type KindIndex = HashMap<Kind, BTreeSet<MappingIdentifier>>;
type AuthorIndex = HashMap<PublicKeyPrefix, BTreeSet<MappingIdentifier>>;
type AuthorAndKindIndex = HashMap<(PublicKeyPrefix, Kind), BTreeSet<MappingIdentifier>>;
type CreatedAtIndex = BTreeMap<Timestamp, BTreeSet<MappingIdentifier>>;
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
struct PublicKeyPrefix([u8; 8]);

impl From<XOnlyPublicKey> for PublicKeyPrefix {
    fn from(pk: XOnlyPublicKey) -> Self {
        let pk = pk.serialize();
        let mut prefix = [0u8; 8];
        prefix.copy_from_slice(&pk[..8]);
        Self(prefix)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MappingIdentifier {
    pub timestamp: Timestamp,
    pub eid: EventId,
}

impl PartialOrd for MappingIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MappingIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.timestamp != other.timestamp {
            other.timestamp.cmp(&self.timestamp)
        } else {
            self.eid.cmp(&other.eid)
        }
    }
}

#[derive(Debug, Default)]
struct DatabaseIndexesInner {
    //mapping: Mapping,
    kinds_index: KindIndex,
    authors_index: AuthorIndex,
    author_and_kind_index: AuthorAndKindIndex,
    created_at_index: CreatedAtIndex,
    tags_index: TagIndex,
}

/// Database Indexes
#[derive(Debug, Clone, Default)]
pub struct DatabaseIndexes {
    inner: Arc<RwLock<DatabaseIndexesInner>>,
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

        let mut inner = self.inner.write().await;

        if event.is_replaceable() {
            // Query event
            let mut kinds = HashSet::with_capacity(1);
            let mut authors = HashSet::with_capacity(1);
            kinds.insert(event.kind);
            authors.insert(event.pubkey);

            let res1 = self.query_index(&inner.kinds_index, &kinds.into_iter().collect());
            let res2 = self.query_index(
                &inner.authors_index,
                &authors.into_iter().map(|pk| pk.into()).collect(),
            );
            let matching_sids: BTreeSet<&MappingIdentifier> = multi_intersection(vec![res1, res2]);

            let mut mids_to_discard = HashSet::new();

            for mid in matching_sids.into_iter() {
                if mid.timestamp >= event.created_at {
                    should_insert = false;
                } else if mid.timestamp < event.created_at {
                    to_discard.insert(mid.eid);
                    mids_to_discard.insert(*mid);
                }
            }

            for mid in mids_to_discard.iter() {
                if let Some(set) = inner.kinds_index.get_mut(&event.kind) {
                    set.remove(mid);
                }
                if let Some(set) = inner.authors_index.get_mut(&event.pubkey.into()) {
                    set.remove(mid);
                }
                if let Some(set) = inner
                    .author_and_kind_index
                    .get_mut(&(event.pubkey.into(), event.kind))
                {
                    set.remove(mid);
                }
                if let Some(set) = inner.created_at_index.get_mut(&mid.timestamp) {
                    set.remove(mid);
                }
                for (_, map) in inner.tags_index.iter_mut() {
                    map.remove(mid);
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
                eid: event.id,
                timestamp: event.created_at,
            };

            //inner.mapping.insert(mapping_id.sid, event.id);

            let pk: PublicKeyPrefix = event.pubkey.into();

            // Index kind
            self.index_event_kind(&mut inner.kinds_index, mapping_id, event.kind);

            // Index author
            self.index_event_author(&mut inner.authors_index, mapping_id, pk);

            // Index author and kind
            self.index_event_author_and_kind(
                &mut inner.author_and_kind_index,
                mapping_id,
                pk,
                event.kind,
            );

            // Index created at
            self.index_event_created_at(&mut inner.created_at_index, mapping_id, event);

            // Index tags
            self.index_event_tags(&mut inner.tags_index, mapping_id, event);
        }

        EventIndexResult {
            to_store: should_insert,
            to_discard,
        }
    }

    /// Index kind
    fn index_event_kind(&self, kinds_index: &mut KindIndex, mid: MappingIdentifier, kind: Kind) {
        kinds_index
            .entry(kind)
            .and_modify(|set| {
                set.insert(mid);
            })
            .or_insert_with(|| {
                let mut set = BTreeSet::new();
                set.insert(mid);
                set
            });
    }

    /// Index author
    fn index_event_author(
        &self,
        authors_index: &mut AuthorIndex,
        mid: MappingIdentifier,
        pk: PublicKeyPrefix,
    ) {
        authors_index
            .entry(pk)
            .and_modify(|set| {
                set.insert(mid);
            })
            .or_insert_with(|| {
                let mut set = BTreeSet::new();
                set.insert(mid);
                set
            });
    }

    fn index_event_author_and_kind(
        &self,
        author_and_kind_index: &mut AuthorAndKindIndex,
        mid: MappingIdentifier,
        pk: PublicKeyPrefix,
        kind: Kind,
    ) {
        author_and_kind_index
            .entry((pk, kind))
            .and_modify(|set| {
                set.insert(mid);
            })
            .or_insert_with(|| {
                let mut set = BTreeSet::new();
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
                let mut set = BTreeSet::new();
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
        let inner = self.inner.read().await;

        let mut matching_event_ids: HashSet<EventId> = HashSet::new();

        for filter in filters.into_iter() {
            if !filter.ids.is_empty() {
                matching_event_ids.extend(filter.ids);
                continue;
            }

            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            let mut sets: Vec<BTreeSet<&MappingIdentifier>> = Vec::new();

            if !filter.kinds.is_empty() && !filter.authors.is_empty() {
                let mut set = HashSet::new();
                for author in filter.authors.iter() {
                    for kind in filter.kinds.iter() {
                        set.insert(((*author).into(), *kind));
                    }
                }
                sets.push(self.query_index(&inner.author_and_kind_index, &set));
            } else {
                if !filter.kinds.is_empty() {
                    sets.push(self.query_index(&inner.kinds_index, &filter.kinds));
                }

                if !filter.authors.is_empty() {
                    sets.push(self.query_index(
                        &inner.authors_index,
                        &filter.authors.into_iter().map(|pk| pk.into()).collect(),
                    ));
                }
            }

            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                let mut temp = BTreeSet::new();
                for ids in inner
                    .created_at_index
                    .range(since..=until)
                    .map(|(_, ids)| ids)
                {
                    temp.extend(ids);
                }
                sets.push(temp);
            } else {
                if let Some(since) = filter.since {
                    let mut temp = BTreeSet::new();
                    for (_, ids) in inner.created_at_index.range(since..) {
                        temp.extend(ids);
                    }
                    sets.push(temp);
                }

                if let Some(until) = filter.until {
                    let mut temp = BTreeSet::new();
                    for (_, ids) in inner.created_at_index.range(..=until) {
                        temp.extend(ids);
                    }
                    sets.push(temp);
                }
            }

            if !filter.generic_tags.is_empty() {
                let mut temp = BTreeSet::new();

                for (tagname, set) in filter.generic_tags.iter() {
                    if let Some(tag_map) = inner.tags_index.get(tagname) {
                        for (id, tag_values) in tag_map.iter() {
                            if set.iter().all(|value| tag_values.contains(value)) {
                                temp.insert(id);
                            }
                        }
                    }
                }

                sets.push(temp);
            }

            // Intersection
            let matching_sids: BTreeSet<&MappingIdentifier> = multi_intersection(sets);

            // Limit
            let limit: usize = filter.limit.unwrap_or(matching_sids.len());

            // Get ids
            matching_event_ids.extend(matching_sids.into_iter().take(limit).map(|mid| mid.eid));
        }

        matching_event_ids
    }

    #[tracing::instrument(skip_all)]
    fn query_index<'a, K>(
        &self,
        index: &'a HashMap<K, BTreeSet<MappingIdentifier>>,
        keys: &HashSet<K>,
    ) -> BTreeSet<&'a MappingIdentifier>
    where
        K: Eq + Hash,
    {
        let mut result: BTreeSet<&MappingIdentifier> = BTreeSet::new();
        for key in keys.iter() {
            if let Some(ids) = index.get(key) {
                result.extend(ids.iter());
            }
        }
        result
    }

    /// Clear indexes
    pub async fn clear(&self) {
        let mut inner = self.inner.write().await;
        inner.kinds_index.clear();
        inner.authors_index.clear();
        inner.created_at_index.clear();
    }
}

#[tracing::instrument(skip_all)]
fn multi_intersection<T>(mut sets: Vec<BTreeSet<&T>>) -> BTreeSet<&T>
where
    T: Ord,
{
    // Sort by len (DESC)
    sets.sort_by_cached_key(|set| Reverse(set.len()));

    if let Some(mut result) = sets.pop() {
        if !sets.is_empty() {
            result.retain(|item| sets.iter().all(|set| set.contains(item)));
        }
        result
    } else {
        BTreeSet::new()
    }
}
