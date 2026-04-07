// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use alloc::string::{String, ToString};
use core::convert::Infallible;
use core::fmt;

use crate::signer::SignerError;

/// Event error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Error serializing or deserializing JSON data
    Json(String),
    /// Builder error
    Builder(String),
    /// Signer error
    Signer(String),
    /// Hex decode error
    Hex(faster_hex::Error),
    /// Unknown JSON event key
    UnknownKey(String),
    /// Invalid event ID
    InvalidId,
    /// Invalid signature
    InvalidSignature,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => e.fmt(f),
            Self::Builder(e) => e.fmt(f),
            Self::Signer(e) => e.fmt(f),
            Self::Hex(e) => e.fmt(f),
            Self::UnknownKey(key) => write!(f, "Unknown key: {key}"),
            Self::InvalidId => f.write_str("Invalid event ID"),
            Self::InvalidSignature => f.write_str("Invalid signature"),
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

impl From<faster_hex::Error> for Error {
    fn from(e: faster_hex::Error) -> Self {
        Self::Hex(e)
    }
}

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
