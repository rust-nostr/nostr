//! Error

use std::fmt;
use std::num::TryFromIntError;

use async_utility::tokio::task::JoinError;
use nostr::{event, key, secp256k1};

/// Migration error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// Migration is in a dirty state
    Dirty(i64),
    /// Database version is newer than supported one
    NewerVersion {
        /// Current database version
        current: i64,
        /// Supported database version
        supported: i64,
    },
}

impl std::error::Error for MigrationError {}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dirty(version) => write!(f, "migration {version} is partially applied"),
            Self::NewerVersion { current, supported } => write!(
                f,
                "database version {current} is newer than supported version {supported}"
            ),
        }
    }
}

/// Nostr SQL error
#[derive(Debug)]
pub enum Error {
    /// TryFromInt error
    TryFromInt(TryFromIntError),
    /// Rusqlite error
    Rusqlite(rusqlite::Error),
    /// Migration error
    Migration(MigrationError),
    /// Thread error
    Thread(JoinError),
    /// JSON error
    Json(serde_json::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Event error
    Event(event::Error),
    /// Key error
    Key(key::Error),
    /// Mutex poisoned
    MutexPoisoned,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryFromInt(e) => write!(f, "{e}"),
            Self::Rusqlite(e) => write!(f, "{e}"),
            Self::Migration(e) => write!(f, "Migration error: {e}"),
            Self::Thread(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::MutexPoisoned => f.write_str("mutex is poisoned"),
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Self::TryFromInt(e)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Rusqlite(e)
    }
}

impl From<MigrationError> for Error {
    fn from(e: MigrationError) -> Self {
        Self::Migration(e)
    }
}

impl From<JoinError> for Error {
    fn from(e: JoinError) -> Self {
        Self::Thread(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
    }
}
