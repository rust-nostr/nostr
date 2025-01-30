// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Transport Error

use core::fmt;

/// Transport Error
#[derive(Debug)]
pub enum TransportError {
    /// An error happened in the underlying backend.
    Backend(Box<dyn std::error::Error + Send + Sync>),
}

impl std::error::Error for TransportError {}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(e) => write!(f, "{e}"),
        }
    }
}

impl TransportError {
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
