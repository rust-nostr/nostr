// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::{fmt, io};

use async_utility::tokio::task::JoinError;
use nostr::secp256k1;
use nostr_database::flatbuffers;
use tokio::sync::oneshot;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MigrationError {
    /// Database version is newer than supported one
    NewerVersion {
        /// Current version of the database
        current_version: u64,
        /// Newer version of the database
        new_version: u64,
    },
}

impl std::error::Error for MigrationError {}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewerVersion {
                current_version,
                new_version,
            } => write!(
                f,
                "Database version {current_version} is newer than supported version {new_version}."
            ),
        }
    }
}

#[derive(Debug)]
pub(crate) enum StoreError {
    Protocol(nostr::error::Error),
    Io(io::Error),
    Heed(heed::Error),
    FlatBuffers(flatbuffers::Error),
    Thread(JoinError),
    Secp256k1(secp256k1::Error),
    OneshotRecv(oneshot::error::RecvError),
    Migration(MigrationError),
    FlumeSend,
    WrongEventKind,
    NotFound,
    BatchTransactionFailed,
}

impl std::error::Error for StoreError {}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::Io(e) => write!(f, "{e}"),
            Self::Heed(e) => write!(f, "{e}"),
            Self::FlatBuffers(e) => write!(f, "{e}"),
            Self::Thread(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::OneshotRecv(e) => write!(f, "{e}"),
            Self::Migration(e) => write!(f, "Migration error: {e}"),
            Self::FlumeSend => write!(f, "flume channel send error"),
            Self::NotFound => write!(f, "Not found"),
            Self::WrongEventKind => write!(f, "Wrong event kind"),
            Self::BatchTransactionFailed => write!(f, "Batched transaction failed"),
        }
    }
}

impl From<nostr::error::Error> for StoreError {
    fn from(e: nostr::error::Error) -> Self {
        Self::Protocol(e)
    }
}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<heed::Error> for StoreError {
    fn from(e: heed::Error) -> Self {
        Self::Heed(e)
    }
}

impl From<flatbuffers::Error> for StoreError {
    fn from(e: flatbuffers::Error) -> Self {
        Self::FlatBuffers(e)
    }
}

impl From<JoinError> for StoreError {
    fn from(e: JoinError) -> Self {
        Self::Thread(e)
    }
}

impl From<secp256k1::Error> for StoreError {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<oneshot::error::RecvError> for StoreError {
    fn from(e: oneshot::error::RecvError) -> Self {
        Self::OneshotRecv(e)
    }
}

impl From<StoreError> for nostr_database::error::Error {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::Protocol(e) => e.into(),
            StoreError::Io(e) => e.into(),
            StoreError::Migration(e) => Self::migration(e),
            e => Self::storage(e),
        }
    }
}
