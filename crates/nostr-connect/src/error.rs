// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect error

use nostr::event::builder;
use nostr::nips::{nip04, nip46};
use nostr::{key, serde_json, PublicKey};
use thiserror::Error;
use tokio::sync::SetError;

/// Nostr Connect error
#[derive(Debug, Error)]
pub enum Error {
    /// Json
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Keys error
    #[error(transparent)]
    Keys(#[from] key::Error),
    /// Event builder error
    #[error(transparent)]
    Builder(#[from] builder::Error),
    /// NIP04 error
    #[error(transparent)]
    NIP04(#[from] nip04::Error),
    /// NIP46 error
    #[error(transparent)]
    NIP46(#[from] nip46::Error),
    /// Relay
    #[error(transparent)]
    Relay(#[from] nostr_relay_pool::relay::Error),
    /// Pool
    #[error(transparent)]
    Pool(#[from] nostr_relay_pool::pool::Error),
    /// Set user public key error
    #[error(transparent)]
    SetUserPublicKey(#[from] SetError<PublicKey>),
    /// NIP46 response error
    #[error("response error: {0}")]
    Response(String),
    /// Signer public key not found
    #[error("signer public key not found")]
    SignerPublicKeyNotFound,
    /// Request timeout
    #[error("timeout")]
    Timeout,
    /// Unexpected URI
    #[error("unexpected Nostr Connect URI")]
    UnexpectedUri,
    /// Public key not match
    #[error("public key from URI not match the app keys")]
    PublicKeyNotMatchAppKeys,
}
