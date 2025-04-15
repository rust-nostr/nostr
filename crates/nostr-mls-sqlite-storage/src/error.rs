/// Error types for the SQLite storage implementation.
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Error type for SQLite storage operations.
#[derive(Debug)]
pub enum Error {
    /// SQLite database error
    Database(String),

    /// Error from rusqlite
    Rusqlite(rusqlite::Error),

    /// Error during database migration
    Migration(String),

    /// Error from OpenMLS
    OpenMls(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Database(msg) => write!(f, "Database error: {}", msg),
            Self::Rusqlite(err) => write!(f, "SQLite error: {}", err),
            Self::Migration(msg) => write!(f, "Migration error: {}", msg),
            Self::OpenMls(msg) => write!(f, "OpenMLS error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Self::Rusqlite(err)
    }
}

impl From<refinery::Error> for Error {
    fn from(err: refinery::Error) -> Self {
        Self::Migration(err.to_string())
    }
}
