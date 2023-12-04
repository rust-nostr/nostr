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

impl From<&XOnlyPublicKey> for PublicKeyPrefix {
    fn from(pk: &XOnlyPublicKey) -> Self {
        let pk: [u8; 32] = pk.serialize();
        Self::from(pk)
    }
}

impl From<XOnlyPublicKey> for PublicKeyPrefix {
    fn from(pk: XOnlyPublicKey) -> Self {
        Self::from(&pk)
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
    deleted: Arc<RwLock<HashSet<EventId>>>,
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
    ///
    /// **This method assume that [`Event`] was already verified**
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
            for ev in self.internal_query(&index, filter, false).await {
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
                    for ev in self.internal_query(&index, filter, false).await {
                        if ev.created_at >= event.created_at {
                            should_insert = false;
                        } else if ev.created_at < event.created_at {
                            to_discard.insert(ev.event_id);
                        }
                    }
                }
                None => should_insert = false,
            }
        } else if event.kind == Kind::EventDeletion {
            let mut deleted = self.deleted.write().await;
            let pubkey_prefix: PublicKeyPrefix = PublicKeyPrefix::from(event.pubkey);

            // Check `e` tags
            let ids = event.event_ids().copied();
            let filter: Filter = Filter::new().ids(ids);
            for ev in self.internal_query(&index, filter, false).await {
                if ev.pubkey == pubkey_prefix {
                    to_discard.insert(ev.event_id);
                    deleted.insert(ev.event_id);
                }
            }

            // Check `a` tags
            for coordinate in event.coordinates() {
                let coordinate_pubkey_prefix: PublicKeyPrefix =
                    PublicKeyPrefix::from(coordinate.pubkey);
                if coordinate_pubkey_prefix == pubkey_prefix {
                    let filter: Filter = coordinate.into();
                    for ev in self.internal_query(&index, filter, false).await {
                        to_discard.insert(ev.event_id);
                        deleted.insert(ev.event_id);
                    }
                }
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
        filter: Filter,
        allow_empty_filter: bool,
    ) -> impl Iterator<Item = &'a EventIndex> {
        let authors: HashSet<PublicKeyPrefix> =
            filter.authors.iter().map(PublicKeyPrefix::from).collect();
        index.iter().filter(move |m| {
            if (filter.is_empty() && allow_empty_filter) || !filter.is_empty() {
                (filter.ids.is_empty() || filter.ids.contains(&m.event_id))
                    && filter.since.map_or(true, |t| m.created_at >= t)
                    && filter.until.map_or(true, |t| m.created_at <= t)
                    && (filter.authors.is_empty() || authors.contains(&m.pubkey))
                    && (filter.kinds.is_empty() || filter.kinds.contains(&m.kind))
                    && m.filter_tags_match(&filter)
            } else {
                false
            }
        })
    }

    /// Query
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn query<I>(&self, filters: I) -> Vec<EventId>
    where
        I: IntoIterator<Item = Filter>,
    {
        let index = self.index.read().await;

        let mut matching_ids: BTreeSet<&EventIndex> = BTreeSet::new();

        for filter in filters.into_iter() {
            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            let limit: Option<usize> = filter.limit;
            let iter = self.internal_query(&index, filter, true).await;
            if let Some(limit) = limit {
                matching_ids.extend(iter.take(limit))
            } else {
                matching_ids.extend(iter)
            }
        }

        matching_ids.into_iter().map(|e| e.event_id).collect()
    }

    /// Count events
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn count<I>(&self, filters: I) -> usize
    where
        I: IntoIterator<Item = Filter>,
    {
        let index = self.index.read().await;

        let mut counter: usize = 0;

        for filter in filters.into_iter() {
            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            let limit: Option<usize> = filter.limit;
            let iter = self.internal_query(&index, filter, true).await;
            if let Some(limit) = limit {
                counter += iter.take(limit).count();
            } else {
                counter += iter.count();
            }
        }

        counter
    }

    /// Check if an event was deleted
    pub async fn has_been_deleted(&self, event_id: &EventId) -> bool {
        let deleted = self.deleted.read().await;
        deleted.contains(event_id)
    }

    /// Clear indexes
    pub async fn clear(&self) {
        let mut index = self.index.write().await;
        index.clear();
    }
}

#[cfg(test)]
mod tests {
    use nostr::nips::nip01::Coordinate;
    use nostr::secp256k1::SecretKey;
    use nostr::{EventBuilder, FromBech32, Keys, Tag};

    use super::*;

    const SECRET_KEY_A: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";
    const SECRET_KEY_B: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";

    #[tokio::test]
    async fn test_database_indexes() {
        let indexes = DatabaseIndexes::new();

        // Keys
        let keys_a = Keys::new(SecretKey::from_bech32(SECRET_KEY_A).unwrap());
        let keys_b = Keys::new(SecretKey::from_bech32(SECRET_KEY_B).unwrap());

        // Build some events
        let events = [
            EventBuilder::new_text_note("Text note", &[])
                .to_event(&keys_a)
                .unwrap(),
            EventBuilder::new(
                Kind::ParameterizedReplaceable(32121),
                "Empty 1",
                &[Tag::Identifier(String::from("abdefgh:12345678"))],
            )
            .to_event(&keys_a)
            .unwrap(),
            EventBuilder::new(
                Kind::ParameterizedReplaceable(32122),
                "Empty 2",
                &[Tag::Identifier(String::from("abdefgh:12345678"))],
            )
            .to_event(&keys_a)
            .unwrap(),
            EventBuilder::new(
                Kind::ParameterizedReplaceable(32122),
                "",
                &[Tag::Identifier(String::from("ijklmnop:87654321"))],
            )
            .to_event(&keys_a)
            .unwrap(),
            EventBuilder::new(
                Kind::ParameterizedReplaceable(32122),
                "",
                &[Tag::Identifier(String::from("abdefgh:87654321"))],
            )
            .to_event(&keys_b)
            .unwrap(),
            EventBuilder::new(
                Kind::ParameterizedReplaceable(32122),
                "",
                &[Tag::Identifier(String::from("abdefgh:12345678"))],
            )
            .to_event(&keys_b)
            .unwrap(),
            EventBuilder::new(
                Kind::ParameterizedReplaceable(32122),
                "Test",
                &[Tag::Identifier(String::from("abdefgh:12345678"))],
            )
            .to_event(&keys_a)
            .unwrap(),
        ];

        for event in events.iter() {
            indexes.index_event(event).await;
        }

        // Total events: 6

        // Invalid event deletion (wrong signing keys)
        let coordinate =
            Coordinate::new(Kind::ParameterizedReplaceable(32122), keys_a.public_key());
        let event = EventBuilder::delete([coordinate])
            .to_event(&keys_b)
            .unwrap();
        indexes.index_event(&event).await;

        // Total events: 7 (6 + 1)

        // Delete valid event
        let coordinate =
            Coordinate::new(Kind::ParameterizedReplaceable(32122), keys_a.public_key())
                .identifier("ijklmnop:87654321");
        let event = EventBuilder::delete([coordinate])
            .to_event(&keys_a)
            .unwrap();
        indexes.index_event(&event).await;

        // Total events: 7 (7 - 1 + 1)

        // Check total number of indexes
        let filter = Filter::new();
        assert_eq!(indexes.count([filter]).await, 7);

        // Check if query len and count match
        assert_eq!(
            indexes.query([Filter::new()]).await.len(),
            indexes.count([Filter::new()]).await
        );
    }
}
