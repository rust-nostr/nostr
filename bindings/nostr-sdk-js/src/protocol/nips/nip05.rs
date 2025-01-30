// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::key::JsPublicKey;

/// NIP05 profile
///
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
#[wasm_bindgen(js_name = Nip05Profile)]
pub struct JsNip05Profile {
    inner: Nip05Profile,
}

impl From<Nip05Profile> for JsNip05Profile {
    fn from(inner: Nip05Profile) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Nip05Profile)]
impl JsNip05Profile {
    /// Public key
    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    /// Relays
    #[wasm_bindgen]
    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.iter().map(|u| u.to_string()).collect()
    }

    /// NIP46 relays
    #[wasm_bindgen]
    pub fn nip46(&self) -> Vec<String> {
        self.inner.nip46.iter().map(|u| u.to_string()).collect()
    }
}

/// Verify NIP05
///
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
#[wasm_bindgen(js_name = verifyNip05)]
pub async fn verify_nip05(public_key: &JsPublicKey, nip05: &str) -> Result<bool> {
    nip05::verify(public_key.deref(), nip05, None)
        .await
        .map_err(into_err)
}

/// Get NIP05 profile
///
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
#[wasm_bindgen(js_name = getNip05Profile)]
pub async fn get_nip05_profile(nip05: &str) -> Result<JsNip05Profile> {
    Ok(nip05::profile(nip05, None).await.map_err(into_err)?.into())
}
