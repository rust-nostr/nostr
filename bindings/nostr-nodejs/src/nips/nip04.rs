// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use napi::Result;
use nostr::nips::nip04;

use crate::error::into_err;
use crate::{JsPublicKey, JsSecretKey};

/// Encrypt (NIP04)
#[napi]
pub fn encrypt(sk: &JsSecretKey, pk: &JsPublicKey, text: String) -> Result<String> {
    nip04::encrypt(sk.deref(), pk.deref(), text).map_err(into_err)
}

/// Decrypt (NIP04)
#[napi]
pub fn decrypt(sk: &JsSecretKey, pk: &JsPublicKey, encrypted_content: String) -> Result<String> {
    nip04::decrypt(sk.deref(), pk.deref(), encrypted_content).map_err(into_err)
}
