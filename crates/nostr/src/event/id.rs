// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event Id

use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::XOnlyPublicKey;

use serde_json::{json, Value};

use super::{Kind, Tag};
use crate::Timestamp;

/// [`EventId`] error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Hex error
    Hex(bitcoin::hashes::hex::Error),
    /// Hash error
    Hash(bitcoin::hashes::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hex(e) => write!(f, "Hex: {e}"),
            Self::Hash(e) => write!(f, "Hash: {e}"),
        }
    }
}

impl From<bitcoin::hashes::hex::Error> for Error {
    fn from(e: bitcoin::hashes::hex::Error) -> Self {
        Self::Hex(e)
    }
}

impl From<bitcoin::hashes::Error> for Error {
    fn from(e: bitcoin::hashes::Error) -> Self {
        Self::Hash(e)
    }
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
        Ok(Self(Sha256Hash::from_str(&hex.into())?))
    }

    /// [`EventId`] from bytes
    pub fn from_slice(sl: &[u8]) -> Result<Self, Error> {
        Ok(Self(Sha256Hash::from_slice(sl)?))
    }

    /// [`EventId`] from hash
    pub fn from_hash(hash: Sha256Hash) -> Self {
        Self(hash)
    }

    /// All zeros
    pub fn all_zeros() -> Self {
        Self(Sha256Hash::all_zeros())
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

impl FromStr for EventId {
    type Err = Error;
    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Self::from_hex(hex)
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
