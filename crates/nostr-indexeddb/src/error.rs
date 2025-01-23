// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use nostr_database::DatabaseError;

/// IndexedDB error
#[derive(Debug)]
pub enum IndexedDBError {
    /// DOM error
    DomException {
        /// DomException code
        code: u16,
        /// Specific name of the DomException
        name: String,
        /// Message given to the DomException
        message: String,
    },
    /// Mutex poisoned
    MutexPoisoned,
}

impl std::error::Error for IndexedDBError {}

impl fmt::Display for IndexedDBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DomException {
                name,
                code,
                message,
            } => write!(f, "DomException {name} ({code}): {message}"),
            Self::MutexPoisoned => write!(f, "mutex poisoned"),
        }
    }
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

pub(crate) fn into_err(e: indexed_db_futures::web_sys::DomException) -> DatabaseError {
    let indexed_err: IndexedDBError = e.into();
    DatabaseError::backend(indexed_err)
}
