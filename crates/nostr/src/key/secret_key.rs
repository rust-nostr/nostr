// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Secret key

use alloc::string::String;
use core::ops::{Deref, DerefMut};
use core::str::FromStr;

use serde::{Deserialize, Deserializer};

use super::Error;
use crate::nips::nip19::FromBech32;
#[cfg(feature = "nip49")]
use crate::nips::nip49::{self, EncryptedSecretKey, KeySecurity};
use crate::provider::NostrProvider;

/// Secret key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey {
    inner: secp256k1::SecretKey,
}

impl Deref for SecretKey {
    type Target = secp256k1::SecretKey;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SecretKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<secp256k1::SecretKey> for SecretKey {
    fn from(inner: secp256k1::SecretKey) -> Self {
        Self { inner }
    }
}

impl SecretKey {
    /// Secret Key len
    pub const LEN: usize = 32;

    /// Parse from `hex` or `bech32`
    pub fn parse(secret_key: &str) -> Result<Self, Error> {
        // Try from hex
        if let Ok(secret_key) = Self::from_hex(secret_key) {
            return Ok(secret_key);
        }

        // Try from bech32
        if let Ok(secret_key) = Self::from_bech32(secret_key) {
            return Ok(secret_key);
        }

        Err(Error::InvalidSecretKey)
    }

    /// Parse from `bytes`
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            inner: secp256k1::SecretKey::from_slice(slice)?,
        })
    }

    /// Parse from `hex`
    #[inline]
    pub fn from_hex(hex: &str) -> Result<Self, Error> {
        Ok(Self {
            inner: secp256k1::SecretKey::from_str(hex)?,
        })
    }

    /// Generate random secret key
    #[inline]
    pub fn generate() -> Self {
        let provider = NostrProvider::get();

        let mut data: [u8; 32] = random_32_bytes(&provider);

        loop {
            match Self::from_slice(&data) {
                Ok(secret_key) => return secret_key,
                Err(_) => {
                    data = random_32_bytes(&provider);
                }
            }
        }
    }

    /// Get secret key as `hex` string
    #[inline]
    pub fn to_secret_hex(&self) -> String {
        hex::encode(self.as_secret_bytes())
    }

    /// Get secret key as `bytes`
    #[inline]
    pub fn as_secret_bytes(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// Get secret key as `bytes`
    #[inline]
    pub fn to_secret_bytes(&self) -> [u8; Self::LEN] {
        self.inner.secret_bytes()
    }

    /// Encrypt secret key
    ///
    /// By default, `LOG_N` is set to `16` and [`KeySecurity::Unknown`].
    /// To use custom values check [`EncryptedSecretKey`] constructors.
    #[inline]
    #[cfg(feature = "nip49")]
    pub fn encrypt(&self, password: &str) -> Result<EncryptedSecretKey, nip49::Error> {
        EncryptedSecretKey::new(self, password, 16, KeySecurity::Unknown)
    }
}

impl FromStr for SecretKey {
    type Err = Error;

    /// Try to parse from `hex` or `bech32`
    #[inline]
    fn from_str(secret_key: &str) -> Result<Self, Self::Err> {
        Self::parse(secret_key)
    }
}

impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secret_key: String = String::deserialize(deserializer)?;
        Self::parse(&secret_key).map_err(serde::de::Error::custom)
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.inner.non_secure_erase();
    }
}

fn random_32_bytes(provider: &NostrProvider) -> [u8; 32] {
    let mut ret: [u8; 32] = [0u8; 32];
    provider.rng.fill(&mut ret);
    ret
}
