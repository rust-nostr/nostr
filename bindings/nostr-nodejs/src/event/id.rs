// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use napi::bindgen_prelude::BigInt;
use napi::Result;
use nostr::prelude::*;

use crate::error::into_err;

#[napi(js_name = "EventId")]
pub struct JsEventId {
    inner: EventId,
}

impl From<EventId> for JsEventId {
    fn from(event_id: EventId) -> Self {
        Self { inner: event_id }
    }
}

impl From<&JsEventId> for EventId {
    fn from(event_id: &JsEventId) -> Self {
        event_id.inner
    }
}

#[napi]
impl JsEventId {
    #[napi(constructor)]
    pub fn new(
        pubkey: String,
        created_at: BigInt,
        kind: BigInt,
        tags: Vec<Vec<String>>,
        content: String,
    ) -> Result<Self> {
        let pubkey = XOnlyPublicKey::from_str(&pubkey).map_err(into_err)?;
        let created_at = Timestamp::from(created_at.get_u64().1);
        let kind = Kind::from(kind.get_u64().1);
        let mut new_tags: Vec<Tag> = Vec::with_capacity(tags.len());
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }
        Ok(Self {
            inner: EventId::new(&pubkey, created_at, &kind, &new_tags, &content),
        })
    }

    #[napi(factory)]
    pub fn from_slice(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            inner: EventId::from_slice(&bytes).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn from_hex(hex: String) -> Result<Self> {
        Ok(Self {
            inner: EventId::from_hex(hex).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn from_bech32(id: String) -> Result<Self> {
        Ok(Self {
            inner: EventId::from_bech32(id).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    #[napi]
    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    #[napi]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }
}
