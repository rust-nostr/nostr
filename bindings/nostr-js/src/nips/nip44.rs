// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip44::{self, Version};
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::{JsPublicKey, JsSecretKey};

#[wasm_bindgen(js_name = NIP44Version)]
pub enum JsNIP44Version {
    /// XChaCha20
    XChaCha20 = 1,
}

impl From<Version> for JsNIP44Version {
    fn from(version: Version) -> Self {
        match version {
            Version::XChaCha20 => Self::XChaCha20,
        }
    }
}

impl From<JsNIP44Version> for Version {
    fn from(version: JsNIP44Version) -> Self {
        match version {
            JsNIP44Version::XChaCha20 => Self::XChaCha20,
        }
    }
}

/// Encrypt (NIP44)
#[wasm_bindgen]
pub fn nip44_encrypt(
    sk: &JsSecretKey,
    pk: &JsPublicKey,
    content: String,
    version: JsNIP44Version,
) -> Result<String> {
    nip44::encrypt(sk.deref(), pk.deref(), content, version.into()).map_err(into_err)
}

/// Decrypt (NIP44)
#[wasm_bindgen]
pub fn nip44_decrypt(sk: &JsSecretKey, pk: &JsPublicKey, payload: String) -> Result<String> {
    nip44::decrypt(sk.deref(), pk.deref(), payload).map_err(into_err)
}
