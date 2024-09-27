// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fs;
use std::path::Path;
use std::sync::Arc;

use heed::{RoTxn, RwTxn};
use nostr::prelude::*;
use nostr_database::FlatBufferBuilder;
use tokio::sync::Mutex;

mod error;
mod lmdb;
mod types;

use self::error::Error;
use self::lmdb::Lmdb;

#[derive(Debug)]
pub struct Store {
    db: Lmdb,
    fbb: Arc<Mutex<FlatBufferBuilder<'static>>>,
}

impl Store {
    pub fn open<P>(path: P) -> Result<Store, Error>
    where
        P: AsRef<Path>,
    {
        let path: &Path = path.as_ref();

        // Create the directory if it doesn't exist
        fs::create_dir_all(path)?;

        Ok(Store {
            db: Lmdb::new(path)?,
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::with_capacity(70_000))),
        })
    }

    #[inline]
    async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Lmdb) -> R + Send + 'static,
        R: Send + 'static,
    {
        let db = self.db.clone();
        Ok(tokio::task::spawn_blocking(move || f(db)).await?)
    }

    #[inline]
    async fn interact_with_fbb<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Lmdb, &mut FlatBufferBuilder<'static>) -> R + Send + 'static,
        R: Send + 'static,
    {
        let db = self.db.clone();
        let arc_fbb = self.fbb.clone();
        let mut fbb = arc_fbb.lock_owned().await;
        Ok(tokio::task::spawn_blocking(move || f(db, &mut fbb)).await?)
    }

    // /// Sync the data to disk. This happens periodically, but sometimes it's useful to force
    // /// it.
    // pub fn sync(&self) -> Result<(), Error> {
    //     self.db.sync()?;
    //     Ok(())
    // }

    /// Store an event.
    ///
    /// If the event already exists, you will get a Error::Duplicate
    ///
    /// If the event is ephemeral, it will be stored and you will get an offset, but
    /// it will not be indexed.
    pub async fn store_event(&self, event: &Event) -> Result<bool, Error> {
        if event.kind.is_ephemeral() {
            return Ok(false);
        }

        // TODO: avoid this clone
        let event = event.clone();

        self.interact_with_fbb(move |db, fbb| {
            // Read operations
            {
                // Acquire read transaction
                let txn = db.read_txn()?;

                // Already exists
                if db.has_event(&txn, event.id.as_bytes())? {
                    //return Err(Error::Duplicate);
                    return Ok(false);
                }

                // Reject event if ID was deleted
                if db.is_deleted(&txn, &event.id)? {
                    //return Err(Error::Deleted);
                    return Ok(false);
                }

                // Reject event if ADDR was deleted after it's created_at date
                // (non-parameterized)
                if event.kind.is_replaceable() {
                    let coordinate: Coordinate = Coordinate::new(event.kind, event.pubkey);
                    if let Some(time) = db.when_is_coordinate_deleted(&txn, &coordinate)? {
                        if event.created_at <= time {
                            //return Err(Error::Deleted);
                            return Ok(false);
                        }
                    }
                }

                // Reject event if ADDR was deleted after it's created_at date
                // (parameterized)
                if event.kind.is_parameterized_replaceable() {
                    if let Some(identifier) = event.identifier() {
                        let coordinate: Coordinate =
                            Coordinate::new(event.kind, event.pubkey).identifier(identifier);
                        if let Some(time) = db.when_is_coordinate_deleted(&txn, &coordinate)? {
                            if event.created_at <= time {
                                //return Err(Error::Deleted);
                                return Ok(false);
                            }
                        }
                    }
                }
            }

            // Acquire write transaction
            let mut txn = db.write_txn()?;

            // Pre-remove replaceable events being replaced
            {
                if event.kind.is_replaceable() {
                    // Pre-remove any replaceable events that this replaces
                    db.remove_replaceable(&mut txn, &event.pubkey, event.kind, event.created_at)?;

                    // If any remaining matching replaceable events exist, then
                    // this event is invalid, return Replaced
                    if db
                        .find_replaceable_event(&txn, &event.pubkey, event.kind)?
                        .is_some()
                    {
                        //return Err(Error::Replaced);
                        return Ok(false);
                    }
                }

                if event.kind.is_parameterized_replaceable() {
                    if let Some(identifier) = event.identifier() {
                        let coordinate: Coordinate =
                            Coordinate::new(event.kind, event.pubkey).identifier(identifier);

                        // Pre-remove any parameterized-replaceable events that this replaces
                        db.remove_parameterized_replaceable(
                            &mut txn,
                            &coordinate,
                            Timestamp::max(),
                        )?;

                        // If any remaining matching parameterized replaceable events exist, then
                        // this event is invalid, return Replaced
                        if db
                            .find_parameterized_replaceable_event(&txn, &coordinate)?
                            .is_some()
                        {
                            //return Err(Error::Replaced);
                            return Ok(false);
                        }
                    }
                }
            }

            // Store and index the event
            db.store(&mut txn, fbb, &event)?;

            // Handle deletion events
            if let Kind::EventDeletion = event.kind {
                Self::handle_deletion_event(&db, &mut txn, &event)?;
            }

            txn.commit()?;

            Ok(true)
        })
        .await?
    }

    fn handle_deletion_event(db: &Lmdb, txn: &mut RwTxn, event: &Event) -> Result<(), Error> {
        for id in event.event_ids() {
            // Actually remove
            if let Some(target) = db.get_event_by_id(txn, id.as_bytes())? {
                // author must match
                if target.author() != &event.pubkey.to_bytes() {
                    continue;
                }

                // Remove event
                db.remove_by_id(txn, id.as_bytes())?;
            }

            // Mark deleted
            // NOTE: if we didn't have the target event, we presume this is valid,
            //       and if not, clients will just have to deal with that.
            db.mark_deleted(txn, id)?;
        }

        for coordinate in event.coordinates() {
            if coordinate.public_key != event.pubkey {
                continue;
            }

            // Mark deleted
            db.mark_coordinate_deleted(txn, coordinate, event.created_at)?;

            // Remove events (up to the created_at of the deletion event)
            if coordinate.kind.is_replaceable() {
                db.remove_replaceable(
                    txn,
                    &coordinate.public_key,
                    coordinate.kind,
                    event.created_at,
                )?;
            } else if coordinate.kind.is_parameterized_replaceable() {
                db.remove_parameterized_replaceable(txn, coordinate, event.created_at)?;
            }
        }

        Ok(())
    }

    /// Get an event by ID
    pub async fn get_event_by_id(&self, id: &EventId) -> Result<Option<Event>, Error> {
        let bytes = id.to_bytes();
        self.interact(move |db| {
            let txn = db.read_txn()?;
            match db.get_event_by_id(&txn, &bytes)? {
                Some(e) => Ok(Some(e.to_event()?)),
                None => Ok(None),
            }
        })
        .await?
    }

    /// Do we have an event
    pub async fn has_event(&self, id: &EventId) -> Result<bool, Error> {
        let bytes = id.to_bytes();
        self.interact(move |db| {
            let txn = db.read_txn()?;
            db.has_event(&txn, &bytes)
        })
        .await?
    }

    /// Is the event deleted
    pub async fn event_is_deleted(&self, id: EventId) -> Result<bool, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            db.is_deleted(&txn, &id)
        })
        .await?
    }

    #[inline]
    pub async fn when_is_coordinate_deleted(
        &self,
        coordinate: Coordinate,
    ) -> Result<Option<Timestamp>, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            db.when_is_coordinate_deleted(&txn, &coordinate)
        })
        .await?
    }

    pub async fn count(&self, filters: Vec<Filter>) -> Result<usize, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let output = db.query(&txn, filters)?;
            Ok(output.len())
        })
        .await?
    }

    pub async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Error> {
        self.interact(move |db| {
            let txn: RoTxn = db.read_txn()?;
            let output = db.query(&txn, filters)?;
            Ok(output
                .into_iter()
                .filter_map(|e| e.to_event().ok())
                .collect())
        })
        .await?
    }

    pub async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let events = db.single_filter_query(&txn, filter)?;
            Ok(events
                .into_iter()
                .map(|e| (EventId::from_byte_array(*e.id()), e.created_at))
                .collect())
        })
        .await?
    }

    pub async fn delete(&self, filter: Filter) -> Result<(), Error> {
        self.interact(move |db| {
            let read_txn = db.read_txn()?;
            let events = db.single_filter_query(&read_txn, filter)?;

            let mut txn = db.write_txn()?;
            for event in events.into_iter() {
                db.remove(&mut txn, &event)?;
            }
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
