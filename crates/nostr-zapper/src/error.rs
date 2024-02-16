// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Zapper Error

use thiserror::Error;

/// Zapper Error
#[derive(Debug, Error)]
pub enum ZapperError {
    /// An error happened in the underlying zapper backend.
    #[error("zapper: {0}")]
    Backend(Box<dyn std::error::Error + Send + Sync>),
    /// Nostr error
    #[error("nostr: {0}")]
    Nostr(Box<dyn std::error::Error + Send + Sync>),
    /// Not supported
    #[error("method not supported by current backend")]
    NotSupported,
    /// Feature disabled
    #[error("feature disabled for current backend")]
    FeatureDisabled,
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

    /// Create a new `Nostr` error.
    ///
    /// Shorthand for `Error::Nostr(Box::new(error))`.
    #[inline]
    pub fn nostr<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Nostr(Box::new(error))
    }
}
