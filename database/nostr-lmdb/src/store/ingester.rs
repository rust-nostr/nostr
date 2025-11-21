// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Event ingester for LMDB storage backend
//!
//! This module implements an asynchronous event ingester that processes database operations
//! in the background using a dedicated thread.
//!
//! The ingester provides automatic batching of operations for optimal LMDB write performance.
//! Events are collected from a channel and committed in batches using a single transaction.

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use heed::RwTxn;
use nostr::{Event, Filter};
use nostr_database::{FlatBufferBuilder, SaveEventStatus};
use tokio::sync::oneshot;

use super::error::Error;
use super::lmdb::Lmdb;

/// Pre-allocated buffer size for FlatBufferBuilder
///
/// This size is chosen to handle most Nostr events without reallocation.
/// Large events (with many tags or large content) may still trigger reallocation.
const FLATBUFFER_CAPACITY: usize = 70_000;

enum OperationResult {
    Save {
        result: Result<SaveEventStatus, Error>,
        tx: Option<oneshot::Sender<Result<SaveEventStatus, Error>>>,
    },
    Delete {
        result: Result<(), Error>,
        tx: Option<oneshot::Sender<Result<(), Error>>>,
    },
    Wipe {
        result: Result<(), Error>,
        tx: Option<oneshot::Sender<Result<(), Error>>>,
    },
}

impl OperationResult {
    /// Send the result through the channel if present, or log errors
    fn send(self) {
        match self {
            Self::Save { result, tx } => {
                if let Some(tx) = tx {
                    if tx.send(result).is_err() {
                        tracing::debug!("Failed to send save result: receiver dropped");
                    }
                } else if let Err(e) = result {
                    tracing::error!(error = %e, "Event save failed in batch");
                }
            }
            Self::Delete { result, tx } => {
                if let Some(tx) = tx {
                    if tx.send(result).is_err() {
                        tracing::debug!("Failed to send delete result: receiver dropped");
                    }
                } else if let Err(e) = result {
                    tracing::error!(error = %e, "Delete operation failed in batch");
                }
            }
            Self::Wipe { result, tx } => {
                if let Some(tx) = tx {
                    if tx.send(result).is_err() {
                        tracing::debug!("Failed to send wipe result: receiver dropped");
                    }
                } else if let Err(e) = result {
                    tracing::error!(error = %e, "Wipe operation failed in batch");
                }
            }
        }
    }
}

enum IngesterOperation {
    SaveEvent {
        event: Event,
        tx: Option<oneshot::Sender<Result<SaveEventStatus, Error>>>,
    },
    Delete {
        filter: Filter,
        tx: Option<oneshot::Sender<Result<(), Error>>>,
    },
    Wipe {
        tx: Option<oneshot::Sender<Result<(), Error>>>,
    },
}

impl IngesterOperation {
    /// Create an error result for this operation type, moving the channel
    fn into_error_result(self, error: Error) -> OperationResult {
        match self {
            Self::SaveEvent { tx, .. } => OperationResult::Save {
                result: Err(error),
                tx,
            },
            Self::Delete { tx, .. } => OperationResult::Delete {
                result: Err(error),
                tx,
            },
            Self::Wipe { tx } => OperationResult::Wipe {
                result: Err(error),
                tx,
            },
        }
    }
}

pub(super) struct IngesterItem {
    operation: IngesterOperation,
}

impl IngesterItem {
    #[must_use]
    pub(super) fn save_event_with_feedback(
        event: Event,
    ) -> (Self, oneshot::Receiver<Result<SaveEventStatus, Error>>) {
        let (tx, rx) = oneshot::channel();
        let item: Self = Self {
            operation: IngesterOperation::SaveEvent {
                event,
                tx: Some(tx),
            },
        };
        (item, rx)
    }

    #[must_use]
    pub(super) fn delete_with_feedback(
        filter: Filter,
    ) -> (Self, oneshot::Receiver<Result<(), Error>>) {
        let (tx, rx) = oneshot::channel();
        let item: Self = Self {
            operation: IngesterOperation::Delete {
                filter,
                tx: Some(tx),
            },
        };
        (item, rx)
    }

    #[must_use]
    pub(super) fn wipe_with_feedback() -> (Self, oneshot::Receiver<Result<(), Error>>) {
        let (tx, rx) = oneshot::channel();
        let item: Self = Self {
            operation: IngesterOperation::Wipe { tx: Some(tx) },
        };
        (item, rx)
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
        let ingester: Self = Self { db, rx };
        ingester.spawn_ingester();

        tx
    }

    #[inline]
    fn spawn_ingester(self) {
        thread::spawn(move || self.run_ingester_loop());
    }

    fn run_ingester_loop(self) {
        tracing::debug!("Ingester thread started");

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::with_capacity(FLATBUFFER_CAPACITY);

        // Listen for items
        while let Ok(item) = self.rx.recv() {
            // Process item
            let res: OperationResult = self.process_item(&mut fbb, item);

            // Send result
            res.send();
        }

        tracing::debug!("Ingester thread exited");
    }

    fn process_item(&self, fbb: &mut FlatBufferBuilder, item: IngesterItem) -> OperationResult {
        // Try to get write transaction
        let mut write_txn = match self.db.write_txn() {
            Ok(txn) => txn,
            Err(e) => return item.operation.into_error_result(e),
        };

        // Process operation
        self.process_operation(&mut write_txn, fbb, item.operation)
    }

    fn process_operation(
        &self,
        txn: &mut RwTxn,
        fbb: &mut FlatBufferBuilder,
        operation: IngesterOperation,
    ) -> OperationResult {
        match operation {
            IngesterOperation::SaveEvent { event, tx } => {
                let result = self.db.save_event_with_txn(txn, fbb, &event);
                OperationResult::Save { result, tx }
            }
            IngesterOperation::Delete { filter, tx } => {
                let result = self.db.delete(txn, filter);
                OperationResult::Delete { result, tx }
            }
            IngesterOperation::Wipe { tx } => {
                let result = self.db.wipe(txn);
                OperationResult::Wipe { result, tx }
            }
        }
    }
}
