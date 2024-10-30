// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use std::sync::Arc;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::nwc::JsNwc;
use crate::protocol::event::JsEventId;
use crate::protocol::key::JsPublicKey;
use crate::protocol::nips::nip57::JsZapType;

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

/// Nostr Zapper
#[wasm_bindgen(js_name = NostrZapper)]
pub struct JsNostrZapper {
    inner: Arc<DynNostrZapper>,
}

impl Deref for JsNostrZapper {
    type Target = Arc<DynNostrZapper>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Arc<DynNostrZapper>> for JsNostrZapper {
    fn from(inner: Arc<DynNostrZapper>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NostrZapper)]
impl JsNostrZapper {
    /// Create new `WebLN` instance and compose `NostrZapper`
    pub async fn webln() -> Result<JsNostrZapper> {
        let zapper = WebLNZapper::new().await.map_err(into_err)?;
        Ok(Self {
            inner: zapper.into_nostr_zapper(),
        })
    }

    pub fn nwc(client: &JsNwc) -> Self {
        let zapper: NWC = client.deref().clone();
        Self {
            inner: zapper.into_nostr_zapper(),
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
    pub fn message(self, message: &str) -> Self {
        Self {
            inner: self.inner.message(message),
        }
    }
}
