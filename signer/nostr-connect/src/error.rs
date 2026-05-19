// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect error

use std::fmt;

use nostr::PublicKey;
use nostr_sdk::client;
use tokio::sync::SetError;

/// Nostr Connect error
#[derive(Debug)]
pub enum Error {
    /// Nostr protocol error
    Protocol(nostr::error::Error),
    /// Client
    Client(client::Error),
    /// Set user public key error
    SetUserPublicKey(SetError<PublicKey>),
    /// Invalid response from remote signer
    InvalidResponse(String),
    /// NIP46 response error
    Response(String),
    /// Signer public key not found
    SignerPublicKeyNotFound,
    /// Request timeout
    Timeout,
    /// Unexpected URI
    UnexpectedUri,
    /// Public key not match
    PublicKeyNotMatchAppKeys,
    /// Nostr connect client without a secret
    NoClientSecret,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::Client(e) => e.fmt(f),
            Self::SetUserPublicKey(e) => e.fmt(f),
            Self::InvalidResponse(e) => e.fmt(f),
            Self::Response(e) => e.fmt(f),
            Self::SignerPublicKeyNotFound => f.write_str("signer public key not found"),
            Self::Timeout => f.write_str("timeout"),
            Self::UnexpectedUri => f.write_str("unexpected URI"),
            Self::PublicKeyNotMatchAppKeys => f.write_str("public key not match app keys"),
            Self::NoClientSecret => f.write_str("missing client secret"),
        }
    }
}

impl From<nostr::error::Error> for Error {
    fn from(e: nostr::error::Error) -> Self {
        Self::Protocol(e)
    }
}

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        Self::Client(e)
    }
}

impl From<SetError<PublicKey>> for Error {
    fn from(e: SetError<PublicKey>) -> Self {
        Self::SetUserPublicKey(e)
    }
}
