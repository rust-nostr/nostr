// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};

#[derive(Clone, Copy)]
#[wasm_bindgen(js_name = PublicKey)]
pub struct JsPublicKey {
    pub(crate) inner: PublicKey,
}

impl Deref for JsPublicKey {
    type Target = PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<PublicKey> for JsPublicKey {
    fn from(inner: PublicKey) -> Self {
        Self { inner }
    }
}

impl From<JsPublicKey> for PublicKey {
    fn from(public_key: JsPublicKey) -> Self {
        public_key.inner
    }
}

impl From<&JsPublicKey> for PublicKey {
    fn from(public_key: &JsPublicKey) -> Self {
        public_key.inner
    }
}

#[wasm_bindgen(js_class = PublicKey)]
impl JsPublicKey {
    /// Try to parse public key from `hex`, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    pub fn parse(public_key: &str) -> Result<JsPublicKey> {
        Ok(Self {
            inner: PublicKey::parse(public_key).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex(hex: &str) -> Result<JsPublicKey> {
        Ok(Self {
            inner: PublicKey::from_hex(hex).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(bech32: &str) -> Result<JsPublicKey> {
        Ok(Self {
            inner: PublicKey::from_bech32(bech32).map_err(into_err)?,
        })
    }

    /// Get in hex format
    #[wasm_bindgen(js_name = toHex)]
    pub fn to_hex(&self) -> String {
        self.inner.to_string()
    }

    /// Get in bech32 format
    #[wasm_bindgen(js_name = toBech32)]
    pub fn to_bech32(&self) -> String {
        self.inner.to_bech32()
    }
}
