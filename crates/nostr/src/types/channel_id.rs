// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Channel Id

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;

use crate::nips::nip19::{
    Error as Bech32Error, FromBech32, ToBech32, PREFIX_BECH32_CHANNEL, RELAY, SPECIAL,
};
use crate::EventId;

/// [`ChannelId`] error
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

/// Channel Id
///
/// Kind 40 event id (32-bytes lowercase hex-encoded)
///
/// <https://github.com/nostr-protocol/nips/blob/master/19.md>
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
        Ok(Self::new(Sha256Hash::from_str(&hex.into())?, Vec::new()))
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

impl FromBech32 for ChannelId {
    type Err = Bech32Error;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: Into<String>,
    {
        let (hrp, data, checksum) = bech32::decode(&s.into())?;

        if hrp != PREFIX_BECH32_CHANNEL || checksum != Variant::Bech32 {
            return Err(Bech32Error::WrongPrefixOrVariant);
        }

        let mut data: Vec<u8> = Vec::from_base32(&data)?;

        let mut hash: Option<Sha256Hash> = None;
        let mut relays: Vec<String> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Bech32Error::TLV)?;
            let l = data.get(1).ok_or(Bech32Error::TLV)?;
            let l = *l as usize;

            let bytes = data.get(2..l + 2).ok_or(Bech32Error::TLV)?;

            match *t {
                SPECIAL => {
                    if hash.is_none() {
                        hash = Some(Sha256Hash::from_slice(bytes)?);
                    }
                }
                RELAY => {
                    relays.push(String::from_utf8(bytes.to_vec())?);
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self::new(
            hash.ok_or_else(|| Bech32Error::FieldMissing("hash".to_string()))?,
            relays,
        ))
    }
}

impl ToBech32 for ChannelId {
    type Err = Bech32Error;
    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = vec![SPECIAL, 32];
        bytes.extend(self.hash().as_byte_array());

        for relay in self.relays.iter() {
            bytes.extend([RELAY, relay.len() as u8]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_bech32_channel_only_id() {
        let channel_id =
            ChannelId::from_hex("3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d")
                .unwrap();
        assert_eq!(
            "nchannel1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gg07hju".to_string(),
            channel_id.to_bech32().unwrap()
        );
    }

    #[test]
    fn from_bech32_channel_only_id() {
        let channel_id = ChannelId::from_bech32(
            "nchannel1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gg07hju",
        )
        .unwrap();
        assert_eq!(
            "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d".to_string(),
            channel_id.to_hex()
        );
    }

    #[test]
    fn to_bech32_channel_with_relays() {
        let event_id =
            EventId::from_hex("25e5c82273a271cb1a840d0060391a0bf4965cafeb029d5ab55350b418953fbb")
                .unwrap();
        let relays = vec!["wss://relay.damus.io".to_string()];
        let channel_id = ChannelId::new(event_id.inner(), relays);
        assert_eq!(
            "nchannel1qqsztewgyfe6yuwtr2zq6qrq8ydqhayktjh7kq5at264x595rz2nlwcpz3mhxue69uhhyetvv9ujuerpd46hxtnfducdjqp3".to_string(),
            channel_id.to_bech32().unwrap()
        );
    }

    #[test]
    fn from_bech32_channel_with_relays() {
        let channel_id = ChannelId::from_bech32(
            "nchannel1qqsztewgyfe6yuwtr2zq6qrq8ydqhayktjh7kq5at264x595rz2nlwcpz3mhxue69uhhyetvv9ujuerpd46hxtnfducdjqp3",
        ).unwrap();
        assert_eq!(
            "25e5c82273a271cb1a840d0060391a0bf4965cafeb029d5ab55350b418953fbb".to_string(),
            channel_id.to_hex()
        );
        assert_eq!(
            vec!["wss://relay.damus.io".to_string()],
            channel_id.relays()
        );
    }
}
