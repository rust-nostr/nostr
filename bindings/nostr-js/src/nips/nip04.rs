// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip04;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::{JsPublicKey, JsSecretKey};

/// Encrypt (NIP04)
#[wasm_bindgen]
pub fn encrypt(sk: &JsSecretKey, pk: &JsPublicKey, text: String) -> Result<String> {
    nip04::encrypt(sk.deref(), pk.deref(), text).map_err(into_err)
}

/// Decrypt (NIP04)
#[wasm_bindgen]
pub fn decrypt(sk: &JsSecretKey, pk: &JsPublicKey, encrypted_content: String) -> Result<String> {
    nip04::decrypt(sk.deref(), pk.deref(), encrypted_content).map_err(into_err)
}
