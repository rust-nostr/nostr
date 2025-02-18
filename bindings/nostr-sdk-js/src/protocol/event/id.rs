// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::{JsKind, JsTags};
use crate::error::{into_err, Result};
use crate::protocol::key::JsPublicKey;
use crate::protocol::types::JsTimestamp;

#[wasm_bindgen(js_name = EventId)]
#[derive(Clone, Copy)]
pub struct JsEventId {
    inner: EventId,
}

impl Deref for JsEventId {
    type Target = EventId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<EventId> for JsEventId {
    fn from(inner: EventId) -> Self {
        Self { inner }
    }
}

impl From<JsEventId> for EventId {
    fn from(event_id: JsEventId) -> Self {
        event_id.inner
    }
}

#[wasm_bindgen(js_class = EventId)]
impl JsEventId {
    #[wasm_bindgen(constructor)]
    pub fn new(
        pubkey: &JsPublicKey,
        created_at: &JsTimestamp,
        kind: &JsKind,
        tags: &JsTags,
        content: &str,
    ) -> Self {
        Self {
            inner: EventId::new(
                pubkey.deref(),
                created_at.deref(),
                kind.deref(),
                tags.deref(),
                content,
            ),
        }
    }

    /// Try to parse event ID from `hex`, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    pub fn parse(id: &str) -> Result<JsEventId> {
        Ok(Self {
            inner: EventId::parse(id).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromSlice)]
    pub fn from_slice(bytes: &[u8]) -> Result<JsEventId> {
        Ok(Self {
            inner: EventId::from_slice(bytes).map_err(into_err)?,
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

    #[wasm_bindgen(js_name = toNostrUri)]
    pub fn to_nostr_uri(&self) -> Result<String> {
        self.inner.to_nostr_uri().map_err(into_err)
    }
}
