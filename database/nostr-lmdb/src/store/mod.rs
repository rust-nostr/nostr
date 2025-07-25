// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;

use async_utility::task;
use heed::RoTxn;
use nostr_database::prelude::*;

mod error;
mod ingester;
mod lmdb;
mod types;

use self::error::Error;
use self::ingester::{Ingester, IngesterItem};
use self::lmdb::Lmdb;

#[derive(Debug)]
pub struct Store {
    db: Lmdb,
    ingester: Sender<IngesterItem>,
}

impl Store {
    pub(super) fn open<P>(
        path: P,
        map_size: usize,
        max_readers: u32,
        additional_dbs: u32,
    ) -> Result<Store, Error>
    where
        P: AsRef<Path>,
    {
        let path: &Path = path.as_ref();

        // Create the directory if it doesn't exist
        fs::create_dir_all(path)?;

        let db: Lmdb = Lmdb::new(path, map_size, max_readers, additional_dbs)?;
        let ingester: Sender<IngesterItem> = Ingester::run(db.clone());

        Ok(Self { db, ingester })
    }

    #[inline]
    async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Lmdb) -> R + Send + 'static,
        R: Send + 'static,
    {
        let db = self.db.clone();
        Ok(task::spawn_blocking(move || f(db)).await?)
    }

    /// Store an event.
    pub async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, Error> {
        let (item, rx) = IngesterItem::with_feedback(event.clone());

        // Send to the ingester
        // This will never block the current thread according to `std::sync::mpsc::Sender` docs
        self.ingester.send(item).map_err(|_| Error::MpscSend)?;

        // Wait for a reply
        rx.await?
    }

    /// Get an event by ID
    pub fn get_event_by_id(&self, id: &EventId) -> Result<Option<Event>, Error> {
        let txn = self.db.read_txn()?;
        let event: Option<Event> = self
            .db
            .get_event_by_id(&txn, id.as_bytes())?
            .map(|e| e.into_owned());
        txn.commit()?;
        Ok(event)
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
        let deleted: bool = self.db.is_deleted(&txn, id)?;
        txn.commit()?;
        Ok(deleted)
    }

    pub fn count(&self, filter: Filter) -> Result<usize, Error> {
        let txn = self.db.read_txn()?;
        let output = self.db.query(&txn, filter)?;
        let len: usize = output.count();
        txn.commit()?;
        Ok(len)
    }

    // Lookup ID: EVENT_ORD_IMPL
    pub fn query(&self, filter: Filter) -> Result<Events, Error> {
        let mut events: Events = Events::new(&filter);

        let txn: RoTxn = self.db.read_txn()?;
        let output = self.db.query(&txn, filter)?;
        events.extend(output.into_iter().map(|e| e.into_owned()));
        txn.commit()?;

        Ok(events)
    }

    pub fn negentropy_items(&self, filter: Filter) -> Result<Vec<(EventId, Timestamp)>, Error> {
        let txn = self.db.read_txn()?;
        let events = self.db.query(&txn, filter)?;
        let items = events
            .into_iter()
            .map(|e| (EventId::from_byte_array(*e.id), e.created_at))
            .collect();
        txn.commit()?;
        Ok(items)
    }

    pub async fn delete(&self, filter: Filter) -> Result<(), Error> {
        self.interact(move |db| {
            let read_txn = db.read_txn()?;
            let mut txn = db.write_txn()?;

            db.delete(&read_txn, &mut txn, filter)?;

            read_txn.commit()?;
            txn.commit()?;

            Ok(())
        })
        .await?
    }

    pub async fn wipe(&self) -> Result<(), Error> {
        self.interact(move |db| {
            let mut txn = db.write_txn()?;
            db.wipe(&mut txn)?;
            txn.commit()?;
            Ok(())
        })
        .await?
    }
}
