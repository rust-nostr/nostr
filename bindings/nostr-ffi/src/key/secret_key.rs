// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::nips::nip19::{FromBech32, ToBech32};
use nostr::secp256k1::SecretKey as Sk;

use crate::error::Result;

pub struct SecretKey {
    inner: Sk,
}

impl From<Sk> for SecretKey {
    fn from(inner: Sk) -> Self {
        Self { inner }
    }
}

impl Deref for SecretKey {
    type Target = Sk;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SecretKey {
    pub fn from_hex(hex: String) -> Result<Self> {
        Ok(Self {
            inner: Sk::from_str(&hex)?,
        })
    }

    pub fn from_bech32(pk: String) -> Result<Self> {
        Ok(Self {
            inner: Sk::from_bech32(pk)?,
        })
    }

    pub fn to_hex(&self) -> String {
        self.inner.display_secret().to_string()
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }
}
