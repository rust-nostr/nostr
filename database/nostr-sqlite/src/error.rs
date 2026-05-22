//! Error

use std::fmt;
use std::num::TryFromIntError;

use async_utility::tokio::task::JoinError;
use nostr::secp256k1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MigrationError {
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
            Self::NewerVersion { current, supported } => write!(
                f,
                "database version {current} is newer than supported version {supported}"
            ),
        }
    }
}

#[derive(Debug)]
pub(crate) enum StoreError {
    Protocol(nostr::error::Error),
    TryFromInt(TryFromIntError),
    Rusqlite(rusqlite::Error),
    Migration(MigrationError),
    Thread(JoinError),
    Json(serde_json::Error),
    Secp256k1(secp256k1::Error),
}

impl std::error::Error for StoreError {}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::TryFromInt(e) => write!(f, "{e}"),
            Self::Rusqlite(e) => write!(f, "{e}"),
            Self::Migration(e) => write!(f, "Migration error: {e}"),
            Self::Thread(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
        }
    }
}

impl From<nostr::error::Error> for StoreError {
    fn from(e: nostr::error::Error) -> Self {
        Self::Protocol(e)
    }
}

impl From<TryFromIntError> for StoreError {
    fn from(e: TryFromIntError) -> Self {
        Self::TryFromInt(e)
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Rusqlite(e)
    }
}

impl From<MigrationError> for StoreError {
    fn from(e: MigrationError) -> Self {
        Self::Migration(e)
    }
}

impl From<JoinError> for StoreError {
    fn from(e: JoinError) -> Self {
        Self::Thread(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<secp256k1::Error> for StoreError {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<StoreError> for nostr_database::error::Error {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Protocol(e) => e.into(),
            StoreError::Migration(e) => Self::migration(e),
            e => Self::storage(e),
        }
    }
}
