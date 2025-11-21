// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fs;
use std::path::{Path, PathBuf};

use async_utility::task;
use flume::Sender;
use heed::RoTxn;
use nostr_database::prelude::*;

mod error;
mod filter;
mod ingester;
mod lmdb;

use self::error::Error;
use self::ingester::{Ingester, IngesterItem};
use self::lmdb::Lmdb;

#[derive(Debug)]
pub(super) struct Store {
    db: Lmdb,
    ingester: Sender<IngesterItem>,
}

impl Store {
    pub(super) async fn open<P>(
        path: P,
        map_size: usize,
        max_readers: u32,
        additional_dbs: u32,
    ) -> Result<Store, Error>
    where
        P: AsRef<Path>,
    {
        let path: PathBuf = path.as_ref().to_path_buf();

        // Open the database in a blocking task
        let db: Lmdb = task::spawn_blocking(move || {
            // Create the directory if it doesn't exist
            fs::create_dir_all(&path)?;

            let db: Lmdb = Lmdb::new(path, map_size, max_readers, additional_dbs)?;

            Ok::<Lmdb, Error>(db)
        })
        .await??;

        // Run the ingester
        let ingester: Sender<IngesterItem> = Ingester::run(db.clone());

        Ok(Self { db, ingester })
    }

    #[inline]
    async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Lmdb) -> R + Send + 'static,
        R: Send + 'static,
    {
        // TODO: is this clone cheap?
        let db = self.db.clone();
        Ok(task::spawn_blocking(move || f(db)).await?)
    }

    pub(super) async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, Error> {
        let (item, rx) = IngesterItem::save_event_with_feedback(event.clone());
        self.ingester.send(item).map_err(|_| Error::FlumeSend)?;
        rx.await?
    }

    pub(super) async fn get_event_by_id(&self, id: EventId) -> Result<Option<Event>, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let event: Option<Event> = db
                .get_event_by_id(&txn, id.as_bytes())?
                .map(|e| e.into_owned());
            txn.commit()?;
            Ok(event)
        })
        .await?
    }

    pub(super) async fn check_id(&self, id: EventId) -> Result<DatabaseEventStatus, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;

            let status: DatabaseEventStatus = if db.is_deleted(&txn, &id)? {
                DatabaseEventStatus::Deleted
            } else if db.has_event(&txn, &id)? {
                DatabaseEventStatus::Saved
            } else {
                DatabaseEventStatus::NotExistent
            };

            txn.commit()?;

            Ok(status)
        })
        .await?
    }

    pub(super) async fn count(&self, filter: Filter) -> Result<usize, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let output = db.query(&txn, filter)?;
            let len: usize = output.count();
            txn.commit()?;
            Ok(len)
        })
        .await?
    }

    // Lookup ID: EVENT_ORD_IMPL
    pub(super) async fn query(&self, filter: Filter) -> Result<Events, Error> {
        self.interact(move |db| {
            let mut events: Events = Events::new(&filter);

            let txn: RoTxn = db.read_txn()?;
            let output = db.query(&txn, filter)?;
            events.extend(output.into_iter().map(|e| e.into_owned()));
            txn.commit()?;

            Ok(events)
        })
        .await?
    }

    pub(super) async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Error> {
        let txn = self.db.read_txn()?;
        let events = self.db.query(&txn, filter)?;
        let items = events
            .into_iter()
            .map(|e| (EventId::from_byte_array(*e.id), e.created_at))
            .collect();
        txn.commit()?;
        Ok(items)
    }

    pub(super) async fn delete(&self, filter: Filter) -> Result<(), Error> {
        let (item, rx) = IngesterItem::delete_with_feedback(filter);
        self.ingester.send(item).map_err(|_| Error::FlumeSend)?;
        rx.await?
    }

    pub(super) async fn wipe(&self) -> Result<(), Error> {
        let (item, rx) = IngesterItem::wipe_with_feedback();
        self.ingester.send(item).map_err(|_| Error::FlumeSend)?;
        rx.await?
    }
}
