// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Secret key

use alloc::string::{String, ToString};
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::str::FromStr;

use bitcoin::secp256k1;
use serde::{Deserialize, Deserializer};

use super::Error;
use crate::nips::nip19::FromBech32;
#[cfg(all(feature = "std", feature = "nip49"))]
use crate::nips::nip49::{self, EncryptedSecretKey, KeySecurity};

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

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_secret_hex())
    }
}

impl SecretKey {
    /// Try to parse [SecretKey] from `hex` or `bech32`
    pub fn parse<S>(secret_key: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let secret_key: &str = secret_key.as_ref();
        match Self::from_hex(secret_key) {
            Ok(secret_key) => Ok(secret_key),
            Err(_) => match Self::from_bech32(secret_key) {
                Ok(secret_key) => Ok(secret_key),
                Err(_) => Err(Error::InvalidSecretKey),
            },
        }
    }

    /// Parse [SecretKey] from `bytes`
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            inner: secp256k1::SecretKey::from_slice(slice)?,
        })
    }

    /// Parse [SecretKey] from `hex` string
    pub fn from_hex<S>(hex: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            inner: secp256k1::SecretKey::from_str(hex.as_ref())?,
        })
    }

    /// Get secret key as `hex` string
    pub fn to_secret_hex(&self) -> String {
        self.inner.display_secret().to_string()
    }

    /// Get secret key as `bytes`
    pub fn to_secret_bytes(&self) -> [u8; 32] {
        self.inner.secret_bytes()
    }

    /// Encrypt [SecretKey]
    ///
    /// By default `LOG_N` is set to `16` and [KeySecurity] to `Unknown`.
    /// To use custom values check [EncryptedSecretKey] constructors.
    #[cfg(all(feature = "std", feature = "nip49"))]
    pub fn encrypt<S>(&self, password: S) -> Result<EncryptedSecretKey, nip49::Error>
    where
        S: AsRef<str>,
    {
        EncryptedSecretKey::new(self, password, 16, KeySecurity::Unknown)
    }
}

impl FromStr for SecretKey {
    type Err = Error;

    /// Try to parse [SecretKey] from `hex` or `bech32`
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
        Self::parse(secret_key).map_err(serde::de::Error::custom)
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.inner.non_secure_erase();
        tracing::trace!("Secret Key dropped.");
    }
}
