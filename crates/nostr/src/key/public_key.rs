// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Public key

use alloc::string::{String, ToString};
use core::fmt;
use core::ops::Deref;
use core::str::FromStr;

use bitcoin::secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Deserializer, Serialize};

use super::Error;
use crate::nips::nip19::FromBech32;
use crate::nips::nip21::NostrURI;

/// Public Key
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicKey {
    inner: XOnlyPublicKey,
}

impl Deref for PublicKey {
    type Target = XOnlyPublicKey;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<XOnlyPublicKey> for PublicKey {
    fn from(inner: XOnlyPublicKey) -> Self {
        Self { inner }
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", self.to_hex())
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl PublicKey {
    /// Public Key len
    pub const LEN: usize = 32;

    /// Parse from `hex`, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    pub fn parse<S>(public_key: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let public_key: &str = public_key.as_ref();

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

    /// Parse [PublicKey] from `bytes`
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            inner: XOnlyPublicKey::from_slice(slice)?,
        })
    }

    /// Parse [PublicKey] from `hex` string
    #[inline]
    pub fn from_hex<S>(hex: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            inner: XOnlyPublicKey::from_str(hex.as_ref())?,
        })
    }

    /// Get public key as `hex` string
    #[inline]
    pub fn to_hex(&self) -> String {
        self.inner.to_string()
    }

    /// Get public key as `bytes`
    #[inline]
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        self.inner.serialize()
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
        Self::parse(public_key).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_public_key_parse() {
        let public_key = PublicKey::parse(
            "nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy",
        )
        .unwrap();
        assert_eq!(
            public_key.to_hex(),
            "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4"
        );
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
