// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip19::ToBech32;
use uniffi::Object;

use crate::error::Result;
use crate::protocol::nips::nip49::EncryptedSecretKey;

#[derive(Debug, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
pub struct SecretKey {
    inner: nostr::SecretKey,
}

impl From<nostr::SecretKey> for SecretKey {
    fn from(inner: nostr::SecretKey) -> Self {
        Self { inner }
    }
}

impl Deref for SecretKey {
    type Target = nostr::SecretKey;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl SecretKey {
    /// Try to parse secret key from `hex` or `bech32`
    #[uniffi::constructor]
    pub fn parse(secret_key: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::SecretKey::parse(secret_key)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            inner: nostr::SecretKey::from_slice(bytes)?,
        })
    }

    /// Generate random secret key
    #[uniffi::constructor]
    pub fn generate() -> Self {
        Self {
            inner: nostr::SecretKey::generate(),
        }
    }

    pub fn to_hex(&self) -> String {
        self.inner.to_secret_hex()
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }

    /// Encrypt `SecretKey`
    ///
    /// By default, `LOG_N` is set to `16` and `KeySecurity` to `Unknown`.
    /// To use custom values check `EncryptedSecretKey` constructor.
    pub fn encrypt(&self, password: &str) -> Result<EncryptedSecretKey> {
        Ok(self.inner.encrypt(password)?.into())
    }
}
