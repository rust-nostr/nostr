// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use nostr_database::{flatbuffers, DatabaseError};
use rusqlite::types::FromSqlError;
use tokio::task::JoinError;

/// Store error
#[derive(Debug)]
pub enum Error {
    /// Sqlite error
    Sqlite(rusqlite::Error),
    /// Pool error
    Thread(JoinError),
    /// From SQL error
    FromSql(FromSqlError),
    /// Flatbuffers error
    Flatbuffers(flatbuffers::Error),
    /// Url error
    Url(nostr::types::url::ParseError),
    /// Migration error
    NewerDbVersion { 
        current: usize,
        other: usize,
    },
    /// Not found
    NotFound(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(err) => write!(f, "Sqlite error: {}", err),
            Self::Thread(err) => write!(f, "Thread error: {}", err),
            Self::FromSql(err) => write!(f, "From SQL error: {}", err),
            Self::Flatbuffers(err) => write!(f, "Flatbuffers error: {}", err),
            Self::Url(err) => write!(f, "Url error: {}", err),
            Self::NewerDbVersion { current, other } => write!(f, "Database version is newer than supported by this executable (v{current} > v{other})"),
            Self::NotFound(item) => write!(f, "sqlite: {} not found", item),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Self::Sqlite(err)
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Self::Thread(err)
    }
}


impl From<FromSqlError> for Error {
    fn from(err: FromSqlError) -> Self {
        Self::FromSql(err)
    }
}

impl From<flatbuffers::Error> for Error {
    fn from(err: flatbuffers::Error) -> Self {
        Self::Flatbuffers(err)
    }
}

impl From<nostr::types::url::ParseError> for Error {
    fn from(err: nostr::types::url::ParseError) -> Self {
        Self::Url(err)
    }
}

impl From<Error> for DatabaseError {
    fn from(e: Error) -> Self {
        Self::backend(e)
    }
}
