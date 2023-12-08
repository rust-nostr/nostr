// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::JsEventId;

/// Channel Id
///
/// Kind 40 event id (32-bytes lowercase hex-encoded)
///
/// https://github.com/nostr-protocol/nips/blob/master/19.md
#[wasm_bindgen(js_name = ChannelId)]
pub struct JsChannelId {
    inner: ChannelId,
}

impl From<ChannelId> for JsChannelId {
    fn from(channel_id: ChannelId) -> Self {
        Self { inner: channel_id }
    }
}

impl From<&JsChannelId> for ChannelId {
    fn from(channel_id: &JsChannelId) -> Self {
        channel_id.inner.clone()
    }
}

#[wasm_bindgen(js_class = ChannelId)]
impl JsChannelId {
    /// New [`ChannelId`]
    #[wasm_bindgen(constructor)]
    pub fn new(event_id: &JsEventId) -> Self {
        let event_id: EventId = event_id.into();
        Self {
            inner: ChannelId::new(event_id.inner(), Vec::new()),
        }
    }

    /// [`ChannelId`] from bytes
    #[wasm_bindgen(js_name = fromSlice)]
    pub fn from_slice(sl: Vec<u8>) -> Result<JsChannelId> {
        Ok(Self {
            inner: ChannelId::from_slice(&sl).map_err(into_err)?,
        })
    }

    /// [`ChannelId`] hex string
    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex(hex: String) -> Result<JsChannelId> {
        Ok(Self {
            inner: ChannelId::from_hex(hex).map_err(into_err)?,
        })
    }

    /// [`ChannelId`] bech32 string
    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(id: String) -> Result<JsChannelId> {
        Ok(Self {
            inner: ChannelId::from_bech32(id).map_err(into_err)?,
        })
    }

    /// Get as bytes
    #[wasm_bindgen(js_name = asBytes)]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    /// Get as hex string
    #[wasm_bindgen(js_name = toHex)]
    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    /// Get as bech32 string
    #[wasm_bindgen(js_name = toBech32)]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }
}
