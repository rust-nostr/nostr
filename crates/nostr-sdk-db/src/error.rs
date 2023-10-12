// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Database Error

use thiserror::Error;

/// Database Error
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// An error happened in the underlying database backend.
    #[error(transparent)]
    Backend(Box<dyn std::error::Error + Send + Sync>),
    /// Not supported
    #[error("method not supported by current backend")]
    NotSupported,
}

impl DatabaseError {
    /// Create a new [`Backend`][Self::Backend] error.
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
