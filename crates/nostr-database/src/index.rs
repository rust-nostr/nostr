// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database Indexes

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter;
use std::sync::Arc;

use nostr::event::id;
use nostr::nips::nip01::Coordinate;
use nostr::{Alphabet, Event, EventId, Filter, Kind, PublicKey, SingleLetterTag, Timestamp};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::tag_indexes::{hash, TagIndexValues, TagIndexes, TAG_INDEX_VALUE_SIZE};
#[cfg(feature = "flatbuf")]
use crate::temp::TempEvent;
use crate::Order;

/// Public Key Prefix Size
const PUBLIC_KEY_PREFIX_SIZE: usize = 8;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    EventId(#[from] id::Error),
}

type ArcEventIndex = Arc<EventIndex>;

/// Event Index
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl From<&Event> for EventIndex {
    fn from(e: &Event) -> Self {
        Self {
            created_at: e.created_at(),
            event_id: e.id(),
            pubkey: PublicKeyPrefix::from(e.author_ref()),
            kind: e.kind(),
            tags: TagIndexes::from(e.iter_tags()),
        }
    }
}

/// Public Key prefix
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PublicKeyPrefix([u8; PUBLIC_KEY_PREFIX_SIZE]);

impl From<&PublicKey> for PublicKeyPrefix {
    fn from(pk: &PublicKey) -> Self {
        let pk: [u8; 32] = pk.serialize();
        Self::from(pk)
    }
}

impl From<PublicKey> for PublicKeyPrefix {
    fn from(pk: PublicKey) -> Self {
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
    generic_tags: HashMap<SingleLetterTag, HashSet<String>>,
}

impl FilterIndex {
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
            ids: value.ids.unwrap_or_default(),
            authors: value
                .authors
                .unwrap_or_default()
                .into_iter()
                .map(PublicKeyPrefix::from)
                .collect(),
            kinds: value.kinds.unwrap_or_default(),
            since: value.since,
            until: value.until,
            generic_tags: value.generic_tags,
        }
    }
}

#[allow(missing_docs)]
pub enum EventOrTempEvent<'a> {
    Event(&'a Event),
    EventOwned(Box<Event>),
    #[cfg(feature = "flatbuf")]
    Temp(TempEvent),
}

impl<'a> From<Event> for EventOrTempEvent<'a> {
    fn from(value: Event) -> Self {
        Self::EventOwned(Box::new(value))
    }
}

impl<'a> From<&'a Event> for EventOrTempEvent<'a> {
    fn from(value: &'a Event) -> Self {
        Self::Event(value)
    }
}

#[cfg(feature = "flatbuf")]
impl<'a> From<TempEvent> for EventOrTempEvent<'a> {
    fn from(value: TempEvent) -> Self {
        Self::Temp(value)
    }
}

impl<'a> EventOrTempEvent<'a> {
    fn id(&self) -> Result<EventId, Error> {
        match self {
            EventOrTempEvent::Event(e) => Ok(e.id()),
            EventOrTempEvent::EventOwned(e) => Ok(e.id()),
            #[cfg(feature = "flatbuf")]
            EventOrTempEvent::Temp(r) => Ok(EventId::from_slice(&r.id)?),
        }
    }

    fn pubkey(&self) -> PublicKeyPrefix {
        match self {
            Self::Event(e) => PublicKeyPrefix::from(e.author_ref()),
            Self::EventOwned(e) => PublicKeyPrefix::from(e.author_ref()),
            #[cfg(feature = "flatbuf")]
            Self::Temp(r) => PublicKeyPrefix::from(r.pubkey),
        }
    }

    fn created_at(&self) -> Timestamp {
        match self {
            Self::Event(e) => e.created_at(),
            Self::EventOwned(e) => e.created_at(),
            #[cfg(feature = "flatbuf")]
            Self::Temp(r) => r.created_at,
        }
    }

    fn kind(&self) -> Kind {
        match self {
            Self::Event(e) => e.kind(),
            Self::EventOwned(e) => e.kind(),
            #[cfg(feature = "flatbuf")]
            Self::Temp(r) => r.kind,
        }
    }

    fn tags(self) -> TagIndexes {
        match self {
            Self::Event(e) => TagIndexes::from(e.iter_tags()),
            Self::EventOwned(e) => TagIndexes::from(e.iter_tags()),
            #[cfg(feature = "flatbuf")]
            Self::Temp(r) => r.tags,
        }
    }

    fn identifier(&self) -> Option<[u8; TAG_INDEX_VALUE_SIZE]> {
        match self {
            Self::Event(e) => e.identifier().map(hash),
            Self::EventOwned(e) => e.identifier().map(hash),
            #[cfg(feature = "flatbuf")]
            Self::Temp(r) => r.identifier,
        }
    }

    fn event_ids(&self) -> Box<dyn Iterator<Item = &EventId> + '_> {
        match self {
            Self::Event(e) => Box::new(e.event_ids()),
            Self::EventOwned(e) => Box::new(e.event_ids()),
            #[cfg(feature = "flatbuf")]
            Self::Temp(r) => Box::new(r.event_ids.iter()),
        }
    }

    fn coordinates(&self) -> Box<dyn Iterator<Item = &Coordinate> + '_> {
        match self {
            Self::Event(e) => Box::new(e.coordinates()),
            Self::EventOwned(e) => Box::new(e.coordinates()),
            #[cfg(feature = "flatbuf")]
            Self::Temp(r) => Box::new(r.coordinates.iter()),
        }
    }

    fn is_expired(&self, now: &Timestamp) -> bool {
        match self {
            Self::Event(e) => e.is_expired_at(now),
            Self::EventOwned(e) => e.is_expired_at(now),
            #[cfg(feature = "flatbuf")]
            Self::Temp(r) => r.is_expired(now),
        }
    }
}

struct QueryByAuthorParams {
    author: PublicKeyPrefix,
    since: Option<Timestamp>,
    until: Option<Timestamp>,
}

struct QueryByKindAndAuthorParams {
    kind: Kind,
    author: PublicKeyPrefix,
    since: Option<Timestamp>,
    until: Option<Timestamp>,
}

impl QueryByKindAndAuthorParams {
    pub fn new(kind: Kind, author: PublicKeyPrefix) -> Self {
        Self {
            kind,
            author,
            since: None,
            until: None,
        }
    }
}

struct QueryByParamReplaceable {
    kind: Kind,
    author: PublicKeyPrefix,
    identifier: [u8; TAG_INDEX_VALUE_SIZE],
    since: Option<Timestamp>,
    until: Option<Timestamp>,
}

impl QueryByParamReplaceable {
    pub fn new(
        kind: Kind,
        author: PublicKeyPrefix,
        identifier: [u8; TAG_INDEX_VALUE_SIZE],
    ) -> Self {
        Self {
            kind,
            author,
            identifier,
            since: None,
            until: None,
        }
    }
}

enum QueryPattern {
    Author(QueryByAuthorParams),
    KindAuthor(QueryByKindAndAuthorParams),
    ParamReplaceable(QueryByParamReplaceable),
    Generic(Box<Filter>),
}

impl From<Filter> for QueryPattern {
    fn from(filter: Filter) -> Self {
        let (kinds_len, first_kind): (usize, Option<Kind>) = filter
            .kinds
            .as_ref()
            .map(|set| (set.len(), set.iter().next().copied()))
            .unwrap_or_default();
        let (authors_len, first_author): (usize, Option<PublicKey>) = filter
            .authors
            .as_ref()
            .map(|set| (set.len(), set.iter().next().copied()))
            .unwrap_or_default();
        let ids_len: usize = filter.ids.as_ref().map(|set| set.len()).unwrap_or_default();
        let generic_tags_len: usize = filter.generic_tags.len();
        let identifier = filter
            .generic_tags
            .get(&SingleLetterTag::lowercase(Alphabet::D))
            .and_then(|v| v.iter().next().map(hash));

        match (
            kinds_len,
            first_kind,
            authors_len,
            first_author,
            ids_len,
            generic_tags_len,
            identifier,
        ) {
            (0, None, 1, Some(author), 0, 0, None) => Self::Author(QueryByAuthorParams {
                author: PublicKeyPrefix::from(author),
                since: filter.since,
                until: filter.until,
            }),
            (1, Some(kind), 1, Some(author), 0, 0, None) => {
                Self::KindAuthor(QueryByKindAndAuthorParams {
                    kind,
                    author: PublicKeyPrefix::from(author),
                    since: filter.since,
                    until: filter.until,
                })
            }
            (1, Some(kind), 1, Some(author), 0, _, Some(identifier))
                if kind.is_parameterized_replaceable() =>
            {
                Self::ParamReplaceable(QueryByParamReplaceable {
                    kind,
                    author: PublicKeyPrefix::from(author),
                    identifier,
                    since: filter.since,
                    until: filter.until,
                })
            }
            _ => Self::Generic(Box::new(filter)),
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

enum InternalQueryResult<'a> {
    All,
    Set(BTreeSet<&'a ArcEventIndex>),
}

/// Database Indexes
#[derive(Debug, Clone, Default)]
struct InternalDatabaseIndexes {
    index: BTreeSet<ArcEventIndex>,
    ids_index: HashMap<EventId, ArcEventIndex>,
    author_index: HashMap<PublicKeyPrefix, BTreeSet<ArcEventIndex>>,
    kind_author_index: HashMap<(Kind, PublicKeyPrefix), BTreeSet<ArcEventIndex>>,
    kind_author_tags_index:
        HashMap<(Kind, PublicKeyPrefix, [u8; TAG_INDEX_VALUE_SIZE]), ArcEventIndex>,
    deleted_ids: HashSet<EventId>,
    deleted_coordinates: HashMap<Coordinate, Timestamp>,
}

impl InternalDatabaseIndexes {
    /// Bulk index
    #[tracing::instrument(skip_all)]
    pub fn bulk_index<'a, E>(&mut self, events: BTreeSet<E>) -> HashSet<EventId>
    where
        E: Into<EventOrTempEvent<'a>>,
    {
        let now: Timestamp = Timestamp::now();
        events
            .into_iter()
            .map(|e| e.into())
            .filter(|e| !e.kind().is_ephemeral())
            .filter_map(|event| self.internal_index_event(event, &now).ok())
            .flat_map(|res| res.to_discard)
            .collect()
    }

    /// Bulk import
    #[tracing::instrument(skip_all)]
    pub fn bulk_import<'a>(
        &'a mut self,
        events: BTreeSet<Event>,
    ) -> impl Iterator<Item = Event> + 'a {
        let now: Timestamp = Timestamp::now();
        events
            .into_iter()
            .filter(|e| !e.is_expired() && !e.is_ephemeral())
            .filter(move |event| match self.internal_index_event(event, &now) {
                Ok(res) => res.to_store,
                Err(_) => false,
            })
    }

    fn internal_index_event<'a, E>(
        &mut self,
        event: E,
        now: &Timestamp,
    ) -> Result<EventIndexResult, Error>
    where
        E: Into<EventOrTempEvent<'a>>,
    {
        let event = event.into();
        let event_id: EventId = event.id()?;

        // Check if was already added
        if self.ids_index.contains_key(&event_id) {
            return Ok(EventIndexResult::default());
        }

        // Check if was deleted or is expired
        if self.deleted_ids.contains(&event_id) || event.is_expired(now) {
            let mut to_discard: HashSet<EventId> = HashSet::with_capacity(1);
            to_discard.insert(event_id);
            return Ok(EventIndexResult {
                to_store: false,
                to_discard,
            });
        }

        let mut to_discard: HashSet<EventId> = HashSet::new();

        // Compose others fields
        let pubkey_prefix: PublicKeyPrefix = event.pubkey();
        let created_at: Timestamp = event.created_at();
        let kind: Kind = event.kind();

        let mut should_insert: bool = true;

        if kind.is_replaceable() {
            let params: QueryByKindAndAuthorParams =
                QueryByKindAndAuthorParams::new(kind, pubkey_prefix);
            for ev in self.internal_query_by_kind_and_author(params) {
                if ev.created_at > created_at || ev.event_id == event_id {
                    should_insert = false;
                } else {
                    to_discard.insert(ev.event_id);
                }
            }
        } else if kind.is_parameterized_replaceable() {
            match event.identifier() {
                Some(identifier) => {
                    // TODO: check if coordinate was deleted
                    let params: QueryByParamReplaceable =
                        QueryByParamReplaceable::new(kind, pubkey_prefix, identifier);
                    if let Some(ev) = self.internal_query_param_replaceable(params) {
                        if ev.created_at > created_at || ev.event_id == event_id {
                            should_insert = false;
                        } else {
                            to_discard.insert(ev.event_id);
                        }
                    }
                }
                None => should_insert = false,
            }
        } else if kind == Kind::EventDeletion {
            // Check `e` tags
            for id in event.event_ids() {
                if let Some(ev) = self.ids_index.get(id) {
                    if ev.pubkey == pubkey_prefix && ev.created_at <= created_at {
                        to_discard.insert(ev.event_id);
                    }
                }
            }

            // Check `a` tags
            for coordinate in event.coordinates() {
                let coordinate_pubkey_prefix: PublicKeyPrefix =
                    PublicKeyPrefix::from(coordinate.public_key);
                if coordinate_pubkey_prefix == pubkey_prefix {
                    // Save deleted coordinate at certain timestamp
                    self.deleted_coordinates
                        .insert(coordinate.clone(), created_at);

                    let filter: Filter = coordinate.into();
                    let filter: Filter = filter.until(created_at);
                    // Not check if ev.pubkey match the pubkey_prefix because assume that query
                    // returned only the events owned by pubkey_prefix
                    to_discard.extend(self.internal_generic_query(filter).map(|e| e.event_id));
                }
            }
        }

        // Remove events
        self.discard_events(&to_discard);

        // Insert event
        if should_insert {
            let e: ArcEventIndex = Arc::new(EventIndex {
                created_at,
                event_id,
                pubkey: pubkey_prefix,
                kind,
                tags: event.tags(),
            });

            self.index.insert(e.clone());
            self.ids_index.insert(event_id, e.clone());
            self.author_index
                .entry(pubkey_prefix)
                .or_default()
                .insert(e.clone());

            if kind.is_parameterized_replaceable() {
                if let Some(identifier) = e.tags.identifier() {
                    self.kind_author_tags_index
                        .insert((kind, pubkey_prefix, identifier), e.clone());
                }
            }

            if kind.is_replaceable() {
                let mut set = BTreeSet::new();
                set.insert(e);
                self.kind_author_index.insert((kind, pubkey_prefix), set);
            } else {
                self.kind_author_index
                    .entry((kind, pubkey_prefix))
                    .or_default()
                    .insert(e);
            }
        }

        Ok(EventIndexResult {
            to_store: should_insert,
            to_discard,
        })
    }

    fn discard_events(&mut self, ids: &HashSet<EventId>) {
        if !ids.is_empty() {
            for id in ids.iter() {
                if let Some(ev) = self.ids_index.remove(id) {
                    self.index.remove(&ev);

                    if let Some(set) = self.author_index.get_mut(&ev.pubkey) {
                        set.remove(&ev);
                    }

                    if ev.kind.is_parameterized_replaceable() {
                        if let Some(identifier) = ev.tags.identifier() {
                            self.kind_author_tags_index
                                .remove(&(ev.kind, ev.pubkey, identifier));
                        }
                    }

                    if let Some(set) = self.kind_author_index.get_mut(&(ev.kind, ev.pubkey)) {
                        set.remove(&ev);
                    }
                }
                self.deleted_ids.insert(*id);
            }
        }
    }

    /// Index [`Event`]
    ///
    /// **This method assume that [`Event`] was already verified**
    #[tracing::instrument(skip_all, level = "trace")]
    pub fn index_event(&mut self, event: &Event) -> EventIndexResult {
        // Check if it's expired or ephemeral (in `internal_index_event` is checked only the raw event expiration)
        if event.is_expired() || event.is_ephemeral() {
            return EventIndexResult::default();
        }
        let now = Timestamp::now();
        self.internal_index_event(event, &now).unwrap_or_default()
    }

    /// Query by public key
    fn internal_query_by_author<'a>(
        &'a self,
        params: QueryByAuthorParams,
    ) -> Box<dyn Iterator<Item = &'a ArcEventIndex> + 'a> {
        let QueryByAuthorParams {
            author,
            since,
            until,
        } = params;
        match self.author_index.get(&author) {
            Some(set) => Box::new(set.iter().filter(move |ev| {
                if self.deleted_ids.contains(&ev.event_id) {
                    return false;
                }

                if let Some(since) = since {
                    if ev.created_at < since {
                        return false;
                    }
                }

                if let Some(until) = until {
                    if ev.created_at > until {
                        return false;
                    }
                }

                true
            })),
            None => Box::new(iter::empty()),
        }
    }

    /// Query by [`Kind`] and [`PublicKeyPrefix`]
    fn internal_query_by_kind_and_author<'a>(
        &'a self,
        params: QueryByKindAndAuthorParams,
    ) -> Box<dyn Iterator<Item = &'a ArcEventIndex> + 'a> {
        let QueryByKindAndAuthorParams {
            kind,
            author,
            since,
            until,
        } = params;
        match self.kind_author_index.get(&(kind, author)) {
            Some(set) => Box::new(set.iter().filter(move |ev| {
                if self.deleted_ids.contains(&ev.event_id) {
                    return false;
                }

                if let Some(since) = since {
                    if ev.created_at < since {
                        return false;
                    }
                }

                if let Some(until) = until {
                    if ev.created_at > until {
                        return false;
                    }
                }

                true
            })),
            None => Box::new(iter::empty()),
        }
    }

    /// Query by param. replaceable
    fn internal_query_param_replaceable(
        &self,
        params: QueryByParamReplaceable,
    ) -> Option<&ArcEventIndex> {
        let QueryByParamReplaceable {
            kind,
            author,
            identifier,
            since,
            until,
        } = params;

        if !kind.is_parameterized_replaceable() {
            return None;
        }

        let ev = self
            .kind_author_tags_index
            .get(&(kind, author, identifier))?;

        if self.deleted_ids.contains(&ev.event_id) {
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
    fn internal_generic_query<T>(&self, filter: T) -> impl Iterator<Item = &ArcEventIndex>
    where
        T: Into<FilterIndex>,
    {
        let filter: FilterIndex = filter.into();
        self.index.iter().filter(move |event| {
            !self.deleted_ids.contains(&event.event_id) && filter.match_event(event)
        })
    }

    fn internal_query<I>(&self, filters: I) -> InternalQueryResult
    where
        I: IntoIterator<Item = Filter>,
    {
        let mut matching_ids: BTreeSet<&ArcEventIndex> = BTreeSet::new();

        for filter in filters.into_iter() {
            if filter.is_empty() {
                return InternalQueryResult::All;
            }

            if let (Some(since), Some(until)) = (filter.since, filter.until) {
                if since > until {
                    continue;
                }
            }

            let limit: Option<usize> = filter.limit;

            let evs: Box<dyn Iterator<Item = &ArcEventIndex>> = match QueryPattern::from(filter) {
                QueryPattern::Author(params) => self.internal_query_by_author(params),
                QueryPattern::KindAuthor(params) => self.internal_query_by_kind_and_author(params),
                QueryPattern::ParamReplaceable(params) => {
                    match self.internal_query_param_replaceable(params) {
                        Some(ev) => Box::new(iter::once(ev)),
                        None => Box::new(iter::empty()),
                    }
                }
                QueryPattern::Generic(filter) => Box::new(self.internal_generic_query(*filter)),
            };

            if let Some(limit) = limit {
                matching_ids.extend(evs.take(limit))
            } else {
                matching_ids.extend(evs)
            }
        }

        InternalQueryResult::Set(matching_ids)
    }

    /// Query
    #[tracing::instrument(skip_all, level = "trace")]
    pub fn query<I>(&self, filters: I, order: Order) -> Vec<EventId>
    where
        I: IntoIterator<Item = Filter>,
    {
        match self.internal_query(filters) {
            InternalQueryResult::All => match order {
                Order::Asc => self.index.iter().map(|ev| ev.event_id).rev().collect(),
                Order::Desc => self.index.iter().map(|ev| ev.event_id).collect(),
            },
            InternalQueryResult::Set(set) => match order {
                Order::Asc => set.into_iter().map(|ev| ev.event_id).rev().collect(),
                Order::Desc => set.into_iter().map(|ev| ev.event_id).collect(),
            },
        }
    }

    /// Count events
    #[tracing::instrument(skip_all, level = "trace")]
    pub fn count<I>(&self, filters: I) -> usize
    where
        I: IntoIterator<Item = Filter>,
    {
        match self.internal_query(filters) {
            InternalQueryResult::All => self.index.len(),
            InternalQueryResult::Set(set) => set.len(),
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub fn negentropy_items(&self, filter: Filter) -> Vec<(EventId, Timestamp)> {
        match self.internal_query([filter]) {
            InternalQueryResult::All => self
                .index
                .iter()
                .map(|ev| (ev.event_id, ev.created_at))
                .collect(),
            InternalQueryResult::Set(set) => set
                .into_iter()
                .map(|ev| (ev.event_id, ev.created_at))
                .collect(),
        }
    }

    /// Check if an event with [`EventId`] has been deleted
    pub fn has_event_id_been_deleted(&self, event_id: &EventId) -> bool {
        self.deleted_ids.contains(event_id)
    }

    /// Check if event with [`Coordinate`] has been deleted before [`Timestamp`]
    pub fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: Timestamp,
    ) -> bool {
        if let Some(t) = self.deleted_coordinates.get(coordinate).copied() {
            t >= timestamp
        } else {
            false
        }
    }

    pub fn delete(&mut self, filter: Filter) -> Option<HashSet<EventId>> {
        match self.internal_query([filter]) {
            InternalQueryResult::All => {
                self.clear();
                None
            }
            InternalQueryResult::Set(set) => {
                let ids: HashSet<EventId> = set.into_iter().map(|ev| ev.event_id).collect();
                self.discard_events(&ids);
                Some(ids)
            }
        }
    }

    /// Clear indexes
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Database Indexes
#[derive(Debug, Clone, Default)]
pub struct DatabaseIndexes {
    inner: Arc<RwLock<InternalDatabaseIndexes>>,
}

impl DatabaseIndexes {
    /// New empty database indexes
    pub fn new() -> Self {
        Self::default()
    }

    /// Bulk index
    #[tracing::instrument(skip_all)]
    pub async fn bulk_index<'a, E>(&self, events: BTreeSet<E>) -> HashSet<EventId>
    where
        E: Into<EventOrTempEvent<'a>>,
    {
        let mut inner = self.inner.write().await;
        inner.bulk_index(events)
    }

    /// Bulk import
    ///
    /// Take a set of [Event], index them and return **only** the ones that must be stored into the database
    #[tracing::instrument(skip_all)]
    pub async fn bulk_import(&self, events: BTreeSet<Event>) -> BTreeSet<Event> {
        let mut inner = self.inner.write().await;
        inner.bulk_import(events).collect()
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
        let mut inner = self.inner.write().await;
        inner.index_event(event)
    }

    /// Query
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn query<I>(&self, filters: I, order: Order) -> Vec<EventId>
    where
        I: IntoIterator<Item = Filter>,
    {
        let inner = self.inner.read().await;
        inner.query(filters, order)
    }

    /// Count events
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn count<I>(&self, filters: I) -> usize
    where
        I: IntoIterator<Item = Filter>,
    {
        let inner = self.inner.read().await;
        inner.count(filters)
    }

    /// Get negentropy items
    #[tracing::instrument(skip_all, level = "trace")]
    pub async fn negentropy_items(&self, filter: Filter) -> Vec<(EventId, Timestamp)> {
        let inner = self.inner.read().await;
        inner.negentropy_items(filter)
    }

    /// Check if an event with [`EventId`] has been deleted
    pub async fn has_event_id_been_deleted(&self, event_id: &EventId) -> bool {
        let inner = self.inner.read().await;
        inner.has_event_id_been_deleted(event_id)
    }

    /// Check if event with [`Coordinate`] has been deleted before [`Timestamp`]
    pub async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: Timestamp,
    ) -> bool {
        let inner = self.inner.read().await;
        inner.has_coordinate_been_deleted(coordinate, timestamp)
    }

    /// Delete all events that match [Filter]
    ///
    /// If return `None`, means that all events must be deleted from DB
    pub async fn delete(&self, filter: Filter) -> Option<HashSet<EventId>> {
        let mut inner = self.inner.write().await;
        inner.delete(filter)
    }

    /// Clear indexes
    pub async fn clear(&self) {
        let mut inner = self.inner.write().await;
        inner.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use nostr::secp256k1::schnorr::Signature;
    use nostr::{FromBech32, JsonUtil, Keys, SecretKey, Tag};

    use super::*;

    const SECRET_KEY_A: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99"; // aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4
    const SECRET_KEY_B: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85"; // 79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3

    const EVENTS: [&str; 14] = [
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
        r#"{"id":"99a022e6d61c4e39c147d08a2be943b664e8030c0049325555ac1766429c2832","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1705241093,"kind":30333,"tags":[["d","multi-id"],["p","aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4"]],"content":"Multi-tags","sig":"0abfb2b696a7ed7c9e8e3bf7743686190f3f1b3d4045b72833ab6187c254f7ed278d289d52dfac3de28be861c1471421d9b1bfb5877413cbc81c84f63207a826"}"#,
    ];

    const REPLACEABLE_EVENT_1: &str = r#"{"id":"f06d755821e56fe9e25373d6bd142979ebdca0063bb0f10a95a95baf41bb5419","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1707478309,"kind":0,"tags":[],"content":"{\"name\":\"Test 1\"}","sig":"a095d9cf4f26794e6421445c0d1c4ada8273ad79a9809aaa20c566fc8d679b57f09889121050853c47be9222106abad0215705a80723f002fd47616ff6ba7bb9"}"#;
    const REPLACEABLE_EVENT_2: &str = r#"{"id":"e0899bedc802a836c331282eddf712600fea8e00123b541e25a81aa6a4669b4a","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1707478348,"kind":0,"tags":[],"content":"{\"name\":\"Test 2\"}","sig":"ca1192ac72530010a895b4d76943bf373696a6969911c486c835995122cd59a46988026e8c0ad8322bc3f5942ecd633fc903e93c0460c9a186243ab1f1597a9c"}"#;

    #[tokio::test]
    async fn test_database_indexes() {
        // Keys
        let keys_a = Keys::new(SecretKey::from_bech32(SECRET_KEY_A).unwrap());
        let keys_b = Keys::new(SecretKey::from_bech32(SECRET_KEY_B).unwrap());

        let indexes = DatabaseIndexes::new();

        // Build indexes
        let mut events: BTreeSet<Event> = BTreeSet::new();
        for event in EVENTS.into_iter() {
            let event = Event::from_json(event).unwrap();
            events.insert(event);
        }
        indexes.bulk_index(events).await;

        // Test expected output
        let expected_output = vec![
            Event::from_json(EVENTS[13]).unwrap().id(),
            Event::from_json(EVENTS[12]).unwrap().id(),
            Event::from_json(EVENTS[11]).unwrap().id(),
            // Event 10 deleted by event 12
            // Event 9 replaced by event 10
            Event::from_json(EVENTS[8]).unwrap().id(),
            Event::from_json(EVENTS[7]).unwrap().id(),
            Event::from_json(EVENTS[6]).unwrap().id(),
            Event::from_json(EVENTS[5]).unwrap().id(),
            Event::from_json(EVENTS[4]).unwrap().id(),
            // Event 3 deleted by Event 8
            // Event 2 replaced by Event 6
            Event::from_json(EVENTS[1]).unwrap().id(),
            Event::from_json(EVENTS[0]).unwrap().id(),
        ];
        assert_eq!(
            indexes.query([Filter::new()], Order::Desc).await,
            expected_output
        );
        assert_eq!(indexes.count([Filter::new()]).await, 10);

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
                Event::from_json(EVENTS[4]).unwrap().id(),
                Event::from_json(EVENTS[5]).unwrap().id(),
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
            vec![Event::from_json(EVENTS[4]).unwrap().id()]
        );

        assert_eq!(
            indexes
                .query([Filter::new().author(keys_a.public_key())], Order::Desc)
                .await,
            vec![
                Event::from_json(EVENTS[12]).unwrap().id(),
                Event::from_json(EVENTS[8]).unwrap().id(),
                Event::from_json(EVENTS[6]).unwrap().id(),
                Event::from_json(EVENTS[1]).unwrap().id(),
                Event::from_json(EVENTS[0]).unwrap().id(),
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
                Event::from_json(EVENTS[1]).unwrap().id(),
                Event::from_json(EVENTS[0]).unwrap().id(),
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
                Event::from_json(EVENTS[1]).unwrap().id(),
                Event::from_json(EVENTS[0]).unwrap().id(),
            ]
        );

        // Test get param replaceable events using identifier
        assert_eq!(
            indexes
                .query([Filter::new().identifier("id-1")], Order::Desc)
                .await,
            vec![
                Event::from_json(EVENTS[6]).unwrap().id(),
                Event::from_json(EVENTS[5]).unwrap().id(),
                Event::from_json(EVENTS[1]).unwrap().id(),
            ]
        );

        // Test get param replaceable events with multiple tags using identifier
        assert_eq!(
            indexes
                .query([Filter::new().identifier("multi-id")], Order::Desc)
                .await,
            vec![Event::from_json(EVENTS[13]).unwrap().id(),]
        );
        // As above but by using kind and pubkey
        assert_eq!(
            indexes
                .query(
                    [Filter::new()
                        .pubkey(keys_a.public_key())
                        .kind(Kind::Custom(30333))
                        .limit(1)],
                    Order::Desc
                )
                .await,
            vec![Event::from_json(EVENTS[13]).unwrap().id(),]
        );

        // Test add new replaceable event (metadata)
        let first_ev_metadata = Event::from_json(REPLACEABLE_EVENT_1).unwrap();
        let res = indexes.index_event(&first_ev_metadata).await;
        assert!(res.to_store);
        assert!(res.to_discard.is_empty());
        assert_eq!(
            indexes
                .query(
                    [Filter::new()
                        .kind(Kind::Metadata)
                        .author(keys_a.public_key())],
                    Order::Desc
                )
                .await,
            vec![first_ev_metadata.id()]
        );

        // Test add replace metadata
        let ev = Event::from_json(REPLACEABLE_EVENT_2).unwrap();
        let res = indexes.index_event(&ev).await;
        assert!(res.to_store);
        assert!(res.to_discard.contains(&first_ev_metadata.id));
        assert_eq!(
            indexes
                .query(
                    [Filter::new()
                        .kind(Kind::Metadata)
                        .author(keys_a.public_key())],
                    Order::Desc
                )
                .await,
            vec![ev.id()]
        );
    }

    #[test]
    fn test_match_event() {
        let event_id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let pubkey =
            PublicKey::from_str("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();
        let event =
            Event::new(
                event_id,
                pubkey,
                Timestamp::from(1612809991),
                Kind::TextNote,
                [
                    Tag::public_key(PublicKey::from_str("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a").unwrap()),
                    Tag::event(EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96").unwrap()),
                ],
                "test",
                Signature::from_str("273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502").unwrap(),
            );
        let event: EventIndex = EventIndex::from(&event);
        let event_with_empty_tags: EventIndex = EventIndex::from(

          &Event::new(
            event_id,
            pubkey,
            Timestamp::from(1612809991),
            Kind::TextNote,
            [],
            "test",
            Signature::from_str("273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502").unwrap(),
          )
        );

        // ID match
        let filter: FilterIndex = Filter::new().id(event_id).into();
        assert!(filter.match_event(&event));

        // Not match (kind)
        let filter: FilterIndex = Filter::new().id(event_id).kind(Kind::Metadata).into();
        assert!(!filter.match_event(&event));

        // Match (author, kind and since)
        let filter: FilterIndex = Filter::new()
            .author(pubkey)
            .kind(Kind::TextNote)
            .since(Timestamp::from(1612808000))
            .into();
        assert!(filter.match_event(&event));

        // Not match (since)
        let filter: FilterIndex = Filter::new()
            .author(pubkey)
            .kind(Kind::TextNote)
            .since(Timestamp::from(1700000000))
            .into();
        assert!(!filter.match_event(&event));

        // Match (#p tag and kind)
        let filter: FilterIndex = Filter::new()
            .pubkey(
                PublicKey::from_str(
                    "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
                )
                .unwrap(),
            )
            .kind(Kind::TextNote)
            .into();
        assert!(filter.match_event(&event));

        // Match (tags)
        let filter: FilterIndex = Filter::new()
            .pubkey(
                PublicKey::from_str(
                    "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
                )
                .unwrap(),
            )
            .event(
                EventId::from_hex(
                    "7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96",
                )
                .unwrap(),
            )
            .into();
        assert!(filter.match_event(&event));

        // Match (tags)
        let filter: FilterIndex = Filter::new()
            .events(vec![
                EventId::from_hex(
                    "7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96",
                )
                .unwrap(),
                EventId::from_hex(
                    "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5",
                )
                .unwrap(),
            ])
            .into();
        assert!(filter.match_event(&event));

        // Not match (tags)
        let filter: FilterIndex = Filter::new()
            .events(vec![EventId::from_hex(
                "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5",
            )
            .unwrap()])
            .into();
        assert!(!filter.match_event(&event));

        // Not match (tags filter for events with empty tags)
        let filter: FilterIndex = Filter::new().hashtag("this-should-not-match").into();
        assert!(!filter.match_event(&event));
        assert!(!filter.match_event(&event_with_empty_tags));
    }
}
