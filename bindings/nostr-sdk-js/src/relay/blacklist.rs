// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_js::event::JsEventId;
use nostr_js::key::JsPublicKey;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = RelayBlacklist)]
pub struct JsRelayBlacklist {
    inner: RelayBlacklist,
}

impl Deref for JsRelayBlacklist {
    type Target = RelayBlacklist;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<RelayBlacklist> for JsRelayBlacklist {
    fn from(inner: RelayBlacklist) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = RelayBlacklist)]
impl JsRelayBlacklist {
    #[wasm_bindgen(constructor)]
    pub fn new(ids: Vec<JsEventId>, public_keys: Vec<JsPublicKey>) -> Self {
        Self {
            inner: RelayBlacklist::new(
                ids.into_iter().map(|id| *id),
                public_keys.into_iter().map(|p| *p),
            ),
        }
    }

    /// construct new empty blacklist
    pub fn empty() -> Self {
        Self {
            inner: RelayBlacklist::empty(),
        }
    }

    /// Add event IDs to blacklist
    #[wasm_bindgen(js_name = addIds)]
    pub async fn add_ids(&self, ids: Vec<JsEventId>) {
        self.inner.add_ids(ids.into_iter().map(|id| *id)).await
    }

    /// Remove event IDs from blacklist
    #[wasm_bindgen(js_name = removeIds)]
    pub async fn remove_ids(&self, ids: Vec<JsEventId>) {
        self.inner.remove_ids(ids.iter().map(|id| id.deref())).await
    }

    /// Remove event ID from blacklist
    #[wasm_bindgen(js_name = removeId)]
    pub async fn remove_id(&self, id: &JsEventId) {
        self.inner.remove_id(id.deref()).await
    }

    /// Check if blacklist contains event ID
    #[wasm_bindgen(js_name = hasId)]
    pub async fn has_id(&self, id: &JsEventId) -> bool {
        self.inner.has_id(id.deref()).await
    }

    /// Add public keys to blacklist
    #[wasm_bindgen(js_name = addPublicKeys)]
    pub async fn add_public_keys(&self, public_keys: Vec<JsPublicKey>) {
        self.inner
            .add_public_keys(public_keys.into_iter().map(|p| *p))
            .await
    }

    /// Remove event IDs from blacklist
    #[wasm_bindgen(js_name = removePublicKeys)]
    pub async fn remove_public_keys(&self, ids: Vec<JsPublicKey>) {
        self.inner
            .remove_public_keys(ids.iter().map(|p| p.deref()))
            .await
    }

    /// Remove public key from blacklist
    #[wasm_bindgen(js_name = removePublicKey)]
    pub async fn remove_public_key(&self, public_key: &JsPublicKey) {
        self.inner.remove_public_key(public_key.deref()).await
    }

    /// Check if blacklist contains public key
    #[wasm_bindgen(js_name = hasPublicKey)]
    pub async fn has_public_key(&self, public_key: &JsPublicKey) -> bool {
        self.inner.has_public_key(public_key.deref()).await
    }

    /// Remove everything
    pub async fn clear(&self) {
        self.inner.clear().await
    }
}
