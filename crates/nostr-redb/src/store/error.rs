// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::{fmt, io};

use async_utility::task::Error as JoinError;
use nostr::{key, secp256k1};
use nostr_database::flatbuffers;

#[cfg(target_arch = "wasm32")]
use super::core::wasm::Error as WasmError;

#[derive(Debug)]
pub enum Error {
    /// An upstream I/O error
    Io(io::Error),
    /// An error from LMDB
    Redb(redb::DatabaseError),
    /// An error from LMDB
    RedbTx(redb::TransactionError),
    /// An error from LMDB
    RedbTable(redb::TableError),
    /// An error from LMDB
    RedbStorage(redb::StorageError),
    /// An error from LMDB
    RedbCommit(redb::CommitError),
    /// Flatbuffers error
    FlatBuffers(flatbuffers::Error),
    Thread(JoinError),
    Key(key::Error),
    Secp256k1(secp256k1::Error),
    #[cfg(target_arch = "wasm32")]
    Wasm(WasmError),
    /// Mutex poisoned
    MutexPoisoned,
    /// The event kind is wrong
    WrongEventKind,
    /// Not found
    NotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Redb(e) => write!(f, "{e}"),
            Self::RedbTx(e) => write!(f, "{e}"),
            Self::RedbTable(e) => write!(f, "{e}"),
            Self::RedbStorage(e) => write!(f, "{e}"),
            Self::RedbCommit(e) => write!(f, "{e}"),
            Self::FlatBuffers(e) => write!(f, "{e}"),
            Self::Thread(e) => write!(f, "{e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(e) => write!(f, "{e}"),
            Self::MutexPoisoned => write!(f, "mutex poisoned"),
            Self::NotFound => write!(f, "Not found"),
            Self::WrongEventKind => write!(f, "Wrong event kind"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(e: redb::DatabaseError) -> Self {
        Self::Redb(e)
    }
}

impl From<redb::TransactionError> for Error {
    fn from(e: redb::TransactionError) -> Self {
        Self::RedbTx(e)
    }
}

impl From<redb::TableError> for Error {
    fn from(e: redb::TableError) -> Self {
        Self::RedbTable(e)
    }
}

impl From<redb::StorageError> for Error {
    fn from(e: redb::StorageError) -> Self {
        Self::RedbStorage(e)
    }
}

impl From<redb::CommitError> for Error {
    fn from(e: redb::CommitError) -> Self {
        Self::RedbCommit(e)
    }
}

impl From<flatbuffers::Error> for Error {
    fn from(e: flatbuffers::Error) -> Self {
        Self::FlatBuffers(e)
    }
}

impl From<JoinError> for Error {
    fn from(e: JoinError) -> Self {
        Self::Thread(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<WasmError> for Error {
    fn from(e: WasmError) -> Self {
        Self::Wasm(e)
    }
}
