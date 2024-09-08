// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Zapper Error

use thiserror::Error;

/// Zapper Error
#[derive(Debug, Error)]
pub enum ZapperError {
    /// An error happened in the underlying zapper backend.
    #[error(transparent)]
    Backend(Box<dyn std::error::Error + Send + Sync>),
    /// Not supported
    #[error("not supported by current backend")]
    NotSupported,
}

impl ZapperError {
    /// Create a new `Backend` error.
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
