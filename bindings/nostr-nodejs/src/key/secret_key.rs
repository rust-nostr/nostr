// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use napi::Result;
use nostr::prelude::*;

use crate::error::into_err;

#[napi(js_name = "SecretKey")]
pub struct JsSecretKey {
    inner: SecretKey,
}

impl From<SecretKey> for JsSecretKey {
    fn from(secret_key: SecretKey) -> Self {
        Self { inner: secret_key }
    }
}

impl From<&JsSecretKey> for SecretKey {
    fn from(secret_key: &JsSecretKey) -> Self {
        secret_key.inner
    }
}

#[napi]
impl JsSecretKey {
    #[napi(factory)]
    pub fn from_hex(hex: String) -> Result<Self> {
        Ok(Self {
            inner: SecretKey::from_str(&hex).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn from_bech32(sk: String) -> Result<Self> {
        Ok(Self {
            inner: SecretKey::from_bech32(sk).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn to_hex(&self) -> String {
        self.inner.display_secret().to_string()
    }

    #[napi]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }
}
