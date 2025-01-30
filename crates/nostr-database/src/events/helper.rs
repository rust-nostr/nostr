// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Event Store Helper
//!
//! Used for the in-memory database.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::iter;
use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip01::{Coordinate, CoordinateBorrow};
use nostr::{Alphabet, Event, EventId, Filter, Kind, PublicKey, SingleLetterTag, Timestamp};
use tokio::sync::{OwnedRwLockReadGuard, RwLock};

use crate::collections::tree::{BTreeCappedSet, Capacity, InsertResult, OverCapacityPolicy};
use crate::{Events, RejectedReason, SaveEventStatus};

type DatabaseEvent = Arc<Event>;

struct QueryByAuthorParams {
    author: PublicKey,
    since: Option<Timestamp>,
    until: Option<Timestamp>,
}

struct QueryByKindAndAuthorParams {
    kind: Kind,
    author: PublicKey,
    since: Option<Timestamp>,
    until: Option<Timestamp>,
}

impl QueryByKindAndAuthorParams {
    #[inline]
    pub fn new(kind: Kind, author: PublicKey) -> Self {
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
    author: PublicKey,
    identifier: String,
    since: Option<Timestamp>,
    until: Option<Timestamp>,
}

impl QueryByParamReplaceable {
    #[inline]
    pub fn new(kind: Kind, author: PublicKey, identifier: String) -> Self {
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
            .and_then(|v| v.iter().next().cloned());

        match (
            kinds_len,
            first_kind,
            authors_len,
            first_author,
            ids_len,
            generic_tags_len,
            identifier,
            filter.search.as_ref(),
        ) {
            (0, None, 1, Some(author), 0, 0, None, None) => Self::Author(QueryByAuthorParams {
                author,
                since: filter.since,
                until: filter.until,
            }),
            (1, Some(kind), 1, Some(author), 0, 0, None, None) => {
                Self::KindAuthor(QueryByKindAndAuthorParams {
                    kind,
                    author,
                    since: filter.since,
                    until: filter.until,
                })
            }
            (1, Some(kind), 1, Some(author), 0, _, Some(identifier), None)
                if kind.is_addressable() =>
            {
                Self::ParamReplaceable(QueryByParamReplaceable {
                    kind,
                    author,
                    identifier,
                    since: filter.since,
                    until: filter.until,
                })
            }
            _ => Self::Generic(Box::new(filter)),
        }
    }
}

/// Database Event Result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseEventResult {
    /// Status
    pub status: SaveEventStatus,
    /// List of events that should be removed from database
    pub to_discard: HashSet<EventId>,
}

enum InternalQueryResult<'a> {
    All,
    Set(BTreeSet<&'a DatabaseEvent>),
}

/// Database helper
#[derive(Debug, Clone, Default)]
struct InternalDatabaseHelper {
    /// Sorted events
    events: BTreeCappedSet<DatabaseEvent>,
    /// Events by ID
    ids: HashMap<EventId, DatabaseEvent>,
    author_index: HashMap<PublicKey, BTreeSet<DatabaseEvent>>,
    kind_author_index: HashMap<(Kind, PublicKey), BTreeSet<DatabaseEvent>>,
    param_replaceable_index: HashMap<(Kind, PublicKey, String), DatabaseEvent>,
    deleted_ids: HashSet<EventId>,
    deleted_coordinates: HashMap<Coordinate, Timestamp>,
}

impl InternalDatabaseHelper {
    pub fn bounded(size: usize) -> Self {
        let mut helper: InternalDatabaseHelper = InternalDatabaseHelper::default();
        helper.events.change_capacity(Capacity::Bounded {
            max: size,
            policy: OverCapacityPolicy::Last,
        });
        helper
    }

    // Bulk load
    //
    // NOT CHANGE `events` ARG! Processing events in ASC it's much more performant
    pub fn bulk_load(&mut self, events: BTreeSet<Event>) -> HashSet<EventId> {
        let now: Timestamp = Timestamp::now();
        events
            .into_iter()
            .rev() // Lookup ID: EVENT_ORD_IMPL
            .filter(|e| !e.kind.is_ephemeral())
            .map(|event| self.internal_index_event(&event, &now))
            .flat_map(|res| res.to_discard)
            .collect()
    }

    /// Bulk import
    pub fn bulk_import(&mut self, events: BTreeSet<Event>) -> impl Iterator<Item = Event> + '_ {
        let now: Timestamp = Timestamp::now();
        events
            .into_iter()
            .rev() // Lookup ID: EVENT_ORD_IMPL
            .filter(|e| !e.is_expired() && !e.kind.is_ephemeral())
            .filter(move |event| self.internal_index_event(event, &now).status.is_success())
    }

    fn internal_index_event(&mut self, event: &Event, now: &Timestamp) -> DatabaseEventResult {
        // Check if was already added
        if self.ids.contains_key(&event.id) {
            return DatabaseEventResult {
                status: SaveEventStatus::Rejected(RejectedReason::Duplicate),
                to_discard: HashSet::new(),
            };
        }

        // Check if was deleted or is expired
        if self.deleted_ids.contains(&event.id) {
            let mut to_discard: HashSet<EventId> = HashSet::with_capacity(1);
            to_discard.insert(event.id);
            return DatabaseEventResult {
                status: SaveEventStatus::Rejected(RejectedReason::Deleted),
                to_discard,
            };
        }

        if event.is_expired_at(now) {
            let mut to_discard: HashSet<EventId> = HashSet::with_capacity(1);
            to_discard.insert(event.id);
            return DatabaseEventResult {
                status: SaveEventStatus::Rejected(RejectedReason::Expired),
                to_discard,
            };
        }

        let mut to_discard: HashSet<EventId> = HashSet::new();

        // Compose others fields
        let author: PublicKey = event.pubkey;
        let created_at: Timestamp = event.created_at;
        let kind: Kind = event.kind;

        let mut status: SaveEventStatus = SaveEventStatus::Success;

        if kind.is_replaceable() {
            let params: QueryByKindAndAuthorParams = QueryByKindAndAuthorParams::new(kind, author);
            for ev in self.internal_query_by_kind_and_author(params) {
                if ev.created_at > created_at || ev.id == event.id {
                    status = SaveEventStatus::Rejected(RejectedReason::Replaced);
                } else {
                    to_discard.insert(ev.id);
                }
            }
        } else if kind.is_addressable() {
            match event.tags.identifier() {
                Some(identifier) => {
                    let coordinate: Coordinate =
                        Coordinate::new(kind, author).identifier(identifier);

                    // Check if coordinate was deleted
                    if self.has_coordinate_been_deleted(&coordinate, now) {
                        status = SaveEventStatus::Rejected(RejectedReason::Deleted);
                    } else {
                        let params: QueryByParamReplaceable =
                            QueryByParamReplaceable::new(kind, author, identifier.to_string());
                        if let Some(ev) = self.internal_query_param_replaceable(params) {
                            if ev.created_at > created_at || ev.id == event.id {
                                status = SaveEventStatus::Rejected(RejectedReason::Replaced);
                            } else {
                                to_discard.insert(ev.id);
                            }
                        }
                    }
                }
                None => status = SaveEventStatus::Rejected(RejectedReason::Other),
            }
        } else if kind == Kind::EventDeletion {
            // Check `e` tags
            for id in event.tags.event_ids() {
                if let Some(ev) = self.ids.get(id) {
                    if ev.pubkey != author {
                        to_discard.insert(event.id);
                        status = SaveEventStatus::Rejected(RejectedReason::InvalidDelete);
                        break;
                    }

                    if ev.created_at <= created_at {
                        to_discard.insert(ev.id);
                    }
                }
            }

            // Check `a` tags
            for coordinate in event.tags.coordinates() {
                if coordinate.public_key != author {
                    to_discard.insert(event.id);
                    status = SaveEventStatus::Rejected(RejectedReason::InvalidDelete);
                    break;
                }

                // Save deleted coordinate at certain timestamp
                self.deleted_coordinates
                    .entry(coordinate.clone())
                    .and_modify(|t| {
                        // Update only if newer
                        if created_at > *t {
                            *t = created_at
                        }
                    })
                    .or_insert(created_at);

                // Not check if ev.pubkey match the author because assume that query
                // returned only the events owned by author
                if !coordinate.identifier.is_empty() {
                    let mut params: QueryByParamReplaceable = QueryByParamReplaceable::new(
                        coordinate.kind,
                        coordinate.public_key,
                        coordinate.identifier.clone(),
                    );
                    params.until = Some(created_at);
                    if let Some(ev) = self.internal_query_param_replaceable(params) {
                        to_discard.insert(ev.id);
                    }
                } else {
                    let mut params: QueryByKindAndAuthorParams =
                        QueryByKindAndAuthorParams::new(coordinate.kind, coordinate.public_key);
                    params.until = Some(created_at);
                    to_discard.extend(self.internal_query_by_kind_and_author(params).map(|e| e.id));
                }
            }
        }

        // Remove events
        self.discard_events(&to_discard);

        // Insert event
        if status.is_success() {
            let e: DatabaseEvent = Arc::new(event.clone()); // TODO: avoid clone?

            let InsertResult { inserted, pop } = self.events.insert(e.clone());

            if inserted {
                self.ids.insert(e.id, e.clone());
                self.author_index
                    .entry(author)
                    .or_default()
                    .insert(e.clone());

                if kind.is_addressable() {
                    if let Some(identifier) = e.tags.identifier() {
                        self.param_replaceable_index
                            .insert((kind, author, identifier.to_string()), e.clone());
                    }
                }

                if kind.is_replaceable() {
                    let mut set = BTreeSet::new();
                    set.insert(e);
                    self.kind_author_index.insert((kind, author), set);
                } else {
                    self.kind_author_index
                        .entry((kind, author))
                        .or_default()
                        .insert(e);
                }
            } else {
                to_discard.insert(e.id);
            }

            if let Some(event) = pop {
                to_discard.insert(event.id);
                self.discard_event(event);
            }
        }

        DatabaseEventResult { status, to_discard }
    }

    fn discard_events(&mut self, ids: &HashSet<EventId>) {
        for id in ids.iter() {
            if let Some(ev) = self.ids.remove(id) {
                self.events.remove(&ev);

                if let Some(set) = self.author_index.get_mut(&ev.pubkey) {
                    set.remove(&ev);
                }

                if ev.kind.is_addressable() {
                    if let Some(identifier) = ev.tags.identifier() {
                        self.param_replaceable_index.remove(&(
                            ev.kind,
                            ev.pubkey,
                            identifier.to_string(),
                        ));
                    }
                }

                if let Some(set) = self.kind_author_index.get_mut(&(ev.kind, ev.pubkey)) {
                    set.remove(&ev);
                }
            }
            self.deleted_ids.insert(*id);
        }
    }

    fn discard_event(&mut self, ev: DatabaseEvent) {
        self.ids.remove(&ev.id);

        if let Some(set) = self.author_index.get_mut(&ev.pubkey) {
            set.remove(&ev);
        }

        if ev.kind.is_addressable() {
            if let Some(identifier) = ev.tags.identifier() {
                self.param_replaceable_index
                    .remove(&(ev.kind, ev.pubkey, identifier.to_string()));
            }
        }

        if let Some(set) = self.kind_author_index.get_mut(&(ev.kind, ev.pubkey)) {
            set.remove(&ev);
        }
    }

    /// Import [Event]
    ///
    /// **This method assume that [`Event`] was already verified**
    pub fn index_event(&mut self, event: &Event) -> DatabaseEventResult {
        // Check if it's ephemeral
        if event.kind.is_ephemeral() {
            return DatabaseEventResult {
                status: SaveEventStatus::Rejected(RejectedReason::Ephemeral),
                to_discard: HashSet::new(),
            };
        }
        let now = Timestamp::now();
        self.internal_index_event(event, &now)
    }

    /// Query by public key
    fn internal_query_by_author<'a>(
        &'a self,
        params: QueryByAuthorParams,
    ) -> Box<dyn Iterator<Item = &'a DatabaseEvent> + 'a> {
        let QueryByAuthorParams {
            author,
            since,
            until,
        } = params;
        match self.author_index.get(&author) {
            Some(set) => Box::new(set.iter().filter(move |ev| {
                if self.deleted_ids.contains(&ev.id) {
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
    ) -> Box<dyn Iterator<Item = &'a DatabaseEvent> + 'a> {
        let QueryByKindAndAuthorParams {
            kind,
            author,
            since,
            until,
        } = params;
        match self.kind_author_index.get(&(kind, author)) {
            Some(set) => Box::new(set.iter().filter(move |ev| {
                if self.deleted_ids.contains(&ev.id) {
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
    ) -> Option<&DatabaseEvent> {
        let QueryByParamReplaceable {
            kind,
            author,
            identifier,
            since,
            until,
        } = params;

        if !kind.is_addressable() {
            return None;
        }

        let ev = self
            .param_replaceable_index
            .get(&(kind, author, identifier))?;

        if self.deleted_ids.contains(&ev.id) {
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
    #[inline]
    fn internal_generic_query(&self, filter: Filter) -> impl Iterator<Item = &DatabaseEvent> {
        self.events
            .iter()
            .filter(move |event| !self.deleted_ids.contains(&event.id) && filter.match_event(event))
    }

    fn internal_query(&self, filter: Filter) -> InternalQueryResult {
        if filter.is_empty() {
            return InternalQueryResult::All;
        }

        if let (Some(since), Some(until)) = (filter.since, filter.until) {
            if since > until {
                return InternalQueryResult::Set(BTreeSet::new());
            }
        }

        let mut matching_ids: BTreeSet<&DatabaseEvent> = BTreeSet::new();
        let limit: Option<usize> = filter.limit;

        let evs: Box<dyn Iterator<Item = &DatabaseEvent>> = match QueryPattern::from(filter) {
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

        InternalQueryResult::Set(matching_ids)
    }

    #[inline]
    pub fn event_by_id(&self, id: &EventId) -> Option<&Event> {
        self.ids.get(id).map(|e| e.deref())
    }

    #[inline]
    pub fn has_event(&self, id: &EventId) -> bool {
        self.ids.contains_key(id)
    }

    /// Query
    pub fn query<'a>(&'a self, filter: Filter) -> Box<dyn Iterator<Item = &'a Event> + 'a> {
        match self.internal_query(filter) {
            InternalQueryResult::All => Box::new(self.events.iter().map(|ev| ev.as_ref())),
            InternalQueryResult::Set(set) => Box::new(set.into_iter().map(|ev| ev.as_ref())),
        }
    }

    /// Count events
    pub fn count(&self, filter: Filter) -> usize {
        match self.internal_query(filter) {
            InternalQueryResult::All => self.events.len(),
            InternalQueryResult::Set(set) => set.len(),
        }
    }

    pub fn negentropy_items(&self, filter: Filter) -> Vec<(EventId, Timestamp)> {
        match self.internal_query(filter) {
            InternalQueryResult::All => self
                .events
                .iter()
                .map(|ev| (ev.id, ev.created_at))
                .collect(),
            InternalQueryResult::Set(set) => {
                set.into_iter().map(|ev| (ev.id, ev.created_at)).collect()
            }
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
        timestamp: &Timestamp,
    ) -> bool {
        if let Some(t) = self.deleted_coordinates.get(coordinate) {
            t >= timestamp
        } else {
            false
        }
    }

    pub fn delete(&mut self, filter: Filter) -> Option<HashSet<EventId>> {
        match self.internal_query(filter) {
            InternalQueryResult::All => {
                self.clear();
                None
            }
            InternalQueryResult::Set(set) => {
                let ids: HashSet<EventId> = set.into_iter().map(|ev| ev.id).collect();
                self.discard_events(&ids);
                Some(ids)
            }
        }
    }

    pub fn clear(&mut self) {
        // Get current capacity
        let capacity: Capacity = self.events.capacity();

        // Reset helper to default
        *self = Self::default();

        // Change capacity
        self.events.change_capacity(capacity);
    }
}

/// Database helper transaction
pub struct QueryTransaction {
    guard: OwnedRwLockReadGuard<InternalDatabaseHelper>,
}

/// Database Indexes
#[derive(Debug, Clone, Default)]
pub struct DatabaseHelper {
    inner: Arc<RwLock<InternalDatabaseHelper>>,
}

impl DatabaseHelper {
    /// Unbounded database helper
    #[inline]
    pub fn unbounded() -> Self {
        Self::default()
    }

    /// Bounded database helper
    #[inline]
    pub fn bounded(max: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(InternalDatabaseHelper::bounded(max))),
        }
    }

    /// Query transaction
    #[inline]
    pub async fn qtxn(&self) -> QueryTransaction {
        QueryTransaction {
            guard: self.inner.clone().read_owned().await,
        }
    }

    /// Bulk index
    pub async fn bulk_load(&self, events: BTreeSet<Event>) -> HashSet<EventId> {
        let mut inner = self.inner.write().await;
        inner.bulk_load(events)
    }

    /// Bulk import
    ///
    /// Take a set of [Event], index them and return **only** the ones that must be stored into the database
    pub async fn bulk_import(&self, events: BTreeSet<Event>) -> BTreeSet<Event> {
        let mut inner = self.inner.write().await;
        inner.bulk_import(events).collect()
    }

    /// Index [`Event`]
    ///
    /// **This method assumes that [`Event`] was already verified**
    pub async fn index_event(&self, event: &Event) -> DatabaseEventResult {
        let mut inner = self.inner.write().await;
        inner.index_event(event)
    }

    /// Get [Event] by ID
    pub async fn event_by_id(&self, id: &EventId) -> Option<Event> {
        let inner = self.inner.read().await;
        inner.event_by_id(id).cloned()
    }

    /// Check if event exists
    pub async fn has_event(&self, id: &EventId) -> bool {
        let inner = self.inner.read().await;
        inner.has_event(id)
    }

    /// Query
    pub async fn query(&self, filter: Filter) -> Events {
        let inner = self.inner.read().await;
        let mut events = Events::new(&filter);
        events.extend(inner.query(filter).cloned());
        events
    }

    /// Query
    pub fn fast_query<'a>(
        &self,
        txn: &'a QueryTransaction,
        filter: Filter,
    ) -> Box<dyn Iterator<Item = &'a Event> + 'a> {
        txn.guard.query(filter)
    }

    /// Count events
    pub async fn count(&self, filter: Filter) -> usize {
        let inner = self.inner.read().await;
        inner.count(filter)
    }

    /// Get negentropy items
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
    pub async fn has_coordinate_been_deleted<'a>(
        &self,
        coordinate: &'a CoordinateBorrow<'a>,
        timestamp: &Timestamp,
    ) -> bool {
        let inner = self.inner.read().await;
        inner.has_coordinate_been_deleted(&coordinate.into_owned(), timestamp)
    }

    /// Delete all events that match [Filter]
    ///
    /// If return `None`, means that all events must be deleted from DB
    pub async fn delete(&self, filter: Filter) -> Option<HashSet<EventId>> {
        let mut inner = self.inner.write().await;
        inner.delete(filter)
    }

    /// Clear helper
    pub async fn clear(&self) {
        let mut inner = self.inner.write().await;
        inner.clear();
    }
}

#[cfg(test)]
mod tests {
    use nostr::{FromBech32, JsonUtil, Keys, SecretKey};

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

        let indexes = DatabaseHelper::unbounded();

        // Build indexes
        let mut events: BTreeSet<Event> = BTreeSet::new();
        for event in EVENTS.into_iter() {
            let event = Event::from_json(event).unwrap();
            events.insert(event);
        }
        indexes.bulk_load(events).await;

        // Test expected output
        let expected_output = vec![
            Event::from_json(EVENTS[13]).unwrap(),
            Event::from_json(EVENTS[12]).unwrap(),
            // Event 11 is invalid deletion
            // Event 10 deleted by event 12
            // Event 9 replaced by event 10
            Event::from_json(EVENTS[8]).unwrap(),
            // Event 7 is invalid deletion
            Event::from_json(EVENTS[6]).unwrap(),
            Event::from_json(EVENTS[5]).unwrap(),
            Event::from_json(EVENTS[4]).unwrap(),
            // Event 3 deleted by Event 8
            // Event 2 replaced by Event 6
            Event::from_json(EVENTS[1]).unwrap(),
            Event::from_json(EVENTS[0]).unwrap(),
        ];
        assert_eq!(indexes.query(Filter::new()).await.to_vec(), expected_output);
        assert_eq!(indexes.count(Filter::new()).await, 8);

        // Test get previously deleted replaceable event (check if was deleted by indexes)
        assert!(indexes
            .query(
                Filter::new()
                    .kind(Kind::Metadata)
                    .author(keys_a.public_key())
            )
            .await
            .is_empty());

        // Test get previously deleted param. replaceable event (check if was deleted by indexes)
        assert!(indexes
            .query(
                Filter::new()
                    .kind(Kind::Custom(32122))
                    .author(keys_a.public_key())
                    .identifier("id-2")
            )
            .await
            .is_empty());

        // Test get param replaceable events WITHOUT using indexes (identifier not passed)
        assert_eq!(
            indexes
                .query(
                    Filter::new()
                        .kind(Kind::Custom(32122))
                        .author(keys_b.public_key())
                )
                .await
                .to_vec(),
            vec![
                Event::from_json(EVENTS[5]).unwrap(),
                Event::from_json(EVENTS[4]).unwrap(),
            ]
        );

        // Test get param replaceable events using indexes
        assert_eq!(
            indexes
                .query(
                    Filter::new()
                        .kind(Kind::Custom(32122))
                        .author(keys_b.public_key())
                        .identifier("id-3")
                )
                .await
                .to_vec(),
            vec![Event::from_json(EVENTS[4]).unwrap()]
        );

        assert_eq!(
            indexes
                .query(Filter::new().author(keys_a.public_key()))
                .await
                .to_vec(),
            vec![
                Event::from_json(EVENTS[12]).unwrap(),
                Event::from_json(EVENTS[8]).unwrap(),
                Event::from_json(EVENTS[6]).unwrap(),
                Event::from_json(EVENTS[1]).unwrap(),
                Event::from_json(EVENTS[0]).unwrap(),
            ]
        );

        assert_eq!(
            indexes
                .query(
                    Filter::new()
                        .author(keys_a.public_key())
                        .kinds([Kind::TextNote, Kind::Custom(32121)])
                )
                .await
                .to_vec(),
            vec![
                Event::from_json(EVENTS[1]).unwrap(),
                Event::from_json(EVENTS[0]).unwrap(),
            ]
        );

        assert_eq!(
            indexes
                .query(
                    Filter::new()
                        .authors([keys_a.public_key(), keys_b.public_key()])
                        .kinds([Kind::TextNote, Kind::Custom(32121)])
                )
                .await
                .to_vec(),
            vec![
                Event::from_json(EVENTS[1]).unwrap(),
                Event::from_json(EVENTS[0]).unwrap(),
            ]
        );

        // Test get param replaceable events using identifier
        assert_eq!(
            indexes
                .query(Filter::new().identifier("id-1"))
                .await
                .to_vec(),
            vec![
                Event::from_json(EVENTS[6]).unwrap(),
                Event::from_json(EVENTS[5]).unwrap(),
                Event::from_json(EVENTS[1]).unwrap(),
            ]
        );

        // Test get param replaceable events with multiple tags using identifier
        assert_eq!(
            indexes
                .query(Filter::new().identifier("multi-id"))
                .await
                .to_vec(),
            vec![Event::from_json(EVENTS[13]).unwrap()]
        );
        // As above but by using kind and pubkey
        assert_eq!(
            indexes
                .query(
                    Filter::new()
                        .pubkey(keys_a.public_key())
                        .kind(Kind::Custom(30333))
                        .limit(1)
                )
                .await
                .to_vec(),
            vec![Event::from_json(EVENTS[13]).unwrap()]
        );

        // Test add new replaceable event (metadata)
        let first_ev_metadata = Event::from_json(REPLACEABLE_EVENT_1).unwrap();
        let res = indexes.index_event(&first_ev_metadata).await;
        assert!(res.status.is_success());
        assert!(res.to_discard.is_empty());
        assert_eq!(
            indexes
                .query(
                    Filter::new()
                        .kind(Kind::Metadata)
                        .author(keys_a.public_key())
                )
                .await
                .to_vec(),
            vec![first_ev_metadata.clone()]
        );

        // Test add replace metadata
        let ev = Event::from_json(REPLACEABLE_EVENT_2).unwrap();
        let res = indexes.index_event(&ev).await;
        assert!(res.status.is_success());
        assert!(res.to_discard.contains(&first_ev_metadata.id));
        assert_eq!(
            indexes
                .query(
                    Filter::new()
                        .kind(Kind::Metadata)
                        .author(keys_a.public_key())
                )
                .await
                .to_vec(),
            vec![ev]
        );
    }
}
