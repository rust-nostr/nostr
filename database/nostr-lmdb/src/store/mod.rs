// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fs;
use std::path::Path;

use flume::Sender;
use nostr_database::prelude::*;
use nostr_database::SaveEventStatus;

mod error;
mod ingester;
mod lmdb;
mod types;

pub use self::error::Error;
use self::ingester::{Ingester, IngesterItem};
use self::lmdb::Lmdb;

#[derive(Debug)]
pub struct Store {
    db: Lmdb,
    ingester: Sender<IngesterItem>,
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            ingester: self.ingester.clone(),
        }
    }
}

impl Store {
    pub(super) fn open<P>(
        path: P,
        map_size: usize,
        max_readers: u32,
        max_dbs: u32,
    ) -> Result<Store, Error>
    where
        P: AsRef<Path>,
    {
        let path: &Path = path.as_ref();

        // Create the directory if it doesn't exist
        fs::create_dir_all(path)?;

        let db: Lmdb = Lmdb::new(path, map_size, max_readers, max_dbs)?;
        let ingester = Ingester::run(db.clone());

        Ok(Self { db, ingester })
    }

    /// Store an event.
    pub async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, Error> {
        let (item, rx) = IngesterItem::save_event_with_feedback(event);

        // Send to the ingester
        self.ingester.send(item).map_err(|_| Error::MpscSend)?;

        // Wait for a reply
        rx.await?
    }

    /// Get an event by ID
    pub fn get_event_by_id(&self, id: &EventId) -> Result<Option<Event>, Error> {
        let txn = self.db.read_txn()?;
        let result = self
            .db
            .get_event_by_id(&txn, id.as_bytes())?
            .map(|event_borrow| event_borrow.into_owned());
        txn.commit()?;
        Ok(result)
    }

    /// Do we have an event
    pub fn has_event(&self, id: &EventId) -> Result<bool, Error> {
        let txn = self.db.read_txn()?;
        let has: bool = self.db.has_event(&txn, id.as_bytes())?;
        txn.commit()?;
        Ok(has)
    }

    /// Is the event deleted
    pub fn event_is_deleted(&self, id: &EventId) -> Result<bool, Error> {
        let txn = self.db.read_txn()?;
        let deleted: bool = self.db.is_deleted(&txn, id.as_bytes())?;
        txn.commit()?;
        Ok(deleted)
    }

    pub fn count(&self, filter: Filter) -> Result<usize, Error> {
        let txn = self.db.read_txn()?;
        let iter = self.db.query(&txn, filter)?;
        let count = iter.count();
        txn.commit()?;
        Ok(count)
    }

    // Lookup ID: EVENT_ORD_IMPL
    pub fn query(&self, filter: Filter) -> Result<Events, Error> {
        let txn = self.db.read_txn()?;
        let mut events_wrapper = Events::new(&filter);
        events_wrapper.extend(self.db.query(&txn, filter)?.map(|e| e.into_owned()));
        txn.commit()?;
        Ok(events_wrapper)
    }

    pub fn negentropy_items(&self, filter: Filter) -> Result<Vec<(EventId, Timestamp)>, Error> {
        let txn = self.db.read_txn()?;
        let items = self
            .db
            .query(&txn, filter)?
            .map(|e| (EventId::from_slice(e.id).unwrap(), e.created_at))
            .collect();
        txn.commit()?;
        Ok(items)
    }

    pub async fn delete(&self, filter: Filter) -> Result<(), Error> {
        let (item, rx) = IngesterItem::delete_with_feedback(filter);

        // Send to the ingester
        self.ingester.send(item).map_err(|_| Error::MpscSend)?;

        // Wait for a reply
        rx.await?
    }

    pub fn wipe(&self) -> Result<(), Error> {
        let mut txn = self.db.write_txn()?;
        self.db.wipe(&mut txn)?;
        txn.commit()?;

        Ok(())
    }
}
