// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event Id

use std::fmt;

use bitcoin_hashes::hex::FromHex;
use bitcoin_hashes::sha256::Hash as Sha256Hash;
use bitcoin_hashes::Hash;
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{Kind, Tag};
use crate::Timestamp;

/// [`EventId`] error
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Hex error
    #[error(transparent)]
    Hex(#[from] bitcoin_hashes::hex::Error),
    /// Hash error
    #[error(transparent)]
    Hash(#[from] bitcoin_hashes::Error),
}

/// Event Id
///
/// 32-bytes lowercase hex-encoded sha256 of the the serialized event data
///
/// <https://github.com/nostr-protocol/nips/blob/master/01.md>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(Sha256Hash);

impl EventId {
    /// Generate [`EventId`]
    pub fn new(
        pubkey: &XOnlyPublicKey,
        created_at: Timestamp,
        kind: &Kind,
        tags: &[Tag],
        content: &str,
    ) -> Self {
        let json: Value = json!([0, pubkey, created_at, kind, tags, content]);
        let event_str: String = json.to_string();
        Self(Sha256Hash::hash(event_str.as_bytes()))
    }

    /// [`EventId`] hex string
    pub fn from_hex<S>(hex: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(Self(Sha256Hash::from_hex(&hex.into())?))
    }

    /// [`EventId`] from bytes
    pub fn from_slice(sl: &[u8]) -> Result<Self, Error> {
        Ok(Self(Sha256Hash::from_slice(sl)?))
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        self.0.to_string()
    }

    /// Get [`EventId`] as [`Sha256Hash`]
    pub fn inner(&self) -> Sha256Hash {
        self.0
    }
}

impl AsRef<[u8]> for EventId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<Sha256Hash> for EventId {
    fn from(hash: Sha256Hash) -> Self {
        Self(hash)
    }
}

impl From<EventId> for String {
    fn from(event_id: EventId) -> Self {
        event_id.to_string()
    }
}
