// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip44::{self, Version};
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::{JsPublicKey, JsSecretKey};

#[wasm_bindgen(js_name = NIP44Version)]
pub enum JsNIP44Version {
    /// V1 - Deprecated
    Deprecated = 1,
    V2 = 2,
}

impl From<Version> for JsNIP44Version {
    fn from(version: Version) -> Self {
        match version {
            #[allow(deprecated)]
            Version::V1 => Self::Deprecated,
            Version::V2 => Self::V2,
        }
    }
}

impl From<JsNIP44Version> for Version {
    fn from(version: JsNIP44Version) -> Self {
        match version {
            #[allow(deprecated)]
            JsNIP44Version::Deprecated => Self::V1,
            JsNIP44Version::V2 => Self::V2,
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
