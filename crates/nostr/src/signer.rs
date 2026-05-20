// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Signer

use alloc::string::{String, ToString};
use core::fmt::{self, Debug, Display};

/// Nostr Signer error
#[derive(Debug, PartialEq, Eq)]
pub struct SignerError(String);

impl core::error::Error for SignerError {}

impl fmt::Display for SignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl SignerError {
    /// New signer error
    #[inline]
    pub fn backend<E>(error: E) -> Self
    where
        E: Display,
    {
        Self(error.to_string())
    }
}

impl<S> From<S> for SignerError
where
    S: Into<String>,
{
    fn from(error: S) -> Self {
        Self(error.into())
    }
}
