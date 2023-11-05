// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Indexes

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Event, EventId, Filter, Kind, TagIndexValues, TagIndexes, Timestamp};
#[cfg(feature = "flatbuffers")]
use nostr_sdk_fbs::{event_fbs, Error as FlatBuffersError, FlatBufferDecode};
use tokio::sync::RwLock;

const PUBLIC_KEY_PREFIX_SIZE: usize = 8;

/// Event Index
#[derive(Debug, PartialEq, Eq)]
pub struct EventIndex {
    created_at: Timestamp,
    event_id: EventId,
    pubkey: PublicKeyPrefix,
    kind: Kind,
    tags: TagIndexes,
}

impl PartialOrd for EventIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EventIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.created_at != other.created_at {
            other.created_at.cmp(&self.created_at)
        } else {
            self.event_id.cmp(&other.event_id)
        }
    }
}

impl From<&Event> for EventIndex {
    fn from(e: &Event) -> Self {
        Self {
            created_at: e.created_at,
            event_id: e.id,
            pubkey: PublicKeyPrefix::from(e.pubkey),
            kind: e.kind,
            tags: e.build_tags_index(),
        }
    }
}

impl EventIndex {
    fn filter_tags_match(&self, filter: &Filter) -> bool {
        if filter.generic_tags.is_empty() || self.tags.is_empty() {
            return true;
        }

        filter.generic_tags.iter().all(|(tagname, set)| {
            let set = TagIndexValues::from(set);
            self.tags
                .get(tagname)
                .map(|valset| valset.intersection(&set).count() > 0)
                .unwrap_or(false)
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PublicKeyPrefix([u8; PUBLIC_KEY_PREFIX_SIZE]);

impl From<XOnlyPublicKey> for PublicKeyPrefix {
    fn from(pk: XOnlyPublicKey) -> Self {
        let pk = pk.serialize();
        let mut prefix = [0u8; PUBLIC_KEY_PREFIX_SIZE];
        prefix.copy_from_slice(&pk[..PUBLIC_KEY_PREFIX_SIZE]);
        Self(prefix)
    }
}

#[cfg(feature = "flatbuffers")]
impl FlatBufferDecode for EventIndex {
    fn decode(buf: &[u8]) -> Result<Self, FlatBuffersError> {
        let ev = event_fbs::root_as_event(buf)?;

        // Compose Public Key prefix
        let pk = ev.pubkey().ok_or(FlatBuffersError::NotFound)?.0;
        let mut pubkey = [0u8; PUBLIC_KEY_PREFIX_SIZE];
        pubkey.copy_from_slice(&pk[..PUBLIC_KEY_PREFIX_SIZE]);

        // Compose tags
        let iter = ev
            .tags()
            .ok_or(FlatBuffersError::NotFound)?
            .into_iter()
            .filter_map(|tag| match tag.data() {
                Some(t) => {
                    if t.len() > 1 {
                        Some(t.into_iter().collect::<Vec<&str>>())
                    } else {
                        None
                    }
                }
                None => None,
            });
        let tags = TagIndexes::from(iter);

        Ok(Self {
            event_id: EventId::from_slice(&ev.id().ok_or(FlatBuffersError::NotFound)?.0)?,
            pubkey: PublicKeyPrefix(pubkey),
            created_at: Timestamp::from(ev.created_at()),
            kind: Kind::from(ev.kind()),
            tags,
        })
    }
}

/// Event Index Result
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventIndexResult {
    /// Handled event should be stored into database?
    pub to_store: bool,
    /// List of events that should be removed from database
    pub to_discard: HashSet<EventId>,
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

/// Database Indexes
#[derive(Debug, Clone, Default)]
pub struct DatabaseIndexes {
    index: Arc<RwLock<BTreeSet<EventIndex>>>,
}

impl DatabaseIndexes {
    /// New empty indexes
    pub fn new() -> Self {
        Self::default()
    }

    /// Bulk load
    #[tracing::instrument(skip_all)]
    pub async fn bulk_load<I>(&self, events: I)
    where
        I: IntoIterator<Item = EventIndex>,
    {
        let mut index = self.index.write().await;
        index.extend(events);
    }

    /// Index [`Event`]
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn index_event(&self, event: &Event) -> EventIndexResult {
        // Check if it's expired or ephemeral
        if event.is_expired() || event.is_ephemeral() {
            return EventIndexResult::default();
        }

        let should_insert: bool = true;
        let to_discard = HashSet::new();

        /* if event.is_replaceable() {
            let filter: Filter = Filter::new().author(event.pubkey).kind(event.kind);
            let res = self.query(events, vec![filter]).await;
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
                    }
                }
                None => should_insert = false,
            }
        } */

        if should_insert {
            let mut index = self.index.write().await;
            index.insert(EventIndex::from(event));
        }

        EventIndexResult {
            to_store: should_insert,
            to_discard,
        }
    }

    /// Query
    #[tracing::instrument(skip_all)]
    pub async fn query(&self, filters: Vec<Filter>) -> HashSet<EventId> {
        let index = self.index.read().await;

        let mut matching_ids: HashSet<EventId> = HashSet::new();

        for filter in filters.into_iter() {
            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            let authors: HashSet<PublicKeyPrefix> = filter
                .authors
                .iter()
                .map(|p| PublicKeyPrefix::from(*p))
                .collect();
            let iter = index
                .iter()
                .filter(|m| {
                    (filter.ids.is_empty() || filter.ids.contains(&m.event_id))
                        && filter.since.map_or(true, |t| m.created_at >= t)
                        && filter.until.map_or(true, |t| m.created_at <= t)
                        && (filter.authors.is_empty() || authors.contains(&m.pubkey))
                        && (filter.kinds.is_empty() || filter.kinds.contains(&m.kind))
                        && m.filter_tags_match(&filter)
                })
                .map(|m| m.event_id);
            if let Some(limit) = filter.limit {
                matching_ids.extend(iter.take(limit))
            } else {
                matching_ids.extend(iter)
            }
        }

        matching_ids
    }

    /// Clear indexes
    pub async fn clear(&self) {
        let mut index = self.index.write().await;
        index.clear();
    }
}
