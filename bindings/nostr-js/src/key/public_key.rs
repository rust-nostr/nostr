// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};

#[wasm_bindgen(js_name = PublicKey)]
pub struct JsPublicKey {
    pub(crate) inner: XOnlyPublicKey,
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

#[wasm_bindgen(js_class = PublicKey)]
impl JsPublicKey {
    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex(hex: String) -> Result<JsPublicKey> {
        Ok(Self {
            inner: XOnlyPublicKey::from_str(&hex).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(pk: String) -> Result<JsPublicKey> {
        Ok(Self {
            inner: XOnlyPublicKey::from_bech32(pk).map_err(into_err)?,
        })
    }

    /// Get in hex format
    #[wasm_bindgen(js_name = toHex)]
    pub fn to_hex(&self) -> String {
        self.inner.to_string()
    }

    /// Get in bech32 format
    #[wasm_bindgen(js_name = toBech32)]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }
}
