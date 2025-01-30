// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! [`nostrdb`](https://github.com/damus-io/nostrdb) storage backend for Nostr apps

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for async_trait

use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

pub extern crate nostr;
pub extern crate nostr_database as database;
pub extern crate nostrdb;

use nostr_database::prelude::*;
use nostrdb::{Config, Filter as NdbFilter, Ndb, NdbStrVariant, Note, QueryResult, Transaction};

const MAX_RESULTS: i32 = 10_000;

// Wrap `Ndb` into `NdbDatabase` because only traits defined in the current crate can be implemented for types defined outside the crate!

/// [`nostrdb`](https://github.com/damus-io/nostrdb) backend
#[derive(Debug, Clone)]
pub struct NdbDatabase {
    db: Ndb,
}

impl NdbDatabase {
    /// Open nostrdb
    pub fn open<P>(path: P) -> Result<Self, DatabaseError>
    where
        P: AsRef<str>,
    {
        let path: &str = path.as_ref();
        let config = Config::new();

        Ok(Self {
            db: Ndb::new(path, &config).map_err(DatabaseError::backend)?,
        })
    }
}

impl Deref for NdbDatabase {
    type Target = Ndb;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl DerefMut for NdbDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl From<Ndb> for NdbDatabase {
    fn from(db: Ndb) -> Self {
        Self { db }
    }
}

impl NostrDatabase for NdbDatabase {
    fn backend(&self) -> Backend {
        Backend::LMDB
    }
}

impl NostrEventsDatabase for NdbDatabase {
    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>> {
        Box::pin(async move {
            let msg = RelayMessage::event(SubscriptionId::new("ndb"), event.clone());
            let json: String = msg.as_json();
            self.db
                .process_event(&json)
                .map_err(DatabaseError::backend)?;
            // TODO: shouldn't return a success since we don't know if the ingestion was successful or not.
            Ok(SaveEventStatus::Success)
        })
    }

    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        Box::pin(async move {
            let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
            let res = self.db.get_note_by_id(&txn, event_id.as_bytes());
            Ok(if res.is_ok() {
                DatabaseEventStatus::Saved
            } else {
                DatabaseEventStatus::NotExistent
            })
        })
    }

    fn has_coordinate_been_deleted<'a>(
        &'a self,
        _coordinate: &'a CoordinateBorrow<'a>,
        _timestamp: &'a Timestamp,
    ) -> BoxedFuture<'a, Result<bool, DatabaseError>> {
        Box::pin(async move { Ok(false) })
    }

    fn event_id_seen(
        &self,
        _event_id: EventId,
        _relay_url: RelayUrl,
    ) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { Ok(()) })
    }

    fn event_seen_on_relays<'a>(
        &'a self,
        _event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<HashSet<RelayUrl>>, DatabaseError>> {
        Box::pin(async move { Err(DatabaseError::NotSupported) })
    }

    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
            let note = self
                .db
                .get_note_by_id(&txn, event_id.as_bytes())
                .map_err(DatabaseError::backend)?;
            Ok(Some(ndb_note_to_event(note)?.into_owned()))
        })
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move {
            let txn: Transaction = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
            let res: Vec<QueryResult> = ndb_query(&self.db, &txn, filter)?;
            Ok(res.len())
        })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        Box::pin(async move {
            let txn: Transaction = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
            let mut events: Events = Events::new(&filter);
            let res: Vec<QueryResult> = ndb_query(&self.db, &txn, filter)?;
            events.extend(
                res.into_iter()
                    .filter_map(|r| ndb_note_to_event(r.note).ok())
                    .map(|e| e.into_owned()),
            );
            Ok(events)
        })
    }

    fn negentropy_items(
        &self,
        filter: Filter,
    ) -> BoxedFuture<Result<Vec<(EventId, Timestamp)>, DatabaseError>> {
        Box::pin(async move {
            let txn: Transaction = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
            let res: Vec<QueryResult> = ndb_query(&self.db, &txn, filter)?;
            Ok(res
                .into_iter()
                .map(|r| ndb_note_to_neg_item(r.note))
                .collect())
        })
    }

    fn delete(&self, _filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { Err(DatabaseError::NotSupported) })
    }
}

impl NostrDatabaseWipe for NdbDatabase {
    #[inline]
    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { Err(DatabaseError::NotSupported) })
    }
}

fn ndb_query<'a>(
    db: &Ndb,
    txn: &'a Transaction,
    filter: Filter,
) -> Result<Vec<QueryResult<'a>>, DatabaseError> {
    let filter: nostrdb::Filter = ndb_filter_conversion(filter);
    db.query(txn, &[filter], MAX_RESULTS)
        .map_err(DatabaseError::backend)
}

fn ndb_filter_conversion(f: Filter) -> nostrdb::Filter {
    let mut filter = NdbFilter::new();

    if let Some(ids) = f.ids {
        if !ids.is_empty() {
            filter = filter.ids(ids.iter().map(|p| p.as_bytes()));
        }
    }

    if let Some(authors) = f.authors {
        if !authors.is_empty() {
            filter = filter.authors(authors.iter().map(|p| p.as_bytes()));
        }
    }

    if let Some(kinds) = f.kinds {
        if !kinds.is_empty() {
            filter = filter.kinds(kinds.into_iter().map(|p| p.as_u16() as u64));
        }
    }

    if !f.generic_tags.is_empty() {
        for (single_letter, set) in f.generic_tags.into_iter() {
            filter = filter.tags(set, single_letter.as_char());
        }
    }

    if let Some(since) = f.since {
        filter = filter.since(since.as_u64());
    }

    if let Some(until) = f.until {
        filter = filter.until(until.as_u64());
    }

    if let Some(limit) = f.limit {
        filter = filter.limit(limit as u64);
    }

    filter.build()
}

fn ndb_note_to_event(note: Note) -> Result<EventBorrow, DatabaseError> {
    Ok(EventBorrow {
        id: note.id(),
        pubkey: note.pubkey(),
        created_at: Timestamp::from(note.created_at()),
        kind: note.kind().try_into().map_err(DatabaseError::backend)?,
        tags: ndb_note_to_tags(&note)?,
        content: note.content(),
        sig: note.sig(),
    })
}

fn ndb_note_to_tags<'a>(note: &Note<'a>) -> Result<Vec<CowTag<'a>>, DatabaseError> {
    let ndb_tags = note.tags();
    let mut tags: Vec<CowTag<'a>> = Vec::with_capacity(ndb_tags.count() as usize);
    for tag in ndb_tags.iter() {
        let tag_str: Vec<Cow<'a, str>> = tag
            .into_iter()
            .map(|s| match s.variant() {
                NdbStrVariant::Id(id) => Cow::Owned(hex::encode(id)),
                NdbStrVariant::Str(s) => Cow::Borrowed(s),
            })
            .collect();
        let tag = CowTag::parse(tag_str).map_err(DatabaseError::backend)?;
        tags.push(tag);
    }
    Ok(tags)
}

fn ndb_note_to_neg_item(note: Note) -> (EventId, Timestamp) {
    let id = EventId::from_byte_array(*note.id());
    let created_at = Timestamp::from_secs(note.created_at());
    (id, created_at)
}
