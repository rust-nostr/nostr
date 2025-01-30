// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::key::{JsPublicKey, JsSecretKey};

/// Encrypt (NIP04)
#[wasm_bindgen(js_name = nip04Encrypt)]
pub fn nip04_encrypt(
    secret_key: &JsSecretKey,
    public_key: &JsPublicKey,
    text: &str,
) -> Result<String> {
    nip04::encrypt(secret_key.deref(), public_key.deref(), text).map_err(into_err)
}

/// Decrypt (NIP04)
#[wasm_bindgen(js_name = nip04Decrypt)]
pub fn nip04_decrypt(
    secret_key: &JsSecretKey,
    public_key: &JsPublicKey,
    encrypted_content: &str,
) -> Result<String> {
    nip04::decrypt(secret_key.deref(), public_key.deref(), encrypted_content).map_err(into_err)
}
