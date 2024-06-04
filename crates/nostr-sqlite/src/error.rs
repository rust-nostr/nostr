// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_database::{flatbuffers, DatabaseError};
use thiserror::Error;
use tokio::task::JoinError;

use crate::migration::MigrationError;

/// Store error
#[derive(Debug, Error)]
pub enum Error {
    /// Sqlite error
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// Pool error
    #[error(transparent)]
    JoinError(#[from] JoinError),
    /// Migration error
    #[error(transparent)]
    Migration(#[from] MigrationError),
    /// Database error
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// Flatbuffers error
    #[error(transparent)]
    Flatbuffers(#[from] flatbuffers::Error),
    /// Url error
    #[error(transparent)]
    Url(#[from] nostr::types::url::ParseError),
    /// Not found
    #[error("sqlite: {0} not found")]
    NotFound(String),
}

impl From<Error> for DatabaseError {
    fn from(e: Error) -> Self {
        Self::backend(e)
    }
}
