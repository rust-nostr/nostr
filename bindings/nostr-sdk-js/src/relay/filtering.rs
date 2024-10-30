// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::protocol::event::JsEventId;
use crate::protocol::key::JsPublicKey;

#[wasm_bindgen(js_name = RelayFilteringMode)]
pub enum JsRelayFilteringMode {
    /// Only the matching values will be allowed
    Whitelist,
    /// All matching values will be discarded
    Blacklist,
}

impl From<JsRelayFilteringMode> for RelayFilteringMode {
    fn from(value: JsRelayFilteringMode) -> Self {
        match value {
            JsRelayFilteringMode::Whitelist => Self::Whitelist,
            JsRelayFilteringMode::Blacklist => Self::Blacklist,
        }
    }
}

impl From<RelayFilteringMode> for JsRelayFilteringMode {
    fn from(value: RelayFilteringMode) -> Self {
        match value {
            RelayFilteringMode::Whitelist => Self::Whitelist,
            RelayFilteringMode::Blacklist => Self::Blacklist,
        }
    }
}

#[wasm_bindgen(js_name = RelayFiltering)]
pub struct JsRelayFiltering {
    inner: RelayFiltering,
}

impl Deref for JsRelayFiltering {
    type Target = RelayFiltering;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<RelayFiltering> for JsRelayFiltering {
    fn from(inner: RelayFiltering) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = RelayFiltering)]
impl JsRelayFiltering {
    /// Construct new filtering in whitelist mode
    pub fn whitelist() -> Self {
        Self {
            inner: RelayFiltering::whitelist(),
        }
    }

    /// Construct new filtering in blacklist mode
    pub fn blacklist() -> Self {
        Self {
            inner: RelayFiltering::blacklist(),
        }
    }

    /// Get filtering mode
    pub fn mode(&self) -> JsRelayFilteringMode {
        self.inner.mode().into()
    }

    /// Update filtering mode
    #[wasm_bindgen(js_name = updateMode)]
    pub fn update_mode(&self, mode: JsRelayFilteringMode) {
        self.inner.update_mode(mode.into());
    }

    /// Add event IDs
    ///
    /// Note: IDs are ignored in whitelist mode!
    #[wasm_bindgen(js_name = addIds)]
    pub async fn add_ids(&self, ids: Vec<JsEventId>) {
        self.inner.add_ids(ids.into_iter().map(|id| *id)).await
    }

    /// Remove event IDs
    ///
    /// Note: IDs are ignored in whitelist mode!
    #[wasm_bindgen(js_name = removeIds)]
    pub async fn remove_ids(&self, ids: Vec<JsEventId>) {
        self.inner.remove_ids(ids.iter().map(|id| id.deref())).await
    }

    /// Remove event ID
    ///
    /// Note: IDs are ignored in whitelist mode!
    #[wasm_bindgen(js_name = removeId)]
    pub async fn remove_id(&self, id: &JsEventId) {
        self.inner.remove_id(id.deref()).await
    }

    /// Check if has event ID
    #[wasm_bindgen(js_name = hasId)]
    pub async fn has_id(&self, id: &JsEventId) -> bool {
        self.inner.has_id(id.deref()).await
    }

    /// Add public keys
    #[wasm_bindgen(js_name = addPublicKeys)]
    pub async fn add_public_keys(&self, public_keys: Vec<JsPublicKey>) {
        self.inner
            .add_public_keys(public_keys.into_iter().map(|p| *p))
            .await
    }

    /// Remove public keys
    #[wasm_bindgen(js_name = removePublicKeys)]
    pub async fn remove_public_keys(&self, public_keys: Vec<JsPublicKey>) {
        self.inner
            .remove_public_keys(public_keys.iter().map(|p| p.deref()))
            .await
    }

    /// Remove public key
    #[wasm_bindgen(js_name = removePublicKey)]
    pub async fn remove_public_key(&self, public_key: &JsPublicKey) {
        self.inner.remove_public_key(public_key.deref()).await
    }

    /// Overwrite public keys set
    #[wasm_bindgen(js_name = overwritePublicKeys)]
    pub async fn overwrite_public_keys(&self, public_keys: Vec<JsPublicKey>) {
        self.inner
            .overwrite_public_keys(public_keys.iter().map(|p| **p))
            .await
    }

    /// Check if has public key
    #[wasm_bindgen(js_name = hasPublicKey)]
    pub async fn has_public_key(&self, public_key: &JsPublicKey) -> bool {
        self.inner.has_public_key(public_key.deref()).await
    }

    /// Remove everything
    pub async fn clear(&self) {
        self.inner.clear().await
    }
}
