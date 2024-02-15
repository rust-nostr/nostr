// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip49::{EncryptedSecretKey, KeySecurity, Version};
use nostr::{FromBech32, ToBech32};
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::JsSecretKey;

/// Encrypted Secret Key version (NIP49)
#[wasm_bindgen(js_name = EncryptedSecretKeyVersion)]
pub enum JsEncryptedSecretKeyVersion {
    V2,
}

impl From<Version> for JsEncryptedSecretKeyVersion {
    fn from(value: Version) -> Self {
        match value {
            Version::V2 => Self::V2,
        }
    }
}

/// Key security
#[wasm_bindgen(js_name = KeySecurity)]
pub enum JsKeySecurity {
    /// The key has been known to have been handled insecurely (stored unencrypted, cut and paste unencrypted, etc)
    Weak,
    /// The key has NOT been known to have been handled insecurely (stored encrypted, cut and paste encrypted, etc)
    Medium,
    /// The client does not track this data
    Unknown,
}

impl From<KeySecurity> for JsKeySecurity {
    fn from(value: KeySecurity) -> Self {
        match value {
            KeySecurity::Weak => Self::Weak,
            KeySecurity::Medium => Self::Medium,
            KeySecurity::Unknown => Self::Unknown,
        }
    }
}

impl From<JsKeySecurity> for KeySecurity {
    fn from(value: JsKeySecurity) -> Self {
        match value {
            JsKeySecurity::Weak => Self::Weak,
            JsKeySecurity::Medium => Self::Medium,
            JsKeySecurity::Unknown => Self::Unknown,
        }
    }
}

/// Encrypted Secret Key
#[wasm_bindgen(js_name = EncryptedSecretKey)]
pub struct JsEncryptedSecretKey {
    inner: EncryptedSecretKey,
}

impl From<EncryptedSecretKey> for JsEncryptedSecretKey {
    fn from(inner: EncryptedSecretKey) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = EncryptedSecretKey)]
impl JsEncryptedSecretKey {
    /// Encrypt secret key
    #[wasm_bindgen(constructor)]
    pub fn new(
        secret_key: &JsSecretKey,
        password: &str,
        log_n: u8,
        key_security: JsKeySecurity,
    ) -> Result<JsEncryptedSecretKey> {
        Ok(Self {
            inner: EncryptedSecretKey::new(
                secret_key.deref(),
                password,
                log_n,
                key_security.into(),
            )
            .map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(encrypted_secret_key: String) -> Result<JsEncryptedSecretKey> {
        Ok(Self {
            inner: EncryptedSecretKey::from_bech32(encrypted_secret_key).map_err(into_err)?,
        })
    }

    /// Get encrypted secret key version
    pub fn version(&self) -> JsEncryptedSecretKeyVersion {
        self.inner.version().into()
    }

    /// Get encrypted secret key security
    #[wasm_bindgen(js_name = keySecurity)]
    pub fn key_security(&self) -> JsKeySecurity {
        self.inner.key_security().into()
    }

    /// Consume `EncryptedSecretKey` and return `SecretKey`
    #[wasm_bindgen(js_name = toSecretKey)]
    pub fn to_secret_key(self, password: &str) -> Result<JsSecretKey> {
        Ok(self.inner.to_secret_key(password).map_err(into_err)?.into())
    }

    /// Decrypt to `SecretKey`
    #[wasm_bindgen(js_name = asSecretKey)]
    pub fn as_secret_key(&self, password: &str) -> Result<JsSecretKey> {
        Ok(self.inner.to_secret_key(password).map_err(into_err)?.into())
    }

    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }
}
