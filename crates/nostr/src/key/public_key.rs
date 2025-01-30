// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Public key

use alloc::string::String;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::str::FromStr;

use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Deserializer, Serialize};

use super::Error;
use crate::nips::nip19::FromBech32;
use crate::nips::nip21::NostrURI;
use crate::util::hex;

/// Public Key
#[derive(Clone, Copy)]
pub struct PublicKey {
    buf: [u8; 32],
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", self.to_hex())
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.buf == other.buf
    }
}

impl Eq for PublicKey {}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.buf.cmp(&other.buf)
    }
}

impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buf.hash(state);
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<XOnlyPublicKey> for PublicKey {
    fn from(inner: XOnlyPublicKey) -> Self {
        Self {
            buf: inner.serialize(),
        }
    }
}

impl PublicKey {
    /// Public Key len
    pub const LEN: usize = 32;

    /// Construct from 32-byte array
    #[inline]
    pub const fn from_byte_array(bytes: [u8; Self::LEN]) -> Self {
        Self { buf: bytes }
    }

    /// Parse from `hex`, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    pub fn parse(public_key: &str) -> Result<Self, Error> {
        // Try from hex
        if let Ok(public_key) = Self::from_hex(public_key) {
            return Ok(public_key);
        }

        // Try from bech32
        if let Ok(public_key) = Self::from_bech32(public_key) {
            return Ok(public_key);
        }

        // Try from NIP21 URI
        if let Ok(public_key) = Self::from_nostr_uri(public_key) {
            return Ok(public_key);
        }

        Err(Error::InvalidPublicKey)
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, Error> {
        let mut bytes: [u8; Self::LEN] = [0u8; Self::LEN];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(Self::from_byte_array(bytes))
    }

    /// Parse from bytes
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        // Check len
        if slice.len() != Self::LEN {
            return Err(Error::InvalidPublicKey);
        }

        // Copy bytes
        let mut bytes: [u8; Self::LEN] = [0u8; Self::LEN];
        bytes.copy_from_slice(slice);

        // Construct
        Ok(Self::from_byte_array(bytes))
    }

    /// Get public key as `hex` string
    #[inline]
    pub fn to_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Get as bytes
    #[inline]
    pub fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.buf
    }

    /// Get public key as `bytes`
    #[inline]
    pub fn to_bytes(self) -> [u8; Self::LEN] {
        self.buf
    }

    /// Get the x-only public key
    pub fn xonly(&self) -> Result<XOnlyPublicKey, Error> {
        // TODO: use a OnceCell
        Ok(XOnlyPublicKey::from_slice(self.as_bytes())?)
    }
}

impl FromStr for PublicKey {
    type Err = Error;

    /// Try to parse [PublicKey] from `hex`, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    #[inline]
    fn from_str(public_key: &str) -> Result<Self, Self::Err> {
        Self::parse(public_key)
    }
}

// Required to keep clean the methods of `Filter` struct
impl From<PublicKey> for String {
    fn from(public_key: PublicKey) -> Self {
        public_key.to_hex()
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let public_key: String = String::deserialize(deserializer)?;
        Self::parse(&public_key).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_parse() {
        let public_key = PublicKey::parse(
            "nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy",
        )
        .unwrap();
        assert_eq!(
            public_key.to_hex(),
            "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4"
        );
    }

    #[test]
    fn test_as_xonly() {
        let hex_pk: &str = "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4";

        let public_key = PublicKey::from_hex(hex_pk).unwrap();

        //assert!(public_key.xonly.is_null());

        let expected = XOnlyPublicKey::from_str(hex_pk).unwrap();
        let xonly = public_key.xonly().unwrap();
        assert_eq!(&xonly, &expected);

        let public_key = PublicKey::from(expected);
        let xonly = public_key.xonly().unwrap();
        assert_eq!(&xonly, &expected);
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;
    use crate::nips::nip19::ToBech32;

    const NIP21_URI: &str = "nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy";
    const HEX: &str = "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4";
    const BECH32: &str = "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy";

    #[bench]
    pub fn parse_public_key_nip21_uri(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(PublicKey::parse(NIP21_URI)).unwrap();
        });
    }

    #[bench]
    pub fn parse_public_key_hex(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(PublicKey::parse(HEX)).unwrap();
        });
    }

    #[bench]
    pub fn public_key_from_hex(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(PublicKey::from_hex(HEX)).unwrap();
        });
    }

    #[bench]
    pub fn parse_public_key_bech32(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(PublicKey::parse(BECH32)).unwrap();
        });
    }

    #[bench]
    pub fn public_key_from_bech32(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(PublicKey::from_bech32(BECH32)).unwrap();
        });
    }

    #[bench]
    pub fn public_key_to_hex(bh: &mut Bencher) {
        let public_key = PublicKey::from_hex(HEX).unwrap();
        bh.iter(|| {
            black_box(public_key.to_bech32()).unwrap();
        });
    }

    #[bench]
    pub fn public_key_to_bech32(bh: &mut Bencher) {
        let public_key = PublicKey::from_hex(HEX).unwrap();
        bh.iter(|| {
            black_box(public_key.to_bech32()).unwrap();
        });
    }
}
