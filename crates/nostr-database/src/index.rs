// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database Indexes

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use nostr::event::id;
use nostr::nips::nip01::Coordinate;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Alphabet, Event, EventId, Filter, GenericTagValue, Kind, Timestamp};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::raw::RawEvent;
use crate::tag_indexes::{TagIndexValues, TagIndexes};
use crate::Order;

/// Public Key Prefix Size
const PUBLIC_KEY_PREFIX_SIZE: usize = 8;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    EventId(#[from] id::Error),
}

type ArcEventId = Arc<EventId>;
type ArcEventIndex = Arc<EventIndex>;
type ArcTagIndexes = Arc<TagIndexes>;
type ParameterizedReplaceableIndexes =
    HashMap<(Kind, PublicKeyPrefix, ArcTagIndexes), ArcEventIndex>;

/// Event Index
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EventIndex {
    /// Timestamp (seconds)
    created_at: Timestamp,
    /// Event ID
    event_id: ArcEventId,
    /// Public key prefix
    pubkey: PublicKeyPrefix,
    /// Kind
    kind: Kind,
    /// Tag indexes
    tags: ArcTagIndexes,
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
            created_at: raw.created_at,
            event_id: Arc::new(EventId::from_slice(&raw.id)?),
            pubkey: PublicKeyPrefix::from(raw.pubkey),
            kind: raw.kind,
            tags: Arc::new(TagIndexes::from(raw.tags.into_iter())),
        })
    }
}

impl From<&Event> for EventIndex {
    fn from(e: &Event) -> Self {
        Self {
            created_at: e.created_at,
            event_id: Arc::new(e.id),
            pubkey: PublicKeyPrefix::from(e.pubkey),
            kind: e.kind,
            tags: Arc::new(TagIndexes::from(e.tags.iter().map(|t| t.as_vec()))),
        }
    }
}

/// Public Key prefix
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PublicKeyPrefix([u8; PUBLIC_KEY_PREFIX_SIZE]);

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

#[derive(Default)]
struct FilterIndex {
    ids: HashSet<EventId>,
    authors: HashSet<PublicKeyPrefix>,
    kinds: HashSet<Kind>,
    since: Option<Timestamp>,
    until: Option<Timestamp>,
    generic_tags: HashMap<Alphabet, HashSet<GenericTagValue>>,
}

impl FilterIndex {
    fn author(mut self, author: PublicKeyPrefix) -> Self {
        self.authors.insert(author);
        self
    }

    fn kind(mut self, kind: Kind) -> Self {
        self.kinds.insert(kind);
        self
    }

    fn identifier<S>(mut self, identifier: S) -> Self
    where
        S: Into<String>,
    {
        let identifier: GenericTagValue = GenericTagValue::String(identifier.into());
        self.generic_tags
            .entry(Alphabet::D)
            .and_modify(|list| {
                list.insert(identifier.clone());
            })
            .or_default()
            .insert(identifier);
        self
    }

    fn ids_match(&self, event: &EventIndex) -> bool {
        self.ids.is_empty() || self.ids.contains(&event.event_id)
    }

    fn authors_match(&self, event: &EventIndex) -> bool {
        self.authors.is_empty() || self.authors.contains(&event.pubkey)
    }

    fn tag_match(&self, event: &EventIndex) -> bool {
        if self.generic_tags.is_empty() {
            return true;
        }

        if event.tags.is_empty() {
            return false;
        }

        self.generic_tags.iter().all(|(tagname, set)| {
            event.tags.get(tagname).map_or(false, |valset| {
                TagIndexValues::iter(set.iter())
                    .filter(|t| valset.contains(t))
                    .count()
                    > 0
            })
        })
    }

    fn kind_match(&self, kind: &Kind) -> bool {
        self.kinds.is_empty() || self.kinds.contains(kind)
    }

    pub fn match_event(&self, event: &EventIndex) -> bool {
        self.ids_match(event)
            && self.since.map_or(true, |t| event.created_at >= t)
            && self.until.map_or(true, |t| event.created_at <= t)
            && self.kind_match(&event.kind)
            && self.authors_match(event)
            && self.tag_match(event)
    }
}

impl From<Filter> for FilterIndex {
    fn from(value: Filter) -> Self {
        Self {
            ids: value.ids,
            authors: value
                .authors
                .into_iter()
                .map(PublicKeyPrefix::from)
                .collect(),
            kinds: value.kinds,
            since: value.since,
            until: value.until,
            generic_tags: value.generic_tags,
        }
    }
}

#[allow(missing_docs)]
pub enum EventOrRawEvent<'a> {
    Event(&'a Event),
    EventOwned(Event),
    Raw(RawEvent),
}

impl<'a> From<Event> for EventOrRawEvent<'a> {
    fn from(value: Event) -> Self {
        Self::EventOwned(value)
    }
}

impl<'a> From<&'a Event> for EventOrRawEvent<'a> {
    fn from(value: &'a Event) -> Self {
        Self::Event(value)
    }
}

impl<'a> From<RawEvent> for EventOrRawEvent<'a> {
    fn from(value: RawEvent) -> Self {
        Self::Raw(value)
    }
}

impl<'a> EventOrRawEvent<'a> {
    fn pubkey(&self) -> PublicKeyPrefix {
        match self {
            Self::Event(e) => PublicKeyPrefix::from(e.pubkey),
            Self::EventOwned(e) => PublicKeyPrefix::from(e.pubkey),
            Self::Raw(r) => PublicKeyPrefix::from(r.pubkey),
        }
    }

    fn created_at(&self) -> Timestamp {
        match self {
            Self::Event(e) => e.created_at,
            Self::EventOwned(e) => e.created_at,
            Self::Raw(r) => r.created_at,
        }
    }

    fn kind(&self) -> Kind {
        match self {
            Self::Event(e) => e.kind,
            Self::EventOwned(e) => e.kind,
            Self::Raw(r) => r.kind,
        }
    }

    fn tags(self) -> TagIndexes {
        match self {
            Self::Event(e) => TagIndexes::from(e.tags.iter().map(|t| t.as_vec())),
            Self::EventOwned(e) => TagIndexes::from(e.tags.iter().map(|t| t.as_vec())),
            Self::Raw(r) => TagIndexes::from(r.tags.into_iter()),
        }
    }

    fn identifier(&self) -> Option<&str> {
        match self {
            Self::Event(e) => e.identifier(),
            Self::EventOwned(e) => e.identifier(),
            Self::Raw(r) => r.identifier(),
        }
    }

    fn event_ids(&self) -> Box<dyn Iterator<Item = EventId> + '_> {
        match self {
            Self::Event(e) => Box::new(e.event_ids().copied()),
            Self::EventOwned(e) => Box::new(e.event_ids().copied()),
            Self::Raw(r) => Box::new(r.event_ids()),
        }
    }

    fn coordinates(&self) -> Box<dyn Iterator<Item = Coordinate> + '_> {
        match self {
            Self::Event(e) => Box::new(e.coordinates()),
            Self::EventOwned(e) => Box::new(e.coordinates()),
            Self::Raw(r) => Box::new(r.coordinates()),
        }
    }
}

enum QueryPattern {
    Replaceable,
    ParamReplaceable,
    Generic,
}

impl From<&Filter> for QueryPattern {
    fn from(filter: &Filter) -> Self {
        let kinds_len = filter.kinds.len();
        let first_kind = filter.kinds.iter().next();
        let authors_len = filter.authors.len();
        let ids_len = filter.ids.len();
        let generic_tags_len = filter.generic_tags.len();

        if kinds_len == 1
            && first_kind.map_or(false, |k| k.is_replaceable())
            && authors_len == 1
            && ids_len == 0
            && generic_tags_len == 0
        {
            Self::Replaceable
        } else if kinds_len == 1
            && first_kind.map_or(false, |k| k.is_parameterized_replaceable())
            && authors_len == 1
            && generic_tags_len != 0
            && ids_len == 0
        {
            Self::ParamReplaceable
        } else {
            Self::Generic
        }
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
    index: Arc<RwLock<BTreeSet<ArcEventIndex>>>,
    /// Event IDs index
    ids_index: Arc<RwLock<HashMap<ArcEventId, ArcEventIndex>>>,
    /// Replaceable index
    kind_author_index: Arc<RwLock<HashMap<(Kind, PublicKeyPrefix), ArcEventIndex>>>,
    /// Param. replaceable index
    kind_author_tags_index: Arc<RwLock<ParameterizedReplaceableIndexes>>,
    deleted_ids: Arc<RwLock<HashSet<ArcEventId>>>,
    deleted_coordinates: Arc<RwLock<HashMap<Coordinate, Timestamp>>>,
}

impl DatabaseIndexes {
    /// New empty indexes
    pub fn new() -> Self {
        Self::default()
    }

    /// Bulk index
    #[tracing::instrument(skip_all)]
    pub async fn bulk_index<'a, E>(&self, events: BTreeSet<E>) -> HashSet<EventId>
    where
        E: Into<EventOrRawEvent<'a>>,
    {
        let mut index = self.index.write().await;
        let mut ids_index = self.ids_index.write().await;
        let mut kind_author_index = self.kind_author_index.write().await;
        let mut kind_author_tags_index = self.kind_author_tags_index.write().await;
        let mut deleted_ids = self.deleted_ids.write().await;
        let mut deleted_coordinates = self.deleted_coordinates.write().await;

        let mut to_discard: HashSet<EventId> = HashSet::new();
        let now: Timestamp = Timestamp::now();

        events
            .into_iter()
            .map(|e| e.into())
            .filter(|e| !e.kind().is_ephemeral())
            .for_each(|event| {
                let res = self.internal_index_event(
                    &mut index,
                    &mut ids_index,
                    &mut kind_author_index,
                    &mut kind_author_tags_index,
                    &mut deleted_ids,
                    &mut deleted_coordinates,
                    event,
                    &now,
                );
                if let Ok(res) = res {
                    to_discard.extend(res.to_discard);
                }
            });

        to_discard
    }

    fn internal_index_event<'a, E>(
        &self,
        index: &mut BTreeSet<ArcEventIndex>,
        ids_index: &mut HashMap<ArcEventId, ArcEventIndex>,
        kind_author_index: &mut HashMap<(Kind, PublicKeyPrefix), ArcEventIndex>,
        kind_author_tags_index: &mut ParameterizedReplaceableIndexes,
        deleted_ids: &mut HashSet<ArcEventId>,
        deleted_coordinates: &mut HashMap<Coordinate, Timestamp>,
        event: E,
        now: &Timestamp,
    ) -> Result<EventIndexResult, Error>
    where
        E: Into<EventOrRawEvent<'a>>,
    {
        let event = event.into();

        // Parse event ID
        let event_id: ArcEventId = match &event {
            EventOrRawEvent::Event(e) => Arc::new(e.id),
            EventOrRawEvent::EventOwned(e) => Arc::new(e.id),
            EventOrRawEvent::Raw(r) => Arc::new(EventId::from_slice(&r.id)?),
        };

        // Check if was deleted
        if deleted_ids.contains(&event_id) {
            return Ok(EventIndexResult {
                to_store: false,
                to_discard: HashSet::new(),
            });
        }

        let mut to_discard: HashSet<ArcEventIndex> = HashSet::new();

        // Check if is expired
        if let EventOrRawEvent::Raw(raw) = &event {
            if raw.is_expired(now) {
                let mut to_discard = HashSet::with_capacity(1);
                to_discard.insert(*event_id);
                return Ok(EventIndexResult {
                    to_store: false,
                    to_discard,
                });
            }
        }

        // Compose others fields
        let pubkey_prefix: PublicKeyPrefix = event.pubkey();
        let created_at: Timestamp = event.created_at();
        let kind: Kind = event.kind();

        let mut should_insert: bool = true;

        if kind.is_replaceable() {
            let filter: FilterIndex = FilterIndex::default().author(pubkey_prefix).kind(kind);
            if let Some(ev) =
                self.internal_query_by_kind_and_author(kind_author_index, deleted_ids, filter)
            {
                if ev.created_at > created_at || ev.event_id == event_id {
                    should_insert = false;
                } else {
                    to_discard.insert(ev.clone());
                }
            }
        } else if kind.is_parameterized_replaceable() {
            match event.identifier() {
                Some(identifier) => {
                    let filter: FilterIndex = FilterIndex::default()
                        .author(pubkey_prefix)
                        .kind(kind)
                        .identifier(identifier);
                    if let Some(ev) = self.internal_query_by_kind_author_tag(
                        kind_author_tags_index,
                        deleted_ids,
                        filter,
                    ) {
                        if ev.created_at > created_at || ev.event_id == event_id {
                            should_insert = false;
                        } else {
                            to_discard.insert(ev.clone());
                        }
                    }
                }
                None => should_insert = false,
            }
        } else if kind == Kind::EventDeletion {
            // Check `e` tags
            for id in event.event_ids() {
                if let Some(ev) = ids_index.get(&Arc::new(id)) {
                    if ev.pubkey == pubkey_prefix && ev.created_at <= created_at {
                        to_discard.insert(ev.clone());
                    }
                }
            }

            // Check `a` tags
            for coordinate in event.coordinates() {
                let coordinate_pubkey_prefix: PublicKeyPrefix =
                    PublicKeyPrefix::from(coordinate.pubkey);
                if coordinate_pubkey_prefix == pubkey_prefix {
                    // Save deleted coordinate at certain timestamp
                    deleted_coordinates.insert(coordinate.clone(), created_at);

                    let filter: Filter = coordinate.into();
                    let filter: Filter = filter.until(created_at);
                    // Not check if ev.pubkey match the pubkey_prefix because asume that query
                    // returned only the events owned by pubkey_prefix
                    to_discard.extend(
                        self.internal_generic_query(index, deleted_ids, filter)
                            .cloned(),
                    );
                }
            }
        }

        // Remove events
        if !to_discard.is_empty() {
            for ev in to_discard.iter() {
                index.remove(ev);
                ids_index.remove(&ev.event_id);

                if ev.kind.is_replaceable() {
                    kind_author_index.remove(&(ev.kind, ev.pubkey));
                } else if ev.kind.is_parameterized_replaceable() {
                    kind_author_tags_index.remove(&(ev.kind, ev.pubkey, ev.tags.clone()));
                }
            }

            deleted_ids.extend(to_discard.iter().map(|ev| ev.event_id.clone()));
        }

        // Insert event
        if should_insert {
            let e: ArcEventIndex = Arc::new(EventIndex {
                created_at,
                event_id: event_id.clone(),
                pubkey: pubkey_prefix,
                kind,
                tags: Arc::new(event.tags()),
            });

            index.insert(e.clone());
            ids_index.insert(event_id, e.clone());

            if kind.is_replaceable() {
                kind_author_index.insert((kind, pubkey_prefix), e);
            } else if kind.is_parameterized_replaceable() {
                kind_author_tags_index.insert((kind, pubkey_prefix, e.tags.clone()), e);
            }
        }

        Ok(EventIndexResult {
            to_store: should_insert,
            to_discard: to_discard.into_iter().map(|ev| *ev.event_id).collect(),
        })
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

        // Acquire write lock
        let mut index = self.index.write().await;
        let mut ids_index = self.ids_index.write().await;
        let mut kind_author_index = self.kind_author_index.write().await;
        let mut kind_author_tags_index = self.kind_author_tags_index.write().await;
        let mut deleted_ids = self.deleted_ids.write().await;
        let mut deleted_coordinates = self.deleted_coordinates.write().await;

        let now = Timestamp::now();

        self.internal_index_event(
            &mut index,
            &mut ids_index,
            &mut kind_author_index,
            &mut kind_author_tags_index,
            &mut deleted_ids,
            &mut deleted_coordinates,
            event,
            &now,
        )
        .unwrap_or_default()
    }

    /// Query by [`Kind`] and [`PublicKeyPrefix`] (replaceable)
    fn internal_query_by_kind_and_author<'a, T>(
        &self,
        kind_author_index: &'a HashMap<(Kind, PublicKeyPrefix), ArcEventIndex>,
        deleted_ids: &'a HashSet<ArcEventId>,
        filter: T,
    ) -> Option<&'a ArcEventIndex>
    where
        T: Into<FilterIndex>,
    {
        let FilterIndex {
            authors,
            kinds,
            since,
            until,
            ..
        } = filter.into();

        let kind = kinds.iter().next()?;
        let author = authors.iter().next()?;

        if !kind.is_replaceable() {
            return None;
        }

        let ev = kind_author_index.get(&(*kind, *author))?;

        if deleted_ids.contains(&ev.event_id) {
            return None;
        }

        if let Some(since) = since {
            if ev.created_at < since {
                return None;
            }
        }

        if let Some(until) = until {
            if ev.created_at > until {
                return None;
            }
        }

        Some(ev)
    }

    /// Query by [`Kind`], [`PublicKeyPrefix`] and [`TagIndexes`] (param. replaceable)
    fn internal_query_by_kind_author_tag<'a, T>(
        &self,
        kind_author_tag_index: &'a ParameterizedReplaceableIndexes,
        deleted_ids: &'a HashSet<ArcEventId>,
        filter: T,
    ) -> Option<&'a ArcEventIndex>
    where
        T: Into<FilterIndex>,
    {
        let FilterIndex {
            authors,
            kinds,
            since,
            until,
            generic_tags,
            ..
        } = filter.into();

        let kind = kinds.iter().next()?;
        let author = authors.iter().next()?;

        if !kind.is_parameterized_replaceable() {
            return None;
        }

        let tags = {
            let mut tag_index: TagIndexes = TagIndexes::default();
            for (tagnamechar, set) in generic_tags.into_iter() {
                for inner in TagIndexValues::iter(set.iter()) {
                    tag_index.entry(tagnamechar).or_default().insert(inner);
                }
            }
            Arc::new(tag_index)
        };

        let ev = kind_author_tag_index.get(&(*kind, *author, tags))?;

        if deleted_ids.contains(&ev.event_id) {
            return None;
        }

        if let Some(since) = since {
            if ev.created_at < since {
                return None;
            }
        }

        if let Some(until) = until {
            if ev.created_at > until {
                return None;
            }
        }

        Some(ev)
    }

    /// Generic query
    fn internal_generic_query<'a, T>(
        &self,
        index: &'a BTreeSet<ArcEventIndex>,
        deleted_ids: &'a HashSet<ArcEventId>,
        filter: T,
    ) -> impl Iterator<Item = &'a ArcEventIndex>
    where
        T: Into<FilterIndex>,
    {
        let filter: FilterIndex = filter.into();
        index.iter().filter(move |event| {
            !deleted_ids.contains(&event.event_id) && filter.match_event(event)
        })
    }

    /// Query
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn query<I>(&self, filters: I, order: Order) -> Vec<EventId>
    where
        I: IntoIterator<Item = Filter>,
    {
        let index = self.index.read().await;
        let kind_author_index = self.kind_author_index.read().await;
        let kind_author_tags_index = self.kind_author_tags_index.read().await;
        let deleted_ids = self.deleted_ids.read().await;

        let mut matching_ids: BTreeSet<&ArcEventIndex> = BTreeSet::new();

        for filter in filters.into_iter() {
            if filter.is_empty() {
                return match order {
                    Order::Asc => index.iter().map(|e| *e.event_id).rev().collect(),
                    Order::Desc => index.iter().map(|e| *e.event_id).collect(),
                };
            }

            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            match QueryPattern::from(&filter) {
                QueryPattern::Replaceable => {
                    if let Some(ev) = self.internal_query_by_kind_and_author(
                        &kind_author_index,
                        &deleted_ids,
                        filter,
                    ) {
                        matching_ids.insert(ev);
                    };
                }
                QueryPattern::ParamReplaceable => {
                    if let Some(ev) = self.internal_query_by_kind_author_tag(
                        &kind_author_tags_index,
                        &deleted_ids,
                        filter,
                    ) {
                        matching_ids.insert(ev);
                    };
                }
                QueryPattern::Generic => {
                    if let Some(limit) = filter.limit {
                        matching_ids.extend(
                            self.internal_generic_query(&index, &deleted_ids, filter)
                                .take(limit),
                        )
                    } else {
                        matching_ids.extend(self.internal_generic_query(
                            &index,
                            &deleted_ids,
                            filter,
                        ))
                    }
                }
            }
        }

        match order {
            Order::Asc => matching_ids
                .into_iter()
                .map(|ev| *ev.event_id)
                .rev()
                .collect(),
            Order::Desc => matching_ids.into_iter().map(|ev| *ev.event_id).collect(),
        }
    }

    /// Count events
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn count<I>(&self, filters: I) -> usize
    where
        I: IntoIterator<Item = Filter>,
    {
        let index = self.index.read().await;
        let deleted_ids = self.deleted_ids.read().await;

        let mut counter: usize = 0;

        for filter in filters.into_iter() {
            if filter.is_empty() {
                counter = index.len();
                break;
            }

            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            let limit: Option<usize> = filter.limit;
            let count = self
                .internal_generic_query(&index, &deleted_ids, filter)
                .count();
            if let Some(limit) = limit {
                let count = if limit >= count { limit } else { count };
                counter += count;
            } else {
                counter += count;
            }
        }

        counter
    }

    /// Check if an event with [`EventId`] has been deleted
    pub async fn has_event_id_been_deleted(&self, event_id: &EventId) -> bool {
        let deleted_ids = self.deleted_ids.read().await;
        deleted_ids.contains(event_id)
    }

    /// Check if event with [`Coordinate`] has been deleted before [`Timestamp`]
    pub async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: Timestamp,
    ) -> bool {
        let deleted_coordinates = self.deleted_coordinates.read().await;
        if let Some(t) = deleted_coordinates.get(coordinate).copied() {
            t >= timestamp
        } else {
            false
        }
    }

    /// Clear indexes
    pub async fn clear(&self) {
        let mut index = self.index.write().await;
        let mut deleted_ids = self.deleted_ids.write().await;
        let mut deleted_coordinates = self.deleted_coordinates.write().await;
        index.clear();
        deleted_ids.clear();
        deleted_coordinates.clear();
    }
}

#[cfg(test)]
mod tests {
    use nostr::secp256k1::SecretKey;
    use nostr::{FromBech32, JsonUtil, Keys};

    use super::*;

    const SECRET_KEY_A: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99"; // aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4
    const SECRET_KEY_B: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85"; // 79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3

    const EVENTS: [&str; 13] = [
        r#"{"id":"b7b1fb52ad8461a03e949820ae29a9ea07e35bcd79c95c4b59b0254944f62805","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644581,"kind":1,"tags":[],"content":"Text note","sig":"ed73a8a4e7c26cd797a7b875c634d9ecb6958c57733305fed23b978109d0411d21b3e182cb67c8ad750884e30ca383b509382ae6187b36e76ee76e6a142c4284"}"#,
        r#"{"id":"7296747d91c53f1d71778ef3e12d18b66d494a41f688ef244d518abf37c959b6","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644586,"kind":32121,"tags":[["d","id-1"]],"content":"Empty 1","sig":"8848989a8e808f7315e950f871b231c1dff7752048f8957d4a541881d2005506c30e85c7dd74dab022b3e01329c88e69c9d5d55d961759272a738d150b7dbefc"}"#,
        r#"{"id":"ec6ea04ba483871062d79f78927df7979f67545b53f552e47626cb1105590442","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644591,"kind":32122,"tags":[["d","id-1"]],"content":"Empty 2","sig":"89946113a97484850fe35fefdb9120df847b305de1216dae566616fe453565e8707a4da7e68843b560fa22a932f81fc8db2b5a2acb4dcfd3caba9a91320aac92"}"#,
        r#"{"id":"63b8b829aa31a2de870c3a713541658fcc0187be93af2032ec2ca039befd3f70","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644596,"kind":32122,"tags":[["d","id-2"]],"content":"","sig":"607b1a67bef57e48d17df4e145718d10b9df51831d1272c149f2ab5ad4993ae723f10a81be2403ae21b2793c8ed4c129e8b031e8b240c6c90c9e6d32f62d26ff"}"#,
        r#"{"id":"6fe9119c7db13ae13e8ecfcdd2e5bf98e2940ba56a2ce0c3e8fba3d88cd8e69d","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644601,"kind":32122,"tags":[["d","id-3"]],"content":"","sig":"d07146547a726fc9b4ec8d67bbbe690347d43dadfe5d9890a428626d38c617c52e6945f2b7144c4e0c51d1e2b0be020614a5cadc9c0256b2e28069b70d9fc26e"}"#,
        r#"{"id":"a82f6ebfc709f4e7c7971e6bf738e30a3bc112cfdb21336054711e6779fd49ef","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644606,"kind":32122,"tags":[["d","id-1"]],"content":"","sig":"96d3349b42ed637712b4d07f037457ab6e9180d58857df77eb5fa27ff1fd68445c72122ec53870831ada8a4d9a0b484435f80d3ff21a862238da7a723a0d073c"}"#,
        r#"{"id":"8ab0cb1beceeb68f080ec11a3920b8cc491ecc7ec5250405e88691d733185832","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644611,"kind":32122,"tags":[["d","id-1"]],"content":"Test","sig":"49153b482d7110e2538eb48005f1149622247479b1c0057d902df931d5cea105869deeae908e4e3b903e3140632dc780b3f10344805eab77bb54fb79c4e4359d"}"#,
        r#"{"id":"63dc49a8f3278a2de8dc0138939de56d392b8eb7a18c627e4d78789e2b0b09f2","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644616,"kind":5,"tags":[["a","32122:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:"]],"content":"","sig":"977e54e5d57d1fbb83615d3a870037d9eb5182a679ca8357523bbf032580689cf481f76c88c7027034cfaf567ba9d9fe25fc8cd334139a0117ad5cf9fe325eef"}"#,
        r#"{"id":"6975ace0f3d66967f330d4758fbbf45517d41130e2639b54ca5142f37757c9eb","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644621,"kind":5,"tags":[["a","32122:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:id-2"]],"content":"","sig":"9bb09e4759899d86e447c3fa1be83905fe2eda74a5068a909965ac14fcdabaed64edaeb732154dab734ca41f2fc4d63687870e6f8e56e3d9e180e4a2dd6fb2d2"}"#,
        r#"{"id":"33f5b4e6a38e107638c20f4536db35191d4b8651ba5a2cefec983b9ec2d65084","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704645586,"kind":0,"tags":[],"content":"{\"name\":\"Key A\"}","sig":"285d090f45a6adcae717b33771149f7840a8c27fb29025d63f1ab8d95614034a54e9f4f29cee9527c4c93321a7ebff287387b7a19ba8e6f764512a40e7120429"}"#,
        r#"{"id":"90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704645606,"kind":0,"tags":[],"content":"{\"name\":\"key-a\",\"display_name\":\"Key A\",\"lud16\":\"keya@ln.address\"}","sig":"ec8f49d4c722b7ccae102d49befff08e62db775e5da43ef51b25c47dfdd6a09dc7519310a3a63cbdb6ec6b3250e6f19518eb47be604edeb598d16cdc071d3dbc"}"#,
        r#"{"id":"a295422c636d3532875b75739e8dae3cdb4dd2679c6e4994c9a39c7ebf8bc620","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704646569,"kind":5,"tags":[["e","90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9"]],"content":"","sig":"d4dc8368a4ad27eef63cacf667345aadd9617001537497108234fc1686d546c949cbb58e007a4d4b632c65ea135af4fbd7a089cc60ab89b6901f5c3fc6a47b29"}"#,
        r#"{"id":"999e3e270100d7e1eaa98fcfab4a98274872c1f2dfdab024f32e42a5a12d5b5e","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704646606,"kind":5,"tags":[["e","90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9"]],"content":"","sig":"4f3a33fd52784cea7ca8428fd35d94d65049712e9aa11a70b1a16a1fcd761c7b7e27afac325728b1c00dfa11e33e78b2efd0430a7e4b28f4ede5b579b3f32614"}"#,
    ];

    #[tokio::test]
    async fn test_database_indexes() {
        // Keys
        let keys_a = Keys::new(SecretKey::from_bech32(SECRET_KEY_A).unwrap());
        let keys_b = Keys::new(SecretKey::from_bech32(SECRET_KEY_B).unwrap());

        let indexes = DatabaseIndexes::new();

        // Build indexes
        let mut events: BTreeSet<RawEvent> = BTreeSet::new();
        for event in EVENTS.into_iter() {
            let event = Event::from_json(event).unwrap();
            let raw: RawEvent = event.into();
            events.insert(raw);
        }
        indexes.bulk_index(events).await;

        // Test expected output
        let expected_output = vec![
            Event::from_json(EVENTS[12]).unwrap().id,
            Event::from_json(EVENTS[11]).unwrap().id,
            // Event 10 deleted by event 12
            // Event 9 replaced by event 10
            Event::from_json(EVENTS[8]).unwrap().id,
            Event::from_json(EVENTS[7]).unwrap().id,
            Event::from_json(EVENTS[6]).unwrap().id,
            Event::from_json(EVENTS[5]).unwrap().id,
            Event::from_json(EVENTS[4]).unwrap().id,
            // Event 3 deleted by Event 8
            // Event 2 replaced by Event 6
            Event::from_json(EVENTS[1]).unwrap().id,
            Event::from_json(EVENTS[0]).unwrap().id,
        ];
        assert_eq!(
            indexes.query([Filter::new()], Order::Desc).await,
            expected_output
        );
        assert_eq!(indexes.count([Filter::new()]).await, 9);

        // Test get previously deleted replaceable event (check if was deleted by indexes)
        assert!(indexes
            .query(
                [Filter::new()
                    .kind(Kind::Metadata)
                    .author(keys_a.public_key())],
                Order::Desc
            )
            .await
            .is_empty());

        // Test get previously deleted param. replaceable event (check if was deleted by indexes)
        assert!(indexes
            .query(
                [Filter::new()
                    .kind(Kind::ParameterizedReplaceable(32122))
                    .author(keys_a.public_key())
                    .identifier("id-2")],
                Order::Desc
            )
            .await
            .is_empty());

        // Test get param replaceable events WITHOUT using indexes (identifier not passed)
        // Test ascending order
        assert_eq!(
            indexes
                .query(
                    [Filter::new()
                        .kind(Kind::ParameterizedReplaceable(32122))
                        .author(keys_b.public_key())],
                    Order::Asc
                )
                .await,
            vec![
                Event::from_json(EVENTS[4]).unwrap().id,
                Event::from_json(EVENTS[5]).unwrap().id,
            ]
        );

        // Test get param replaceable events using indexes
        assert_eq!(
            indexes
                .query(
                    [Filter::new()
                        .kind(Kind::ParameterizedReplaceable(32122))
                        .author(keys_b.public_key())
                        .identifier("id-3")],
                    Order::Desc
                )
                .await,
            vec![Event::from_json(EVENTS[4]).unwrap().id,]
        );

        assert_eq!(
            indexes
                .query([Filter::new().author(keys_a.public_key())], Order::Desc)
                .await,
            vec![
                Event::from_json(EVENTS[12]).unwrap().id,
                Event::from_json(EVENTS[8]).unwrap().id,
                Event::from_json(EVENTS[6]).unwrap().id,
                Event::from_json(EVENTS[1]).unwrap().id,
                Event::from_json(EVENTS[0]).unwrap().id,
            ]
        );

        assert_eq!(
            indexes
                .query(
                    [Filter::new()
                        .author(keys_a.public_key())
                        .kinds([Kind::TextNote, Kind::Custom(32121)])],
                    Order::Desc
                )
                .await,
            vec![
                Event::from_json(EVENTS[1]).unwrap().id,
                Event::from_json(EVENTS[0]).unwrap().id,
            ]
        );

        assert_eq!(
            indexes
                .query(
                    [Filter::new()
                        .authors([keys_a.public_key(), keys_b.public_key()])
                        .kinds([Kind::TextNote, Kind::Custom(32121)])],
                    Order::Desc
                )
                .await,
            vec![
                Event::from_json(EVENTS[1]).unwrap().id,
                Event::from_json(EVENTS[0]).unwrap().id,
            ]
        );

        // Test get param replaceable events using identifier
        assert_eq!(
            indexes
                .query([Filter::new().identifier("id-1")], Order::Desc)
                .await,
            vec![
                Event::from_json(EVENTS[6]).unwrap().id,
                Event::from_json(EVENTS[5]).unwrap().id,
                Event::from_json(EVENTS[1]).unwrap().id,
            ]
        );
    }
}
