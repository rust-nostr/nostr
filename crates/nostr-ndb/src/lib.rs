// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! ndb storage backend for Nostr apps

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use std::collections::{BTreeSet, HashSet};
use std::str::FromStr;

pub extern crate nostr;
pub extern crate nostr_database as database;

use async_trait::async_trait;
use nostr::nips::nip01::Coordinate;
use nostr::secp256k1::schnorr::Signature;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Event, EventId, Filter, JsonUtil, Kind, RelayMessage, SubscriptionId, Timestamp, Url};
use nostr_database::{Backend, DatabaseError, NostrDatabase, Order};
use nostrdb::{Config, Ndb, Note, Transaction};

/// ndb backend
#[derive(Debug, Clone)]
pub struct NdbDatabase {
    db: Ndb,
}

impl NdbDatabase {
    /// Open nostrdb
    pub async fn open<P>(path: P) -> Result<Self, DatabaseError>
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

#[async_trait]
impl NostrDatabase for NdbDatabase {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        Backend::LMDB
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        let msg = RelayMessage::event(SubscriptionId::new("ndb"), event.clone());
        let json: String = msg.as_json();
        self.db
            .process_event(&json)
            .map_err(DatabaseError::backend)?;
        Ok(true)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn bulk_import(&self, events: BTreeSet<Event>) -> Result<(), Self::Err> {
        for event in events.into_iter() {
            let msg = RelayMessage::event(SubscriptionId::new("ndb"), event);
            let json: String = msg.as_json();
            self.db
                .process_event(&json)
                .map_err(DatabaseError::backend)?;
        }
        Ok(())
    }

    async fn has_event_already_been_saved(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let res = self.db.get_note_by_id(&txn, &event_id.to_bytes());
        Ok(res.is_ok())
    }

    async fn has_event_already_been_seen(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        self.has_event_already_been_saved(event_id).await
    }

    async fn has_event_id_been_deleted(&self, _event_id: &EventId) -> Result<bool, Self::Err> {
        Ok(false)
    }

    async fn has_coordinate_been_deleted(
        &self,
        _coordinate: &Coordinate,
        _timestamp: Timestamp,
    ) -> Result<bool, Self::Err> {
        Ok(false)
    }

    async fn event_id_seen(&self, _event_id: EventId, _relay_url: Url) -> Result<(), Self::Err> {
        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        _event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err> {
        Err(DatabaseError::NotSupported)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err> {
        let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let note = self
            .db
            .get_note_by_id(&txn, &event_id.to_bytes())
            .map_err(DatabaseError::backend)?;
        ndb_note_to_event(note)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn count(&self, _filters: Vec<Filter>) -> Result<usize, Self::Err> {
        Err(DatabaseError::FeatureDisabled)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn query(&self, filters: Vec<Filter>, _order: Order) -> Result<Vec<Event>, Self::Err> {
        let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let filters = filters.into_iter().map(ndb_filter_conversion).collect();
        let res = self
            .db
            .query(&txn, filters, i32::MAX)
            .map_err(DatabaseError::backend)?;
        Ok(res
            .into_iter()
            .filter_map(|r| ndb_note_to_event(r.note).ok())
            .collect())
    }

    async fn event_ids_by_filters(
        &self,
        filters: Vec<Filter>,
        _order: Order,
    ) -> Result<Vec<EventId>, Self::Err> {
        let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let filters = filters.into_iter().map(ndb_filter_conversion).collect();
        let res = self
            .db
            .query(&txn, filters, i32::MAX)
            .map_err(DatabaseError::backend)?;
        Ok(res
            .into_iter()
            .filter_map(|r| ndb_note_to_id(r.note).ok())
            .collect())
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
        let txn = Transaction::new(&self.db).map_err(DatabaseError::backend)?;
        let filter = ndb_filter_conversion(filter);
        let res = self
            .db
            .query(&txn, vec![filter], i32::MAX)
            .map_err(DatabaseError::backend)?;
        Ok(res
            .into_iter()
            .filter_map(|r| ndb_note_to_neg_item(r.note).ok())
            .collect())
    }

    async fn delete(&self, filter: Filter) -> Result<(), Self::Err> {
        Err(DatabaseError::NotSupported)
    }

    async fn wipe(&self) -> Result<(), Self::Err> {
        Err(DatabaseError::NotSupported)
    }
}

fn ndb_filter_conversion(f: Filter) -> nostrdb::Filter {
    let mut filter = nostrdb::Filter::new();

    if !f.ids.is_empty() {
        let ids = f.ids.into_iter().map(|p| p.to_bytes()).collect();
        filter.ids(ids);
    }

    if !f.authors.is_empty() {
        let authors = f.authors.into_iter().map(|p| p.serialize()).collect();
        filter.authors(authors);
    }

    if !f.kinds.is_empty() {
        let kinds = f.kinds.into_iter().map(|p| p.as_u64()).collect();
        filter.kinds(kinds);
    }

    // TODO: convert tags

    if let Some(since) = f.since {
        filter.since(since.as_u64());
    }

    /* if let Some(until) = f.until {
        filter.until(until.as_u64());
    } */

    if let Some(limit) = f.limit {
        filter.limit(limit as u64);
    }

    filter.build()
}

fn ndb_note_to_event(note: Note) -> Result<Event, DatabaseError> {
    let id = EventId::from_slice(note.id()).map_err(DatabaseError::nostr)?;
    let public_key = XOnlyPublicKey::from_slice(note.pubkey()).map_err(DatabaseError::nostr)?;
    let created_at = Timestamp::from(note.created_at());
    let kind = Kind::from(note.kind() as u64);
    let tags = Vec::new(); // TODO
    let content = note.content();
    // TODO
    let sig = Signature::from_str("a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd").map_err(DatabaseError::nostr)?;
    Ok(Event::new(
        id, public_key, created_at, kind, tags, content, sig,
    ))
}

fn ndb_note_to_id(note: Note) -> Result<EventId, DatabaseError> {
    EventId::from_slice(note.id()).map_err(DatabaseError::nostr)
}

fn ndb_note_to_neg_item(note: Note) -> Result<(EventId, Timestamp), DatabaseError> {
    let id = EventId::from_slice(note.id()).map_err(DatabaseError::nostr)?;
    let created_at = Timestamp::from(note.created_at());
    Ok((id, created_at))
}
