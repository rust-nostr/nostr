// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_js::key::JsPublicKey;
use nostr_js::types::JsMetadata;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Profile)]
pub struct JsProfile {
    inner: Profile<Metadata>,
}

impl From<Profile<Metadata>> for JsProfile {
    fn from(inner: Profile<Metadata>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Profile)]
impl JsProfile {
    /// Compose new profile
    #[wasm_bindgen(constructor)]
    pub fn new(public_key: &JsPublicKey, metadata: &JsMetadata) -> Self {
        Self {
            inner: Profile::new(**public_key, metadata.deref().clone()),
        }
    }

    /// Get profile public key
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key().into()
    }

    /// Get profile metadata
    pub fn metadata(&self) -> JsMetadata {
        self.inner.metadata().clone().into()
    }

    /// Get profile name
    ///
    /// Steps (go to next step if field is `None` or `empty`):
    /// * Check `display_name` field
    /// * Check `name` field
    /// * Return cut public key (ex. `00000000:00000002`)
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }
}
