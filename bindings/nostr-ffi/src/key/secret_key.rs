// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip19::{FromBech32, ToBech32};
use uniffi::Object;

use crate::error::Result;

#[derive(Object)]
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
    pub fn parse(secret_key: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::SecretKey::parse(secret_key)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_hex(hex: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::SecretKey::from_hex(hex)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_bech32(sk: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::SecretKey::from_bech32(sk)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            inner: nostr::SecretKey::from_slice(&bytes)?,
        })
    }

    pub fn to_hex(&self) -> String {
        self.inner.to_secret_hex()
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }
}
