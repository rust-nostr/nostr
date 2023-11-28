// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr Database Indexes

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use nostr::event::raw::RawEvent;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Event, EventId, Filter, Kind, TagIndexValues, TagIndexes, Timestamp};
use tokio::sync::RwLock;

/// Public Key Prefix Size
const PUBLIC_KEY_PREFIX_SIZE: usize = 8;

/// Event Index
#[derive(Debug, Clone, PartialEq, Eq)]
struct EventIndex {
    /// Timestamp (seconds)
    created_at: Timestamp,
    /// Event ID
    event_id: EventId,
    /// Public key prefix
    pubkey: PublicKeyPrefix,
    /// Kind
    kind: Kind,
    /// Tag indexes
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
            self.created_at.cmp(&other.created_at).reverse()
        } else {
            self.event_id.cmp(&other.event_id)
        }
    }
}

impl TryFrom<RawEvent> for EventIndex {
    type Error = nostr::event::id::Error;
    fn try_from(raw: RawEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            created_at: Timestamp::from(raw.created_at),
            event_id: EventId::from_slice(&raw.id)?,
            pubkey: PublicKeyPrefix::from(raw.pubkey),
            kind: Kind::from(raw.kind),
            tags: TagIndexes::from(raw.tags.into_iter()),
        })
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
        if filter.generic_tags.is_empty() {
            return true;
        }

        if self.tags.is_empty() {
            return false;
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

/// Public Key prefix
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicKeyPrefix([u8; PUBLIC_KEY_PREFIX_SIZE]);

impl From<XOnlyPublicKey> for PublicKeyPrefix {
    fn from(pk: XOnlyPublicKey) -> Self {
        let pk: [u8; 32] = pk.serialize();
        Self::from(pk)
    }
}

impl From<[u8; 32]> for PublicKeyPrefix {
    fn from(pk: [u8; 32]) -> Self {
        let mut pubkey = [0u8; PUBLIC_KEY_PREFIX_SIZE];
        pubkey.copy_from_slice(&pk[..PUBLIC_KEY_PREFIX_SIZE]);
        Self(pubkey)
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
        I: IntoIterator<Item = RawEvent>,
    {
        let mut index = self.index.write().await;
        let now = Timestamp::now();
        index.extend(
            events
                .into_iter()
                .filter(|raw| !raw.is_expired(&now) && !raw.is_ephemeral())
                .filter_map(|raw| EventIndex::try_from(raw).ok()),
        );
    }

    /// Index [`Event`]
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn index_event(&self, event: &Event) -> EventIndexResult {
        // Check if it's expired or ephemeral
        if event.is_expired() || event.is_ephemeral() {
            return EventIndexResult::default();
        }

        let mut index = self.index.write().await;

        let mut should_insert: bool = true;
        let mut to_discard: HashSet<EventId> = HashSet::new();

        if event.is_replaceable() {
            let filter: Filter = Filter::new().author(event.pubkey).kind(event.kind);
            for ev in self.internal_query(&index, &filter).await {
                if ev.created_at > event.created_at {
                    should_insert = false;
                } else if ev.created_at <= event.created_at {
                    to_discard.insert(ev.event_id);
                }
            }
        } else if event.is_parameterized_replaceable() {
            match event.identifier() {
                Some(identifier) => {
                    let filter: Filter = Filter::new()
                        .author(event.pubkey)
                        .kind(event.kind)
                        .identifier(identifier);
                    for ev in self.internal_query(&index, &filter).await {
                        if ev.created_at >= event.created_at {
                            should_insert = false;
                        } else if ev.created_at < event.created_at {
                            to_discard.insert(ev.event_id);
                        }
                    }
                }
                None => should_insert = false,
            }
        }

        // Remove events
        if !to_discard.is_empty() {
            index.retain(|e| !to_discard.contains(&e.event_id));
        }

        // Insert event
        if should_insert {
            index.insert(EventIndex::from(event));
        }

        EventIndexResult {
            to_store: should_insert,
            to_discard,
        }
    }

    async fn internal_query<'a>(
        &self,
        index: &'a BTreeSet<EventIndex>,
        filter: &'a Filter,
    ) -> impl Iterator<Item = &'a EventIndex> {
        let authors: HashSet<PublicKeyPrefix> = filter
            .authors
            .iter()
            .map(|p| PublicKeyPrefix::from(*p))
            .collect();
        index.iter().filter(move |m| {
            (filter.ids.is_empty() || filter.ids.contains(&m.event_id))
                && filter.since.map_or(true, |t| m.created_at >= t)
                && filter.until.map_or(true, |t| m.created_at <= t)
                && (filter.authors.is_empty() || authors.contains(&m.pubkey))
                && (filter.kinds.is_empty() || filter.kinds.contains(&m.kind))
                && m.filter_tags_match(filter)
        })
    }

    /// Query
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn query(&self, filters: Vec<Filter>) -> HashSet<EventId> {
        let index = self.index.read().await;

        let mut matching_ids: HashSet<EventId> = HashSet::new();

        for filter in filters.into_iter() {
            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            let iter = self
                .internal_query(&index, &filter)
                .await
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
