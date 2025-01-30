// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use alloc::string::{String, ToString};
use core::fmt;

use crate::signer::SignerError;
use crate::util::hex;

/// Event error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Error serializing or deserializing JSON data
    Json(String),
    /// Signer error
    Signer(String),
    /// Hex decode error
    Hex(hex::Error),
    /// Unknown JSON event key
    UnknownKey(String),
    /// Invalid event ID
    InvalidId,
    /// Invalid signature
    InvalidSignature,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "{e}"),
            Self::Signer(e) => write!(f, "{e}"),
            Self::Hex(e) => write!(f, "{e}"),
            Self::UnknownKey(key) => write!(f, "Unknown key: {key}"),
            Self::InvalidId => write!(f, "Invalid event ID"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}

impl From<SignerError> for Error {
    fn from(e: SignerError) -> Self {
        Self::Signer(e.to_string())
    }
}

impl From<hex::Error> for Error {
    fn from(e: hex::Error) -> Self {
        Self::Hex(e)
    }
}
