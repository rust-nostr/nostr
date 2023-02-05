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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChannelId {
    hash: Sha256Hash,
    relays: Vec<String>,
}

impl ChannelId {
    /// New [`ChannelId`]
    pub fn new(hash: Sha256Hash, relays: Vec<String>) -> Self {
        Self { hash, relays }
    }

    /// [`ChannelId`] hex string
    pub fn from_hex<S>(hex: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(Self::new(Sha256Hash::from_hex(&hex.into())?, Vec::new()))
    }

    /// [`ChannelId`] from bytes
    pub fn from_slice(sl: &[u8]) -> Result<Self, Error> {
        Ok(Self::new(Sha256Hash::from_slice(sl)?, Vec::new()))
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        self.hash.to_string()
    }

    /// Get [`ChannelId`] as [`Sha256Hash`]
    pub fn hash(&self) -> Sha256Hash {
        self.hash
    }

    /// Get relays
    pub fn relays(&self) -> Vec<String> {
        self.relays.clone()
    }
}

impl AsRef<[u8]> for ChannelId {
    fn as_ref(&self) -> &[u8] {
        self.hash.as_ref()
    }
}

#[cfg(feature = "nip19")]
impl FromBech32 for ChannelId {
    type Err = Error;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: Into<String>,
    {
        let (hrp, data, checksum) = bech32::decode(&s.into())?;

        if hrp != PREFIX_BECH32_CHANNEL || checksum != Variant::Bech32 {
            return Err(Error::Bech32ParseError);
        }

        let data = Vec::<u8>::from_base32(&data).map_err(|_| Error::Bech32ParseError)?;

        let t = data.first().ok_or(Error::Bech32ParseError)?;
        if *t != 0 {
            return Err(Error::Bech32ParseError);
        }

        let l = data.get(1).ok_or(Error::Bech32ParseError)?;
        if *l != 32 {
            return Err(Error::Bech32ParseError);
        }

        let channel_id = data.get(2..34).ok_or(Error::Bech32ParseError)?;
        let hash = Sha256Hash::from_slice(channel_id)?;

        let mut relays: Vec<String> = Vec::new();
        let mut relays_data: Vec<u8> = data.get(34..).ok_or(Error::Bech32ParseError)?.to_vec();

        while !relays_data.is_empty() {
            let t = relays_data.first().ok_or(Error::Bech32ParseError)?;
            if *t != 1 {
                return Err(Error::Bech32ParseError);
            }

            let l = relays_data.get(1).ok_or(Error::Bech32ParseError)?;
            let l = *l as usize;

            let data = relays_data.get(2..l + 2).ok_or(Error::Bech32ParseError)?;

            relays.push(String::from_utf8(data.to_vec()).map_err(|_| Error::Bech32ParseError)?);
            relays_data.drain(..l + 2);
        }

        Ok(Self::new(hash, relays))
    }
}

#[cfg(feature = "nip19")]
impl ToBech32 for ChannelId {
    type Err = Error;
    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = vec![0, 32];
        bytes.extend(self.hash().iter());

        for relay in self.relays.iter() {
            bytes.extend([1, relay.len() as u8]);
            bytes.extend(relay.as_bytes());
        }

        let data = bytes.to_base32();
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
        Self::from(value.hash())
    }
}

impl From<EventId> for ChannelId {
    fn from(value: EventId) -> Self {
        Self::new(value.inner(), Vec::new())
    }
}

#[cfg(feature = "nip19")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn to_bech32_channel() -> Result<()> {
        let channel_id = ChannelId::from_hex(
            "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d",
        )?;
        assert_eq!(
            "nchannel1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gg07hju".to_string(),
            channel_id.to_bech32()?
        );
        Ok(())
    }

    #[test]
    fn from_bech32_channel() -> Result<()> {
        let channel_id = ChannelId::from_bech32(
            "nchannel1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gg07hju",
        )?;
        assert_eq!(
            "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d".to_string(),
            channel_id.to_hex()
        );
        Ok(())
    }
}
