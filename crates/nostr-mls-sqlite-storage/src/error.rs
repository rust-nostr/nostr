//! Error types for the SQLite storage implementation.

use std::fmt;

/// Error type for SQLite storage operations.
#[derive(Debug)]
pub enum Error {
    /// SQLite database error
    Database(String),
    /// Error from rusqlite
    Rusqlite(rusqlite::Error),
    /// Error during database migration
    Refinery(refinery::Error),
    /// Error from OpenMLS
    OpenMls(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "Database error: {}", msg),
            Self::Rusqlite(err) => write!(f, "SQLite error: {}", err),
            Self::Refinery(msg) => write!(f, "Migration error: {}", msg),
            Self::OpenMls(msg) => write!(f, "OpenMLS error: {}", msg),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Rusqlite(e)
    }
}

impl From<refinery::Error> for Error {
    fn from(e: refinery::Error) -> Self {
        Self::Refinery(e)
    }
}

impl From<Error> for rusqlite::Error {
    fn from(err: Error) -> Self {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err.to_string(),
            )),
        )
    }
}
