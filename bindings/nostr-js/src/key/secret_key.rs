// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::nips::nip49::JsEncryptedSecretKey;

#[wasm_bindgen(js_name = SecretKey)]
pub struct JsSecretKey {
    inner: SecretKey,
}

impl Deref for JsSecretKey {
    type Target = SecretKey;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<SecretKey> for JsSecretKey {
    fn from(inner: SecretKey) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = SecretKey)]
impl JsSecretKey {
    /// Try to parse secret key from `hex` or `bech32`
    pub fn parse(secret_key: &str) -> Result<JsSecretKey> {
        Ok(Self {
            inner: SecretKey::parse(secret_key).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex(hex: &str) -> Result<JsSecretKey> {
        Ok(Self {
            inner: SecretKey::from_hex(hex).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(bech32: &str) -> Result<JsSecretKey> {
        Ok(Self {
            inner: SecretKey::from_bech32(bech32).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = toHex)]
    pub fn to_hex(&self) -> String {
        self.inner.to_secret_hex()
    }

    #[wasm_bindgen(js_name = toBech32)]
    pub fn to_bech32(&self) -> String {
        self.inner.to_bech32()
    }

    /// Encrypt secret key
    ///
    /// By default `LOG_N` is set to `16` and `KeySecurity` to `Unknown`.
    /// To use custom values check `EncryptedSecretKey` constructor.
    pub fn encrypt(&self, password: &str) -> Result<JsEncryptedSecretKey> {
        Ok(self.inner.encrypt(password).map_err(into_err)?.into())
    }
}
