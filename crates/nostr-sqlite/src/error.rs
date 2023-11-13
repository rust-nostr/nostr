// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use deadpool_sqlite::{CreatePoolError, InteractError, PoolError};
use nostr_database::{flatbuffers, DatabaseError};
use thiserror::Error;

use crate::migration::MigrationError;

/// Store error
#[derive(Debug, Error)]
pub enum Error {
    /// Sqlite error
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// Pool error
    #[error(transparent)]
    CreateDeadPool(#[from] CreatePoolError),
    /// Pool error
    #[error(transparent)]
    DeadPool(#[from] PoolError),
    /// Pool error
    #[error("{0}")]
    DeadPoolInteract(String),
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
    Url(#[from] nostr::url::ParseError),
    /// Not found
    #[error("sqlite: {0} not found")]
    NotFound(String),
}

impl From<InteractError> for Error {
    fn from(e: InteractError) -> Self {
        Self::DeadPoolInteract(e.to_string())
    }
}

impl From<Error> for DatabaseError {
    fn from(e: Error) -> Self {
        Self::backend(e)
    }
}
