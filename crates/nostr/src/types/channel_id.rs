// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Channel Id

use std::fmt;

#[cfg(feature = "nip19")]
use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::hashes::hex::FromHex;
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
use serde::{Deserialize, Serialize};

#[cfg(feature = "nip19")]
use crate::nips::nip19::{FromBech32, ToBech32, PREFIX_BECH32_CHANNEL};
use crate::EventId;

/// [`ChannelId`] error
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Bech32 error.
    #[cfg(feature = "nip19")]
    #[error(transparent)]
    Bech32(#[from] bech32::Error),
    /// Invalid bech32 channel.
    #[cfg(feature = "nip19")]
    #[error("Invalid bech32 channel")]
    Bech32ParseError,
    /// Hex error
    #[error(transparent)]
    Hex(#[from] bitcoin::hashes::hex::Error),
    /// Hash error
    #[error(transparent)]
    Hash(#[from] bitcoin::hashes::Error),
}

/// Channel Id
///
/// Kind 40 event id (32-bytes lowercase hex-encoded)
///
/// https://github.com/nostr-protocol/nips/blob/master/19.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChannelId(Sha256Hash);

impl ChannelId {
    /// [`ChannelId`] hex string
    pub fn from_hex<S>(hex: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(Self(Sha256Hash::from_hex(&hex.into())?))
    }

    /// [`ChannelId`] from bytes
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

    /// Get [`ChannelId`] as [`Sha256Hash`]
    pub fn inner(&self) -> Sha256Hash {
        self.0
    }
}

impl AsRef<[u8]> for ChannelId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(feature = "nip19")]
impl FromBech32 for ChannelId {
    type Err = Error;
    fn from_bech32<S>(hash: S) -> Result<Self, Self::Err>
    where
        S: Into<String>,
    {
        let (hrp, data, checksum) = bech32::decode(&hash.into())?;

        if hrp != PREFIX_BECH32_CHANNEL || checksum != Variant::Bech32 {
            return Err(Error::Bech32ParseError);
        }

        let data = Vec::<u8>::from_base32(&data)?;
        Self::from_slice(data.as_slice())
    }
}

#[cfg(feature = "nip19")]
impl ToBech32 for ChannelId {
    type Err = Error;
    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_CHANNEL,
            data,
            Variant::Bech32,
        )?)
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "nip19")]
        match self.to_bech32() {
            Ok(r) => write!(f, "{r}"),
            Err(_) => write!(f, "{}", self.to_hex()),
        }

        #[cfg(not(feature = "nip19"))]
        write!(f, "{}", self.to_hex())
    }
}

impl From<ChannelId> for EventId {
    fn from(value: ChannelId) -> Self {
        Self::from(value.inner())
    }
}

impl From<EventId> for ChannelId {
    fn from(value: EventId) -> Self {
        Self(value.inner())
    }
}
