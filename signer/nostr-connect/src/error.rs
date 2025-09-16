// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect error

use std::fmt;

use nostr::PublicKey;
use nostr::event::builder;
use nostr::nips::{nip04, nip44, nip46};
use nostr_relay_pool::pool;
use tokio::sync::SetError;

/// Nostr Connect error
#[derive(Debug)]
pub enum Error {
    /// Event builder error
    Builder(builder::Error),
    /// NIP04 error
    NIP04(nip04::Error),
    /// NIP44 error
    NIP44(nip44::Error),
    /// NIP46 error
    NIP46(nip46::Error),
    /// Pool
    Pool(pool::Error),
    /// Set user public key error
    SetUserPublicKey(SetError<PublicKey>),
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
    /// User public key not match
    // TODO: remove these `Box<T>`. Currently clippy return the following warning: "the `Err`-variant returned from this function is very large"
    UserPublicKeyNotMatch {
        /// The expected user public key, sent by the signer
        expected: Box<PublicKey>,
        /// The local set user public key
        local: Box<PublicKey>,
    },
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builder(e) => e.fmt(f),
            Self::NIP04(e) => e.fmt(f),
            Self::NIP44(e) => e.fmt(f),
            Self::NIP46(e) => e.fmt(f),
            Self::Pool(e) => e.fmt(f),
            Self::SetUserPublicKey(e) => e.fmt(f),
            Self::Response(e) => e.fmt(f),
            Self::SignerPublicKeyNotFound => f.write_str("signer public key not found"),
            Self::Timeout => f.write_str("timeout"),
            Self::UnexpectedUri => f.write_str("unexpected URI"),
            Self::PublicKeyNotMatchAppKeys => f.write_str("public key not match app keys"),
            Self::UserPublicKeyNotMatch { expected, local } => write!(
                f,
                "user public key not match: expected={expected}, local={local}"
            ),
        }
    }
}

impl From<builder::Error> for Error {
    fn from(e: builder::Error) -> Self {
        Self::Builder(e)
    }
}

impl From<nip04::Error> for Error {
    fn from(e: nip04::Error) -> Self {
        Self::NIP04(e)
    }
}

impl From<nip44::Error> for Error {
    fn from(e: nip44::Error) -> Self {
        Self::NIP44(e)
    }
}

impl From<nip46::Error> for Error {
    fn from(e: nip46::Error) -> Self {
        Self::NIP46(e)
    }
}

impl From<pool::Error> for Error {
    fn from(e: pool::Error) -> Self {
        Self::Pool(e)
    }
}

impl From<SetError<PublicKey>> for Error {
    fn from(e: SetError<PublicKey>) -> Self {
        Self::SetUserPublicKey(e)
    }
}
