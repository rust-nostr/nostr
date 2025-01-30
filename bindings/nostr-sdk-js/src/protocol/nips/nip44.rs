// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::nips::nip44::{self, Version};
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::key::{JsPublicKey, JsSecretKey};

#[wasm_bindgen(js_name = NIP44Version)]
pub enum JsNIP44Version {
    V2 = 2,
}

impl From<Version> for JsNIP44Version {
    fn from(version: Version) -> Self {
        match version {
            Version::V2 => Self::V2,
        }
    }
}

impl From<JsNIP44Version> for Version {
    fn from(version: JsNIP44Version) -> Self {
        match version {
            JsNIP44Version::V2 => Self::V2,
        }
    }
}

/// Encrypt (NIP44)
#[wasm_bindgen(js_name = nip44Encrypt)]
pub fn nip44_encrypt(
    secret_key: &JsSecretKey,
    public_key: &JsPublicKey,
    content: &str,
    version: JsNIP44Version,
) -> Result<String> {
    nip44::encrypt(
        secret_key.deref(),
        public_key.deref(),
        content,
        version.into(),
    )
    .map_err(into_err)
}

/// Decrypt (NIP44)
#[wasm_bindgen(js_name = nip44Decrypt)]
pub fn nip44_decrypt(
    secret_key: &JsSecretKey,
    public_key: &JsPublicKey,
    payload: &str,
) -> Result<String> {
    nip44::decrypt(secret_key.deref(), public_key.deref(), payload).map_err(into_err)
}
