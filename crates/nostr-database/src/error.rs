// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database Error

use std::fmt;

/// Database Error
#[derive(Debug)]
pub enum DatabaseError {
    /// An error happened in the underlying database backend.
    Backend(Box<dyn std::error::Error + Send + Sync>),
    /// Not supported
    NotSupported,
}

impl std::error::Error for DatabaseError {}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(e) => write!(f, "{e}"),
            Self::NotSupported => write!(f, "not supported"),
        }
    }
}

impl DatabaseError {
    /// Create a new backend error
    ///
    /// Shorthand for `Error::Backend(Box::new(error))`.
    #[inline]
    pub fn backend<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Backend(Box::new(error))
    }
}
