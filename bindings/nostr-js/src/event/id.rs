// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::JsPublicKey;

#[wasm_bindgen(js_name = EventId)]
pub struct JsEventId {
    pub(crate) inner: EventId,
}

impl From<EventId> for JsEventId {
    fn from(event_id: EventId) -> Self {
        Self { inner: event_id }
    }
}

impl From<JsEventId> for EventId {
    fn from(event_id: JsEventId) -> Self {
        event_id.inner
    }
}

impl From<&JsEventId> for EventId {
    fn from(event_id: &JsEventId) -> Self {
        event_id.inner
    }
}

#[wasm_bindgen(js_class = EventId)]
impl JsEventId {
    #[wasm_bindgen(constructor)]
    pub fn new(
        pubkey: &JsPublicKey,
        created_at: u64,
        kind: u64,
        tags: JsValue,
        content: String,
    ) -> Result<JsEventId> {
        let created_at = Timestamp::from(created_at);
        let kind = Kind::from(kind);
        let tags: Vec<Vec<String>> = serde_wasm_bindgen::from_value(tags)?;
        let mut new_tags: Vec<Tag> = Vec::with_capacity(tags.len());
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }
        Ok(Self {
            inner: EventId::new(&pubkey.into(), created_at, &kind, &new_tags, &content),
        })
    }

    #[wasm_bindgen(js_name = fromSlice)]
    pub fn from_slice(bytes: Vec<u8>) -> Result<JsEventId> {
        Ok(Self {
            inner: EventId::from_slice(&bytes).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex(hex: String) -> Result<JsEventId> {
        Ok(Self {
            inner: EventId::from_hex(hex).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(id: String) -> Result<JsEventId> {
        Ok(Self {
            inner: EventId::from_bech32(id).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asBytes)]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    #[wasm_bindgen(js_name = toHex)]
    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    #[wasm_bindgen(js_name = toBech32)]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }
}
