// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! [`nostrdb`](https://github.com/damus-io/nostrdb) storage backend for Nostr apps

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for async_trait

use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

pub extern crate nostr;
pub extern crate nostr_database as database;
pub extern crate nostrdb;

use nostr::event::borrow::EventBorrow;
use nostr::event::tag::cow::CowTag;
use nostr_database::prelude::*;
use nostrdb::{Config, Filter as NdbFilter, Ndb, NdbStrVariant, Note, QueryResult, Transaction};

const MAX_RESULTS: i32 = 10_000;

// Wrap `Ndb` into `NdbDatabase` because only traits defined in the current crate can be implemented for types defined outside the crate!

/// [`nostrdb`](https://github.com/damus-io/nostrdb) backend
#[derive(Debug, Clone)]
pub struct NdbDatabase {
    db: Ndb,
}

/// [`nostrdb`](https://github.com/damus-io/nostrdb) transaction
pub struct NdbTransaction {
    db: Ndb,
    txn: Transaction,
}

// Required for the DatabaseTransaction trait
unsafe impl Send for NdbTransaction {}
unsafe impl Sync for NdbTransaction {}

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

#[async_trait]
impl NostrDatabase for NdbDatabase {
    fn backend(&self) -> Backend {
        Backend::LMDB
    }

    async fn wipe(&self) -> Result<(), DatabaseError> {
        Err(DatabaseError::NotSupported)
    }
}

#[async_trait]
impl NostrEventsDatabaseTransaction for NdbTransaction {
    async fn query<'a>(&'a self, filters: Vec<Filter>) -> Result<QueryEvents<'a>, DatabaseError> {
        let res: Vec<QueryResult> = ndb_query(&self.db, &self.txn, filters)?;
        let events = res
            .into_iter()
            .filter_map(|r| ndb_note_to_event(r.note).ok())
            .collect();
        Ok(QueryEvents::List(events))
    }
}

#[async_trait]
impl NostrEventsDatabase for NdbDatabase {
    async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, DatabaseError> {
        let msg = RelayMessage::event(SubscriptionId::new("ndb"), event.clone());
        let json: String = msg.as_json();
        self.db
            .process_event(&json)
            .map_err(DatabaseError::backend)?;
        // TODO: shouldn't return a success since we don't know if the ingestion was successful or not.
        Ok(SaveEventStatus::Success)
    }

    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
        let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let res = self.db.get_note_by_id(&txn, event_id.as_bytes());
        Ok(if res.is_ok() {
            DatabaseEventStatus::Saved
        } else {
            DatabaseEventStatus::NotExistent
        })
    }

    async fn has_coordinate_been_deleted(
        &self,
        _coordinate: &Coordinate,
        _timestamp: &Timestamp,
    ) -> Result<bool, DatabaseError> {
        Ok(false)
    }

    async fn event_id_seen(
        &self,
        _event_id: EventId,
        _relay_url: RelayUrl,
    ) -> Result<(), DatabaseError> {
        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        _event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError> {
        // TODO: use in-memory map to keep track of seen relays
        Err(DatabaseError::NotSupported)
    }

    async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Event>, DatabaseError> {
        let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let note = self
            .db
            .get_note_by_id(&txn, event_id.as_bytes())
            .map_err(DatabaseError::backend)?;
        Ok(Some(ndb_note_to_event(note)?.into_event()))
    }

    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        let txn: Transaction = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let res: Vec<QueryResult> = ndb_query(&self.db, &txn, filters)?;
        Ok(res.len())
    }

    async fn begin_txn(&self) -> Result<Box<dyn NostrEventsDatabaseTransaction>, DatabaseError> {
        let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        Ok(Box::new(NdbTransaction {
            db: self.db.clone(),
            txn,
        }))
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        let txn: Transaction = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let res: Vec<QueryResult> = ndb_query(&self.db, &txn, vec![filter])?;
        Ok(res
            .into_iter()
            .map(|r| ndb_note_to_neg_item(r.note))
            .collect())
    }

    async fn delete(&self, _filter: Filter) -> Result<(), DatabaseError> {
        Err(DatabaseError::NotSupported)
    }
}

fn ndb_query<'a>(
    db: &Ndb,
    txn: &'a Transaction,
    filters: Vec<Filter>,
) -> Result<Vec<QueryResult<'a>>, DatabaseError> {
    let filters: Vec<nostrdb::Filter> = filters.into_iter().map(ndb_filter_conversion).collect();
    db.query(txn, &filters, MAX_RESULTS)
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
            let authors: Vec<[u8; 32]> = authors.into_iter().map(|p| p.to_bytes()).collect();
            filter = filter.authors(authors.iter());
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

fn ndb_note_to_event<'a>(note: Note<'a>) -> Result<QueryEvent<'a>, DatabaseError> {
    let event = EventBorrow {
        id: note.id(),
        pubkey: note.pubkey(),
        created_at: Timestamp::from(note.created_at()),
        kind: note.kind().try_into().map_err(DatabaseError::backend)?,
        tags: ndb_note_to_tags(&note)?,
        content: note.content(),
        sig: note.sig(),
    };
    Ok(QueryEvent::Borrowed(event))
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
