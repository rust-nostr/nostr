// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Public key

use alloc::string::String;
use alloc::sync::Arc;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::str::FromStr;

use bitcoin::secp256k1::{self, XOnlyPublicKey};
#[cfg(feature = "std")]
use once_cell::sync::OnceCell; // TODO: when MSRV will be >= 1.70.0, use `std::cell::OnceLock` instead and remove `once_cell` dep.
#[cfg(not(feature = "std"))]
use once_cell::unsync::OnceCell; // TODO: when MSRV will be >= 1.70.0, use `core::cell::OnceCell` instead and remove `once_cell` dep.
use serde::{Deserialize, Deserializer, Serialize};

use super::Error;
use crate::nips::nip19::FromBech32;
use crate::nips::nip21::NostrURI;
use crate::util::hex;

/// Public key size
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Public Key
#[derive(Debug, Clone)]
pub struct PublicKey {
    /// Public key bytes
    buf: [u8; PUBLIC_KEY_SIZE],
    /// Verification key
    xonly: Arc<OnceCell<XOnlyPublicKey>>,
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

impl From<XOnlyPublicKey> for PublicKey {
    fn from(pk: XOnlyPublicKey) -> Self {
        Self {
            buf: pk.serialize(),
            xonly: Arc::new(OnceCell::from(pk)),
        }
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl PublicKey {
    /// Construct unchecked public key
    #[inline]
    pub fn unchecked(bytes: [u8; PUBLIC_KEY_SIZE]) -> Self {
        Self {
            buf: bytes,
            xonly: Arc::new(OnceCell::new()),
        }
    }

    /// Construct checked public key
    #[inline]
    pub fn checked(bytes: [u8; PUBLIC_KEY_SIZE]) -> Result<Self, Error> {
        Ok(Self {
            xonly: Arc::new(OnceCell::from(XOnlyPublicKey::from_slice(&bytes)?)),
            buf: bytes,
        })
    }

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

    /// Parse from `bytes`
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        if slice.len() != PUBLIC_KEY_SIZE {
            return Err(Error::InvalidPublicKey);
        }

        let mut bytes: [u8; PUBLIC_KEY_SIZE] = [0u8; PUBLIC_KEY_SIZE];
        bytes.copy_from_slice(slice);

        Ok(Self::unchecked(bytes))
    }

    /// Parse from `hex` string
    #[inline]
    pub fn from_hex<S>(hex: S) -> Result<Self, Error>
    where
        S: AsRef<[u8]>,
    {
        let mut bytes: [u8; PUBLIC_KEY_SIZE] = [0u8; PUBLIC_KEY_SIZE];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(Self::unchecked(bytes))
    }

    /// Get public key as `hex` string
    #[inline]
    pub fn to_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Get public key as `bytes`
    #[inline]
    pub fn as_bytes(&self) -> &[u8; PUBLIC_KEY_SIZE] {
        &self.buf
    }

    /// Get public key as `bytes`
    #[inline]
    pub fn to_bytes(self) -> [u8; PUBLIC_KEY_SIZE] {
        self.buf
    }

    /// Get or try to init [XOnlyPublicKey]
    pub(crate) fn get_xonly_public_key(&self) -> Result<&XOnlyPublicKey, secp256k1::Error> {
        self.xonly
            .get_or_try_init(|| XOnlyPublicKey::from_slice(self.as_bytes()))
    }

    /// Check if public key is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.get_xonly_public_key().is_ok()
    }
}

impl FromStr for PublicKey {
    type Err = Error;

    /// Parse from `hex`, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
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
