// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use napi::Result;
use nostr::prelude::*;

mod builder;
mod id;

pub use self::builder::JsEventBuilder;
pub use self::id::JsEventId;
use crate::error::into_err;
use crate::key::JsPublicKey;

#[napi(js_name = "Event")]
pub struct JsEvent {
    inner: Event,
}

impl From<Event> for JsEvent {
    fn from(event: Event) -> Self {
        Self { inner: event }
    }
}

impl Deref for JsEvent {
    type Target = Event;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<&JsEvent> for Event {
    fn from(event: &JsEvent) -> Self {
        event.inner.clone()
    }
}

#[napi]
impl JsEvent {
    #[napi(getter)]
    pub fn id(&self) -> JsEventId {
        self.inner.id.into()
    }

    #[napi(getter)]
    pub fn pubkey(&self) -> JsPublicKey {
        self.inner.pubkey.into()
    }

    #[napi(getter)]
    pub fn created_at(&self) -> u64 {
        self.inner.created_at.as_u64()
    }

    #[napi(getter)]
    pub fn kind(&self) -> u64 {
        self.inner.kind.into()
    }

    #[napi(getter)]
    pub fn tags(&self) -> Vec<Vec<String>> {
        self.inner.tags.iter().map(|t| t.as_vec()).collect()
    }

    #[napi(getter)]
    pub fn content(&self) -> String {
        self.inner.content.clone()
    }

    #[napi(getter)]
    pub fn signature(&self) -> String {
        self.inner.sig.to_string()
    }

    #[napi]
    pub fn verify(&self) -> bool {
        self.inner.verify().is_ok()
    }

    #[napi(factory)]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            inner: Event::from_json(json).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }
}
