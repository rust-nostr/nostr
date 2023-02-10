// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use napi::Result;
use nostr::prelude::*;

use crate::error::into_err;
use crate::JsEventId;

/// Channel Id
///
/// Kind 40 event id (32-bytes lowercase hex-encoded)
///
/// https://github.com/nostr-protocol/nips/blob/master/19.md
#[napi(js_name = "ChannelId")]
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

#[napi]
impl JsChannelId {
    /// New [`ChannelId`]
    #[napi(constructor)]
    pub fn new(event_id: &JsEventId, relays: Vec<String>) -> Self {
        let event_id: EventId = event_id.into();
        Self {
            inner: ChannelId::new(event_id.inner(), relays),
        }
    }

    /// [`ChannelId`] from bytes
    #[napi(factory)]
    pub fn from_slice(sl: Vec<u8>) -> Result<Self> {
        Ok(Self {
            inner: ChannelId::from_slice(&sl).map_err(into_err)?,
        })
    }

    /// [`ChannelId`] hex string
    #[napi(factory)]
    pub fn from_hex(hex: String) -> Result<Self> {
        Ok(Self {
            inner: ChannelId::from_hex(hex).map_err(into_err)?,
        })
    }

    /// [`ChannelId`] bech32 string
    #[napi(factory)]
    pub fn from_bech32(id: String) -> Result<Self> {
        Ok(Self {
            inner: ChannelId::from_bech32(id).map_err(into_err)?,
        })
    }

    /// Get as bytes
    #[napi(getter)]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    /// Get as hex string
    #[napi(getter)]
    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    /// Get as bech32 string
    #[napi(getter)]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }

    /// Get relays
    #[napi(getter)]
    pub fn relays(&self) -> Vec<String> {
        self.inner.relays()
    }
}
