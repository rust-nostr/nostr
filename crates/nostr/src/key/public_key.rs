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

/// Public Key
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl PublicKey {
    /// Try to parse [PublicKey] from `hex` or `bech32`
    pub fn parse<S>(public_key: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let public_key: &str = public_key.as_ref();
        match Self::from_hex(public_key) {
            Ok(public_key) => Ok(public_key),
            Err(_) => match Self::from_bech32(public_key) {
                Ok(public_key) => Ok(public_key),
                Err(_) => Err(Error::InvalidPublicKey),
            },
        }
    }

    /// Parse [PublicKey] from `bytes`
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            inner: XOnlyPublicKey::from_slice(slice)?,
        })
    }

    /// Parse [PublicKey] from `hex` string
    ///
    /// Use `PublicKey::from_str` to try to parse it from `hex` or `bech32`
    pub fn from_hex<S>(hex: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            inner: XOnlyPublicKey::from_str(hex.as_ref())?,
        })
    }

    /// Get public key as `hex` string
    pub fn to_hex(&self) -> String {
        self.inner.to_string()
    }

    /// Get public key as `bytes`
    pub fn to_bytes(&self) -> [u8; 32] {
        self.inner.serialize()
    }
}

impl FromStr for PublicKey {
    type Err = Error;

    /// Try to parse [PublicKey] from `hex` or `bech32`
    fn from_str(public_key: &str) -> Result<Self, Self::Err> {
        Self::parse(public_key)
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
        let hex: String = String::deserialize(deserializer)?;
        Self::parse(hex).map_err(serde::de::Error::custom)
    }
}
