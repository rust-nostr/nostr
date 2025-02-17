// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip49::{self, Version};
use nostr::{FromBech32, ToBech32};
use uniffi::{Enum, Object};

use crate::error::Result;
use crate::protocol::key::SecretKey;

/// Encrypted Secret Key version (NIP49)
#[derive(Enum)]
pub enum EncryptedSecretKeyVersion {
    V2,
}

impl From<Version> for EncryptedSecretKeyVersion {
    fn from(value: Version) -> Self {
        match value {
            Version::V2 => Self::V2,
        }
    }
}

/// Key security
#[derive(Enum)]
pub enum KeySecurity {
    /// The key has been known to have been handled insecurely (stored unencrypted, cut and paste unencrypted, etc)
    Weak,
    /// The key has NOT been known to have been handled insecurely (stored encrypted, cut and paste encrypted, etc)
    Medium,
    /// The client does not track this data
    Unknown,
}

impl From<nip49::KeySecurity> for KeySecurity {
    fn from(value: nip49::KeySecurity) -> Self {
        match value {
            nip49::KeySecurity::Weak => Self::Weak,
            nip49::KeySecurity::Medium => Self::Medium,
            nip49::KeySecurity::Unknown => Self::Unknown,
        }
    }
}

impl From<KeySecurity> for nip49::KeySecurity {
    fn from(value: KeySecurity) -> Self {
        match value {
            KeySecurity::Weak => Self::Weak,
            KeySecurity::Medium => Self::Medium,
            KeySecurity::Unknown => Self::Unknown,
        }
    }
}

/// Encrypted Secret Key
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct EncryptedSecretKey {
    inner: nip49::EncryptedSecretKey,
}

impl From<nip49::EncryptedSecretKey> for EncryptedSecretKey {
    fn from(inner: nip49::EncryptedSecretKey) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl EncryptedSecretKey {
    /// Encrypt secret key
    #[uniffi::constructor]
    pub fn new(
        secret_key: &SecretKey,
        password: &str,
        log_n: u8,
        key_security: KeySecurity,
    ) -> Result<Self> {
        Ok(Self {
            inner: nip49::EncryptedSecretKey::new(
                secret_key.deref(),
                password,
                log_n,
                key_security.into(),
            )?,
        })
    }

    #[uniffi::constructor]
    pub fn from_bech32(bech32: &str) -> Result<Self> {
        Ok(Self {
            inner: nip49::EncryptedSecretKey::from_bech32(bech32)?,
        })
    }

    /// Get encrypted secret key version
    pub fn version(&self) -> EncryptedSecretKeyVersion {
        self.inner.version().into()
    }

    /// Get encrypted secret key security
    pub fn key_security(&self) -> KeySecurity {
        self.inner.key_security().into()
    }

    /// Decrypt secret key
    pub fn decrypt(&self, password: &str) -> Result<SecretKey> {
        Ok(self.inner.decrypt(password)?.into())
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }
}
