// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use async_utility::task;
use heed::{RoTxn, RwTxn};
use nostr_database::prelude::*;
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

    // TODO: spawn an ingester and remove the `fbb` field (use it in the ingester without mutex)?

    #[inline]
    async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Lmdb) -> R + Send + 'static,
        R: Send + 'static,
    {
        let db = self.db.clone();
        Ok(task::spawn_blocking(move || f(db)).await?)
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
        Ok(task::spawn_blocking(move || f(db, &mut fbb)).await?)
    }

    /// Store an event.
    pub async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, Error> {
        if event.kind.is_ephemeral() {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Ephemeral));
        }

        // TODO: avoid this clone
        let event = event.clone();

        self.interact_with_fbb(move |db, fbb| {
            // Acquire read transaction
            let read_txn = db.read_txn()?;

            // Already exists
            if db.has_event(&read_txn, event.id.as_bytes())? {
                return Ok(SaveEventStatus::Rejected(RejectedReason::Duplicate));
            }

            // Reject event if ID was deleted
            if db.is_deleted(&read_txn, &event.id)? {
                return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
            }

            // Reject event if ADDR was deleted after it's created_at date
            // (non-parameterized)
            if event.kind.is_replaceable() {
                let coordinate: Coordinate = Coordinate::new(event.kind, event.pubkey);
                if let Some(time) = db.when_is_coordinate_deleted(&read_txn, &coordinate)? {
                    if event.created_at <= time {
                        return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
                    }
                }
            }

            // Reject event if ADDR was deleted after it's created_at date
            // (parameterized)
            if event.kind.is_addressable() {
                if let Some(identifier) = event.tags.identifier() {
                    let coordinate: Coordinate =
                        Coordinate::new(event.kind, event.pubkey).identifier(identifier);
                    if let Some(time) = db.when_is_coordinate_deleted(&read_txn, &coordinate)? {
                        if event.created_at <= time {
                            return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
                        }
                    }
                }
            }

            // Acquire write transaction
            let mut txn = db.write_txn()?;

            // Remove replaceable events being replaced
            if event.kind.is_replaceable() {
                // Find replaceable event
                if let Some(stored) =
                    db.find_replaceable_event(&read_txn, &event.pubkey, event.kind)?
                {
                    if stored.created_at > event.created_at {
                        txn.abort();
                        return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                    }

                    let coordinate: Coordinate = Coordinate::new(event.kind, event.pubkey);
                    db.remove_replaceable(&read_txn, &mut txn, &coordinate, event.created_at)?;
                }
            }

            // Remove parameterized replaceable events being replaced
            if event.kind.is_addressable() {
                if let Some(identifier) = event.tags.identifier() {
                    let coordinate: Coordinate =
                        Coordinate::new(event.kind, event.pubkey).identifier(identifier);

                    // Find param replaceable event
                    if let Some(stored) = db.find_addressable_event(&read_txn, &coordinate)? {
                        if stored.created_at > event.created_at {
                            txn.abort();
                            return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                        }

                        db.remove_addressable(&read_txn, &mut txn, &coordinate, Timestamp::max())?;
                    }
                }
            }

            // Handle deletion events
            if let Kind::EventDeletion = event.kind {
                let invalid: bool = Self::handle_deletion_event(&db, &read_txn, &mut txn, &event)?;

                if invalid {
                    txn.abort();
                    return Ok(SaveEventStatus::Rejected(RejectedReason::InvalidDelete));
                }
            }

            // Store and index the event
            db.store(&mut txn, fbb, &event)?;

            read_txn.commit()?;
            txn.commit()?;

            Ok(SaveEventStatus::Success)
        })
        .await?
    }

    fn handle_deletion_event(
        db: &Lmdb,
        read_txn: &RoTxn,
        txn: &mut RwTxn,
        event: &Event,
    ) -> Result<bool, Error> {
        for id in event.tags.event_ids() {
            if let Some(target) = db.get_event_by_id(read_txn, id.as_bytes())? {
                // Author must match
                if target.pubkey != event.pubkey.as_bytes() {
                    return Ok(true);
                }

                // Mark as deleted and remove event
                db.mark_deleted(txn, id)?;
                db.remove(txn, &target)?;
            }
        }

        for coordinate in event.tags.coordinates() {
            // Author must match
            if coordinate.public_key != event.pubkey {
                return Ok(true);
            }

            // Mark deleted
            db.mark_coordinate_deleted(txn, coordinate, event.created_at)?;

            // Remove events (up to the created_at of the deletion event)
            if coordinate.kind.is_replaceable() {
                db.remove_replaceable(read_txn, txn, coordinate, event.created_at)?;
            } else if coordinate.kind.is_addressable() {
                db.remove_addressable(read_txn, txn, coordinate, event.created_at)?;
            }
        }

        Ok(false)
    }

    /// Get an event by ID
    pub async fn get_event_by_id(&self, id: &EventId) -> Result<Option<Event>, Error> {
        let bytes = id.to_bytes();
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let event: Option<Event> = db.get_event_by_id(&txn, &bytes)?.map(|e| e.into_owned());
            txn.commit()?;
            Ok(event)
        })
        .await?
    }

    /// Do we have an event
    pub async fn has_event(&self, id: &EventId) -> Result<bool, Error> {
        let bytes = id.to_bytes();
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let has: bool = db.has_event(&txn, &bytes)?;
            txn.commit()?;
            Ok(has)
        })
        .await?
    }

    /// Is the event deleted
    pub async fn event_is_deleted(&self, id: EventId) -> Result<bool, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let deleted: bool = db.is_deleted(&txn, &id)?;
            txn.commit()?;
            Ok(deleted)
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
            let when = db.when_is_coordinate_deleted(&txn, &coordinate)?;
            txn.commit()?;
            Ok(when)
        })
        .await?
    }

    pub async fn count(&self, filters: Vec<Filter>) -> Result<usize, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let output = db.query(&txn, filters)?;
            let len: usize = output.len();
            txn.commit()?;
            Ok(len)
        })
        .await?
    }

    // Lookup ID: EVENT_ORD_IMPL
    pub async fn query(&self, filters: Vec<Filter>) -> Result<Events, Error> {
        self.interact(move |db| {
            let mut events: Events = Events::new(&filters);

            let txn: RoTxn = db.read_txn()?;
            let output: BTreeSet<EventBorrow> = db.query(&txn, filters)?;
            events.extend(output.into_iter().map(|e| e.into_owned()));
            txn.commit()?;

            Ok(events)
        })
        .await?
    }

    pub async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let events = db.query(&txn, vec![filter])?;
            let items = events
                .into_iter()
                .map(|e| (EventId::from_byte_array(*e.id), e.created_at))
                .collect();
            txn.commit()?;
            Ok(items)
        })
        .await?
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
