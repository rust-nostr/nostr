// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_js::event::JsEventId;
use nostr_js::key::JsPublicKey;
use nostr_js::nips::nip47::JsNostrWalletConnectURI;
use nostr_js::nips::nip57::JsZapType;
use nostr_sdk::client::{ClientZapper, ZapDetails, ZapEntity};
use wasm_bindgen::prelude::*;
use webln_js::JsWebLN;

/// Zap entity
#[wasm_bindgen(js_name = ZapEntity)]
pub struct JsZapEntity {
    inner: ZapEntity,
}

impl Deref for JsZapEntity {
    type Target = ZapEntity;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = ZapEntity)]
impl JsZapEntity {
    pub fn event(event_id: &JsEventId) -> Self {
        Self {
            inner: ZapEntity::Event(**event_id),
        }
    }

    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(public_key: &JsPublicKey) -> Self {
        Self {
            inner: ZapEntity::PublicKey(**public_key),
        }
    }
}

/// Client Zapper
#[wasm_bindgen(js_name = ClientZapper)]
pub struct JsClientZapper {
    inner: ClientZapper,
}

impl Deref for JsClientZapper {
    type Target = ClientZapper;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = ClientZapper)]
impl JsClientZapper {
    pub fn webln(instance: &JsWebLN) -> Self {
        Self {
            inner: ClientZapper::WebLN(instance.deref().clone()),
        }
    }

    pub fn nwc(uri: &JsNostrWalletConnectURI) -> Self {
        Self {
            inner: ClientZapper::NWC(uri.deref().clone()),
        }
    }
}

/// Zap Details
#[wasm_bindgen(js_name = ZapDetails)]
pub struct JsZapDetails {
    inner: ZapDetails,
}

impl From<JsZapDetails> for ZapDetails {
    fn from(value: JsZapDetails) -> Self {
        value.inner
    }
}

#[wasm_bindgen(js_class = ZapDetails)]
impl JsZapDetails {
    /// Create new Zap Details
    ///
    /// **Note: `private` zaps are not currently supported here!**
    #[wasm_bindgen(constructor)]
    pub fn new(zap_type: JsZapType) -> Self {
        Self {
            inner: ZapDetails::new(zap_type.into()),
        }
    }

    /// Add message
    pub fn message(self, message: String) -> Self {
        Self {
            inner: self.inner.message(message),
        }
    }
}
