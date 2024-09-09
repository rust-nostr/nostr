// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::{key, secp256k1};
use nostr_database::flatbuffers;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum Error {
    /// An upstream I/O error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// An error from LMDB, our upstream storage crate
    #[error(transparent)]
    Lmdb(#[from] heed::Error),
    /// Flatbuffers error
    #[error(transparent)]
    FlatBuffers(#[from] flatbuffers::Error),
    #[error(transparent)]
    Thread(#[from] JoinError),
    #[error(transparent)]
    Key(#[from] key::Error),
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    // /// The event has already been deleted
    // #[error("Event was previously deleted")]
    // Deleted,
    // /// The event duplicates an event we already have
    // #[error("Duplicate event")]
    // Duplicate,
    // /// The delete is invalid (perhaps the author does not match)
    // #[error("Invalid delete event")]
    // InvalidDelete,
    // /// The event was previously replaced
    // #[error("Event was previously replaced")]
    // Replaced,
    /// The event kind is wrong
    #[error("Wrong event kind")]
    WrongEventKind,
    #[error("Not found")]
    NotFound,
}
