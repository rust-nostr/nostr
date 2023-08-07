// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip19::{FromBech32, ToBech32};
use nostr::secp256k1::SecretKey as Sk;
use uniffi::Object;

use crate::error::Result;

#[derive(Debug, Object)]
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

#[uniffi::export]
impl SecretKey {
    #[uniffi::constructor]
    pub fn from_hex(hex: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: Sk::from_str(&hex)?,
        }))
    }

    #[uniffi::constructor]
    pub fn from_bech32(pk: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: Sk::from_bech32(pk)?,
        }))
    }

    pub fn to_hex(&self) -> String {
        self.inner.display_secret().to_string()
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }
}
