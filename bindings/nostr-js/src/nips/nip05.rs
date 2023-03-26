// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::nips::nip05;
use wasm_bindgen::prelude::*;

use crate::key::JsPublicKey;

/// Verify NIP05
#[wasm_bindgen(js_name = verifyNip05)]
pub async fn verify_nip05(public_key: &JsPublicKey, nip05: String) -> bool {
    nip05::verify(public_key.into(), nip05.as_str())
        .await
        .is_ok()
}
