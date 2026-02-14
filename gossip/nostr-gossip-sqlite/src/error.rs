//! Gossip SQLite error

use std::fmt;
use std::num::TryFromIntError;

use tokio::sync::AcquireError;
use tokio::task::JoinError;

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

/// Gossip SQLite error
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
    /// Failed to acquire semaphore
    Acquire(AcquireError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryFromInt(e) => e.fmt(f),
            Self::Rusqlite(e) => e.fmt(f),
            Self::Migration(e) => e.fmt(f),
            Self::Thread(e) => e.fmt(f),
            Self::Acquire(e) => e.fmt(f),
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Self::TryFromInt(err)
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

impl From<AcquireError> for Error {
    fn from(err: AcquireError) -> Self {
        Self::Acquire(err)
    }
}
