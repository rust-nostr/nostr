// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event Id

use core::fmt;
use core::str::FromStr;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::str::FromStr;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::{String, ToString};
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{fmt, vec};

use bitcoin_hashes::sha256::Hash as Sha256Hash;
use bitcoin_hashes::Hash;
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{Kind, Tag};
use crate::Timestamp;

/// [`EventId`] error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Hex error
    Hex(bitcoin_hashes::hex::Error),
    /// Hash error
    Hash(bitcoin_hashes::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hex(e) => write!(f, "{e}"),
            Self::Hash(e) => write!(f, "{e}"),
        }
    }
}

impl From<bitcoin_hashes::hex::Error> for Error {
    fn from(e: bitcoin_hashes::hex::Error) -> Self {
        Self::Hex(e)
    }
}

impl From<bitcoin_hashes::Error> for Error {
    fn from(e: bitcoin_hashes::Error) -> Self {
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
