// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use core::str::FromStr;

use nostr::bitcoin::secp256k1::schnorr::Signature;
use nostr::{JsonUtil, UnsignedEvent};
use wasm_bindgen::prelude::*;

use super::tag::JsTag;
use crate::error::{into_err, Result};
use crate::event::{JsEvent, JsEventId};
use crate::key::{JsKeys, JsPublicKey};
use crate::types::JsTimestamp;

#[wasm_bindgen(js_name = UnsignedEvent)]
pub struct JsUnsignedEvent {
    inner: UnsignedEvent,
}

impl Deref for JsUnsignedEvent {
    type Target = UnsignedEvent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<UnsignedEvent> for JsUnsignedEvent {
    fn from(inner: UnsignedEvent) -> Self {
        Self { inner }
    }
}

impl From<JsUnsignedEvent> for UnsignedEvent {
    fn from(event: JsUnsignedEvent) -> Self {
        event.inner
    }
}

#[wasm_bindgen(js_class = UnsignedEvent)]
impl JsUnsignedEvent {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Option<JsEventId> {
        self.inner.id.map(|id| id.into())
    }

    #[wasm_bindgen(getter)]
    pub fn pubkey(&self) -> JsPublicKey {
        self.inner.pubkey.into()
    }

    #[wasm_bindgen(js_name = createdAt, getter)]
    pub fn created_at(&self) -> JsTimestamp {
        self.inner.created_at.into()
    }

    #[inline]
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> u16 {
        self.inner.kind.as_u16()
    }

    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> Vec<JsTag> {
        self.inner.tags.iter().cloned().map(JsTag::from).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.inner.content.clone()
    }

    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<JsUnsignedEvent> {
        Ok(Self {
            inner: UnsignedEvent::from_json(json).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> Result<String> {
        self.inner.try_as_json().map_err(into_err)
    }

    /// Sign an unsigned event
    ///
    /// Internally: calculate event ID (if not set), sign it, compose and verify event.
    pub fn sign(self, keys: &JsKeys) -> Result<JsEvent> {
        Ok(self.inner.sign(keys.deref()).map_err(into_err)?.into())
    }

    /// Add signature to unsigned event
    ///
    /// Internally verify the event.
    #[wasm_bindgen(js_name = addSignature)]
    pub fn add_signature(self, sig: &str) -> Result<JsEvent> {
        let sig: Signature = Signature::from_str(sig).map_err(into_err)?;
        Ok(self.inner.add_signature(sig).map_err(into_err)?.into())
    }
}
