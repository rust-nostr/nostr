// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use napi::Result;
use nostr::prelude::*;

use crate::error::into_err;

#[napi(js_name = "PublicKey")]
pub struct JsPublicKey {
    inner: XOnlyPublicKey,
}

impl Deref for JsPublicKey {
    type Target = XOnlyPublicKey;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<XOnlyPublicKey> for JsPublicKey {
    fn from(public_key: XOnlyPublicKey) -> Self {
        Self { inner: public_key }
    }
}

impl From<&JsPublicKey> for XOnlyPublicKey {
    fn from(public_key: &JsPublicKey) -> Self {
        public_key.inner
    }
}

#[napi]
impl JsPublicKey {
    #[napi(factory)]
    pub fn from_hex(hex: String) -> Result<Self> {
        Ok(Self {
            inner: XOnlyPublicKey::from_str(&hex).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn from_bech32(pk: String) -> Result<Self> {
        Ok(Self {
            inner: XOnlyPublicKey::from_bech32(pk).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn to_hex(&self) -> String {
        self.inner.to_string()
    }

    #[napi]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }
}
