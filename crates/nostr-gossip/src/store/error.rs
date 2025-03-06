// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use async_utility::tokio::task::JoinError;
use nostr::types::url;
use rusqlite::types::FromSqlError;

/// Store error
#[derive(Debug)]
pub enum Error {
    /// Sqlite error
    Sqlite(rusqlite::Error),
    /// Pool error
    Thread(JoinError),
    /// From SQL error
    FromSql(FromSqlError),
    /// Not found
    RelayUrl(url::Error),
    /// Migration error
    NewerDbVersion {
        /// Current version
        current: usize,
        /// Other version
        other: usize,
    },
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(e) => write!(f, "{e}"),
            Self::Thread(e) => write!(f, "{e}"),
            Self::FromSql(e) => write!(f, "{e}"),
            Self::RelayUrl(e) => write!(f, "{e}"),
            Self::NewerDbVersion { current, other } => write!(f, "Database version is newer than supported by this executable (v{current} > v{other})"),
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

impl From<url::Error> for Error {
    fn from(e: url::Error) -> Self {
        Self::RelayUrl(e)
    }
}
