// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::sync::{Arc, Mutex};

use async_utility::task;
use nostr_database::flatbuffers::FlatBufferDecodeBorrowed;
use nostr_database::prelude::*;
use redb::{ReadTransaction, WriteTransaction};

use crate::store::types::AccessGuardEvent;

mod core;
mod error;
mod types;

use self::core::Db;
use self::error::Error;

type Fbb = Arc<Mutex<FlatBufferBuilder<'static>>>;

#[derive(Debug)]
pub struct Store {
    db: Db,
    fbb: Fbb,
    persistent: bool,
}

impl Store {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn persistent<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            db: Db::persistent(path)?,
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::with_capacity(70_000))),
            persistent: true,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn web(name: &str) -> Result<Self, Error> {
        Ok(Self {
            db: Db::web(name).await?,
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::with_capacity(70_000))),
            persistent: true,
        })
    }

    pub fn in_memory() -> Self {
        Self {
            // SAFETY: newly created database, should never panic.
            db: Db::in_memory().unwrap(),
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::with_capacity(70_000))),
            persistent: false,
        }
    }

    #[inline]
    pub fn is_persistent(&self) -> bool {
        self.persistent
    }

    #[inline]
    async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Db) -> R + Send + 'static,
        R: Send + 'static,
    {
        let db = self.db.clone();
        Ok(task::spawn_blocking(move || f(db)).join().await?)
    }

    #[inline]
    async fn interact_with_fbb<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Db, Fbb) -> R + Send + 'static,
        R: Send + 'static,
    {
        let db = self.db.clone();
        let fbb = self.fbb.clone();
        Ok(task::spawn_blocking(move || f(db, fbb)).join().await?)
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
            // (non-parameterized or parameterized)
            if let Some(coordinate) = event.coordinate() {
                if let Some(time) = db.when_is_coordinate_deleted(&read_txn, &coordinate)? {
                    if event.created_at <= time {
                        return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
                    }
                }
            }

            // Acquire write transaction
            let txn = db.write_txn()?;

            // Remove replaceable events being replaced
            if event.kind.is_replaceable() {
                // Find replaceable event
                if let Some(stored) =
                    db.find_replaceable_event(&read_txn, &event.pubkey, event.kind)?
                {
                    if stored.created_at > event.created_at {
                        txn.abort()?;
                        return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                    }

                    let coordinate: Coordinate = Coordinate::new(event.kind, event.pubkey);
                    db.remove_replaceable(&read_txn, &txn, &coordinate, event.created_at)?;
                }
            }

            // Remove parameterized replaceable events being replaced
            if event.kind.is_addressable() {
                if let Some(identifier) = event.tags.identifier() {
                    let coordinate: Coordinate =
                        Coordinate::new(event.kind, event.pubkey).identifier(identifier);

                    // Find param replaceable event
                    if let Some(stored) =
                        db.find_parameterized_replaceable_event(&read_txn, &coordinate)?
                    {
                        if stored.created_at > event.created_at {
                            txn.abort()?;
                            return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                        }

                        db.remove_parameterized_replaceable(
                            &read_txn,
                            &txn,
                            &coordinate,
                            Timestamp::max(),
                        )?;
                    }
                }
            }

            // Handle deletion events
            if let Kind::EventDeletion = event.kind {
                let invalid: bool = Self::handle_deletion_event(&db, &read_txn, &txn, &event)?;

                if invalid {
                    txn.abort()?;
                    return Ok(SaveEventStatus::Rejected(RejectedReason::InvalidDelete));
                }
            }

            // Acquire lock
            let mut fbb = fbb.lock().map_err(|_| Error::MutexPoisoned)?;

            // Store and index the event
            db.store(&txn, &mut fbb, &event)?;

            // Immediately drop the lock
            drop(fbb);

            // Commit
            read_txn.close()?;
            txn.commit()?;

            Ok(SaveEventStatus::Success)
        })
        .await?
    }

    fn handle_deletion_event(
        db: &Db,
        read_txn: &ReadTransaction,
        txn: &WriteTransaction,
        event: &Event,
    ) -> Result<bool, Error> {
        for id in event.tags.event_ids() {
            if let Some(target) = db.get_event_by_id(read_txn, id.as_bytes())? {
                let value = target.guard.value();
                let temp = EventBorrow::decode(value)?;

                // Author must match
                if temp.pubkey != event.pubkey.as_bytes() {
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
            db.mark_coordinate_deleted(txn, &coordinate.borrow(), event.created_at)?;

            // Remove events (up to the created_at of the deletion event)
            if coordinate.kind.is_replaceable() {
                db.remove_replaceable(read_txn, txn, coordinate, event.created_at)?;
            } else if coordinate.kind.is_addressable() {
                db.remove_parameterized_replaceable(read_txn, txn, coordinate, event.created_at)?;
            }
        }

        Ok(false)
    }

    /// Get an event by ID
    pub async fn get_event_by_id(&self, id: &EventId) -> Result<Option<Event>, Error> {
        let bytes = id.to_bytes();
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let event: Option<Event> = match db.get_event_by_id(&txn, &bytes)? {
                Some(e) => Some(e.to_event()?),
                None => None,
            };
            txn.close()?;
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
            txn.close()?;
            Ok(has)
        })
        .await?
    }

    /// Is the event deleted
    pub async fn event_is_deleted(&self, id: EventId) -> Result<bool, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let deleted: bool = db.is_deleted(&txn, &id)?;
            txn.close()?;
            Ok(deleted)
        })
        .await?
    }

    #[inline]
    pub async fn when_is_coordinate_deleted<'a>(
        &self,
        coordinate: &'a CoordinateBorrow<'a>,
    ) -> Result<Option<Timestamp>, Error> {
        let key: Vec<u8> = core::index::make_coordinate_index_key(coordinate);
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let when = db.when_is_coordinate_deleted_by_key(&txn, key)?;
            txn.close()?;
            Ok(when)
        })
        .await?
    }

    pub async fn count(&self, filters: Vec<Filter>) -> Result<usize, Error> {
        self.interact(move |db| {
            let txn = db.read_txn()?;
            let output = db.query(&txn, filters)?;
            let len: usize = output.len();
            //txn.close()?;
            Ok(len)
        })
        .await?
    }

    // Lookup ID: EVENT_ORD_IMPL
    pub async fn query(&self, filters: Vec<Filter>) -> Result<Events, Error> {
        self.interact(move |db| {
            let mut events: Events = Events::new(&filters);

            let txn: ReadTransaction = db.read_txn()?;
            let output: BTreeSet<AccessGuardEvent> = db.query(&txn, filters)?;
            events.extend(output.into_iter().filter_map(|e| e.to_event().ok()));
            txn.close()?;

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
                .map(|e| (EventId::from_byte_array(e.id), e.created_at))
                .collect();
            txn.close()?;
            Ok(items)
        })
        .await?
    }

    pub async fn delete(&self, filter: Filter) -> Result<(), Error> {
        self.interact(move |db| {
            let read_txn = db.read_txn()?;
            let txn = db.write_txn()?;

            db.delete(&read_txn, &txn, filter)?;

            read_txn.close()?;
            txn.commit()?;

            Ok(())
        })
        .await?
    }

    pub async fn wipe(&self) -> Result<(), Error> {
        self.interact(move |db| {
            let txn = db.write_txn()?;
            db.wipe(&txn)?;
            txn.commit()?;
            Ok(())
        })
        .await?
    }
}
