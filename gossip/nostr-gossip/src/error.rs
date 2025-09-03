// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Gossip error

use std::fmt;

/// Gossip Error
#[derive(Debug)]
pub enum GossipError {
    /// An error happened in the underlying database backend.
    Backend(Box<dyn std::error::Error + Send + Sync>),
}

impl std::error::Error for GossipError {}

impl fmt::Display for GossipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(e) => e.fmt(f),
        }
    }
}

impl GossipError {
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
