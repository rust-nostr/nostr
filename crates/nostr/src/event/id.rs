// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Event Id

use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
use serde_json::{json, Value};

use super::{Kind, Tag};
use crate::nips::nip13;
use crate::nips::nip19::FromBech32;
use crate::{PublicKey, Timestamp};

/// [`EventId`] error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Hex error
    Hex(bitcoin::hashes::hex::Error),
    /// Hash error
    Hash(bitcoin::hashes::Error),
    /// Invalid event ID
    InvalidEventId,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hex(e) => write!(f, "Hex: {e}"),
            Self::Hash(e) => write!(f, "Hash: {e}"),
            Self::InvalidEventId => write!(f, "Invalid event ID"),
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
        public_key: &PublicKey,
        created_at: Timestamp,
        kind: &Kind,
        tags: &[Tag],
        content: &str,
    ) -> Self {
        let json: Value = json!([0, public_key, created_at, kind, tags, content]);
        let event_str: String = json.to_string();
        Self(Sha256Hash::hash(event_str.as_bytes()))
    }

    /// Try to parse [EventId] from `hex` or `bech32`
    pub fn parse<S>(id: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let id: &str = id.as_ref();
        match Self::from_hex(id) {
            Ok(id) => Ok(id),
            Err(_) => match Self::from_bech32(id) {
                Ok(id) => Ok(id),
                Err(_) => Err(Error::InvalidEventId),
            },
        }
    }

    /// [`EventId`] hex string
    #[inline]
    pub fn from_hex<S>(hex: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Ok(Self(Sha256Hash::from_str(hex.as_ref())?))
    }

    /// [`EventId`] from bytes
    #[inline]
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

    /// Consume and get bytes
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_byte_array()
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        self.0.to_string()
    }

    /// Check POW
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    pub fn check_pow(&self, difficulty: u8) -> bool {
        nip13::get_leading_zero_bits(self.as_bytes()) >= difficulty
    }

    /// Get [`EventId`] as [`Sha256Hash`]
    pub fn inner(&self) -> Sha256Hash {
        self.0
    }
}

impl FromStr for EventId {
    type Err = Error;

    /// Try to parse [EventId] from `hex` or `bech32`
    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::parse(id)
    }
}

impl AsRef<[u8]> for EventId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::LowerHex for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

impl From<Sha256Hash> for EventId {
    fn from(hash: Sha256Hash) -> Self {
        Self(hash)
    }
}

impl From<EventId> for Tag {
    fn from(event_id: EventId) -> Self {
        Tag::event(event_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_pow() {
        let id =
            EventId::from_hex("2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45")
                .unwrap();
        assert!(!id.check_pow(16));

        // POW 20
        let id =
            EventId::from_hex("00000340cb60be5829fbf2712a285f12cf89e5db951c5303b731651f0d71ac1b")
                .unwrap();
        assert!(id.check_pow(16));
        assert!(id.check_pow(20));
        assert!(!id.check_pow(25));
    }
}
