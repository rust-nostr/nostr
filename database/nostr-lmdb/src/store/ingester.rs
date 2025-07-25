// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use nostr::Event;
use nostr_database::{FlatBufferBuilder, SaveEventStatus};
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
        let read_txn = self.db.read_txn()?;
        let mut write_txn = self.db.write_txn()?;

        let result = self
            .db
            .save_event_with_txn(&read_txn, &mut write_txn, fbb, &event)?;

        match &result {
            SaveEventStatus::Success => {
                write_txn.commit()?;
                read_txn.commit()?;
            }
            SaveEventStatus::Rejected(_) => {
                write_txn.abort();
                read_txn.commit()?;
            }
        }

        Ok(result)
    }
}
