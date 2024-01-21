// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip05;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::JsPublicKey;

/// Verify NIP05
#[wasm_bindgen(js_name = verifyNip05)]
pub async fn verify_nip05(public_key: JsPublicKey, nip05: &str) -> Result<()> {
    nip05::verify(public_key.into(), nip05, None)
        .await
        .map_err(into_err)
}
