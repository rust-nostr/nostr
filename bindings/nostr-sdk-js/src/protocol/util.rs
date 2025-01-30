// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::key::{JsPublicKey, JsSecretKey};

/// Generate shared key
///
/// **Important: use of a strong cryptographic hash function may be critical to security! Do NOT use
/// unless you understand cryptographical implications.**
#[wasm_bindgen(js_name = generateSharedKey)]
pub fn generate_shared_key(secret_key: &JsSecretKey, public_key: &JsPublicKey) -> Result<Vec<u8>> {
    Ok(
        util::generate_shared_key(secret_key.deref(), public_key.deref())
            .map_err(into_err)?
            .to_vec(),
    )
}
