//! Gossip SQLite error

use std::fmt;
use std::num::TryFromIntError;

use tokio::sync::AcquireError;

/// Gossip SQLite error
#[derive(Debug)]
pub enum Error {
    /// TryFromInt error
    TryFromInt(TryFromIntError),
    /// SQLx error
    Sqlx(sqlx::Error),
    /// SQLx migration error
    Migrate(sqlx::migrate::MigrateError),
    /// Failed to acquire semaphore
    Acquire(AcquireError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryFromInt(e) => e.fmt(f),
            Self::Sqlx(e) => e.fmt(f),
            Self::Migrate(e) => e.fmt(f),
            Self::Acquire(e) => e.fmt(f),
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Self::TryFromInt(err)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::Migrate(err)
    }
}

impl From<AcquireError> for Error {
    fn from(err: AcquireError) -> Self {
        Self::Acquire(err)
    }
}
