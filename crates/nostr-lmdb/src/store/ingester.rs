// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use heed::RwTxn;
use nostr::nips::nip01::Coordinate;
use nostr::{Event, Kind, Timestamp};
use nostr_database::{FlatBufferBuilder, RejectedReason, SaveEventStatus};
use tokio::sync::oneshot;

use super::error::Error;
use super::lmdb::Lmdb;

pub(super) struct IngesterItem {
    event: Event,
    tx: Option<oneshot::Sender<Result<SaveEventStatus, Error>>>,
}

impl IngesterItem {
    // #[inline]
    // pub(super) fn without_feedback(event: Event) -> Self {
    //     Self { event, tx: None }
    // }

    #[must_use]
    pub(super) fn with_feedback(
        event: Event,
    ) -> (Self, oneshot::Receiver<Result<SaveEventStatus, Error>>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                event,
                tx: Some(tx),
            },
            rx,
        )
    }
}

#[derive(Debug)]
pub(super) struct Ingester {
    db: Lmdb,
    rx: Receiver<IngesterItem>,
}

impl Ingester {
    /// Build and spawn a new ingester
    pub(super) fn run(db: Lmdb) -> Sender<IngesterItem> {
        // Create new asynchronous channel
        let (tx, rx) = std::sync::mpsc::channel();

        // Construct and spawn ingester
        let ingester = Self { db, rx };
        ingester.spawn_ingester();

        // Return ingester sender
        tx
    }

    fn spawn_ingester(self) {
        thread::spawn(move || {
            #[cfg(debug_assertions)]
            tracing::debug!("Ingester thread started");

            let mut fbb = FlatBufferBuilder::with_capacity(70_000);

            // Listen for items
            while let Ok(IngesterItem { event, tx }) = self.rx.recv() {
                // Ingest
                let res = self.ingest_event(event, &mut fbb);

                // If sender is available send the `Result` otherwise log as error
                match tx {
                    // Send to receiver
                    Some(tx) => {
                        let _ = tx.send(res);
                    }
                    // Log error if `Result::Err`
                    None => {
                        if let Err(e) = res {
                            tracing::error!(error = %e, "Event ingestion failed.");
                        }
                    }
                }
            }

            #[cfg(debug_assertions)]
            tracing::debug!("Ingester thread exited");
        });
    }

    fn ingest_event(
        &self,
        event: Event,
        fbb: &mut FlatBufferBuilder,
    ) -> nostr::Result<SaveEventStatus, Error> {
        if event.kind.is_ephemeral() {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Ephemeral));
        }

        // Initial read txn checks
        {
            // Acquire read txn
            let read_txn = self.db.read_txn()?;

            // Already exists
            if self.db.has_event(&read_txn, event.id.as_bytes())? {
                return Ok(SaveEventStatus::Rejected(RejectedReason::Duplicate));
            }

            // Reject event if ID was deleted
            if self.db.is_deleted(&read_txn, &event.id)? {
                return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
            }

            // Reject event if ADDR was deleted after it's created_at date
            // (non-parameterized or parameterized)
            if let Some(coordinate) = event.coordinate() {
                if let Some(time) = self.db.when_is_coordinate_deleted(&read_txn, &coordinate)? {
                    if event.created_at <= time {
                        return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
                    }
                }
            }

            read_txn.commit()?;
        }

        // Acquire write transaction
        let mut txn = self.db.write_txn()?;

        // Remove replaceable events being replaced
        if event.kind.is_replaceable() {
            // Find replaceable event
            if let Some(stored) = self
                .db
                .find_replaceable_event(&txn, &event.pubkey, event.kind)?
            {
                if stored.created_at > event.created_at {
                    txn.abort();
                    return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                }

                // Acquire read txn
                let read_txn = self.db.read_txn()?;

                let coordinate: Coordinate = Coordinate::new(event.kind, event.pubkey);
                self.db
                    .remove_replaceable(&read_txn, &mut txn, &coordinate, event.created_at)?;

                read_txn.commit()?;
            }
        }

        // Remove parameterized replaceable events being replaced
        if event.kind.is_addressable() {
            if let Some(identifier) = event.tags.identifier() {
                let coordinate: Coordinate =
                    Coordinate::new(event.kind, event.pubkey).identifier(identifier);

                // Find param replaceable event
                if let Some(stored) = self.db.find_addressable_event(&txn, &coordinate)? {
                    if stored.created_at > event.created_at {
                        txn.abort();
                        return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                    }

                    // Acquire read txn
                    let read_txn = self.db.read_txn()?;

                    self.db.remove_addressable(
                        &read_txn,
                        &mut txn,
                        &coordinate,
                        Timestamp::max(),
                    )?;

                    read_txn.commit()?;
                }
            }
        }

        // Handle deletion events
        if let Kind::EventDeletion = event.kind {
            let invalid: bool = self.handle_deletion_event(&mut txn, &event)?;

            if invalid {
                txn.abort();
                return Ok(SaveEventStatus::Rejected(RejectedReason::InvalidDelete));
            }
        }

        // Store and index the event
        self.db.store(&mut txn, fbb, &event)?;

        // Commit
        txn.commit()?;

        Ok(SaveEventStatus::Success)
    }

    fn handle_deletion_event(&self, txn: &mut RwTxn, event: &Event) -> nostr::Result<bool, Error> {
        // Acquire read txn
        let read_txn = self.db.read_txn()?;

        for id in event.tags.event_ids() {
            if let Some(target) = self.db.get_event_by_id(&read_txn, id.as_bytes())? {
                // Author must match
                if target.pubkey != event.pubkey.as_bytes() {
                    return Ok(true);
                }

                // Mark as deleted and remove event
                self.db.mark_deleted(txn, id)?;
                self.db.remove(txn, &target)?;
            }
        }

        for coordinate in event.tags.coordinates() {
            // Author must match
            if coordinate.public_key != event.pubkey {
                return Ok(true);
            }

            // Mark deleted
            self.db
                .mark_coordinate_deleted(txn, &coordinate.borrow(), event.created_at)?;

            // Remove events (up to the created_at of the deletion event)
            if coordinate.kind.is_replaceable() {
                self.db
                    .remove_replaceable(&read_txn, txn, coordinate, event.created_at)?;
            } else if coordinate.kind.is_addressable() {
                self.db
                    .remove_addressable(&read_txn, txn, coordinate, event.created_at)?;
            }
        }

        read_txn.commit()?;

        Ok(false)
    }
}
