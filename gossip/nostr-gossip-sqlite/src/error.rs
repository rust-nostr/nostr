//! Gossip SQLite error

use std::fmt;

/// Gossip SQLite error
#[derive(Debug)]
pub enum Error {
    /// SQLx error
    Sqlx(sqlx::Error),
    /// SQLx migration error
    Migrate(sqlx::migrate::MigrateError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlx(e) => e.fmt(f),
            Self::Migrate(e) => e.fmt(f),
        }
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
