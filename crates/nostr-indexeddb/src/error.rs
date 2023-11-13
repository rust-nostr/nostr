// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_database::DatabaseError;
use thiserror::Error;

/// IndexedDB error
#[derive(Debug, Error)]
pub enum IndexedDBError {
    /// DOM error
    #[error("DomException {name} ({code}): {message}")]
    DomException {
        /// DomException code
        code: u16,
        /// Specific name of the DomException
        name: String,
        /// Message given to the DomException
        message: String,
    },
    /// Database error
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

impl From<indexed_db_futures::web_sys::DomException> for IndexedDBError {
    fn from(frm: indexed_db_futures::web_sys::DomException) -> Self {
        Self::DomException {
            name: frm.name(),
            message: frm.message(),
            code: frm.code(),
        }
    }
}

impl From<IndexedDBError> for DatabaseError {
    fn from(e: IndexedDBError) -> Self {
        Self::backend(e)
    }
}
