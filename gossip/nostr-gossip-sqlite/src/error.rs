//! Gossip SQLite error

use std::fmt;
use std::num::TryFromIntError;

pub use nostr_gossip::error::{Error, ErrorKind};
use tokio::task::JoinError;

/// Migration error
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
    TryFromInt(TryFromIntError),
    Rusqlite(rusqlite::Error),
    Migration(MigrationError),
    Thread(JoinError),
}

impl std::error::Error for StoreError {}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryFromInt(e) => write!(f, "{e}"),
            Self::Rusqlite(e) => write!(f, "{e}"),
            Self::Migration(e) => write!(f, "Migration error: {e}"),
            Self::Thread(e) => write!(f, "{e}"),
        }
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

impl From<StoreError> for Error {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Migration(e) => Self::migration(e),
            StoreError::Thread(e) => Self::new(ErrorKind::IO, e),
            e => Self::storage(e),
        }
    }
}
