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
    event: Event,
}

impl From<Event> for JsEvent {
    fn from(event: Event) -> Self {
        Self { event }
    }
}

impl Deref for JsEvent {
    type Target = Event;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

#[napi]
impl JsEvent {
    #[napi(getter)]
    pub fn id(&self) -> JsEventId {
        self.event.id.into()
    }

    #[napi(getter)]
    pub fn pubkey(&self) -> JsPublicKey {
        self.event.pubkey.into()
    }

    #[napi(getter)]
    pub fn created_at(&self) -> u64 {
        self.event.created_at.as_u64()
    }

    #[napi(getter)]
    pub fn kind(&self) -> u64 {
        self.event.kind.into()
    }

    #[napi(getter)]
    pub fn tags(&self) -> Vec<Vec<String>> {
        self.event.tags.iter().map(|t| t.as_vec()).collect()
    }

    #[napi(getter)]
    pub fn content(&self) -> String {
        self.event.content.clone()
    }

    #[napi(getter)]
    pub fn signature(&self) -> String {
        self.event.sig.to_string()
    }

    #[napi]
    pub fn verify(&self) -> bool {
        self.event.verify().is_ok()
    }

    #[napi(factory)]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            event: Event::from_json(json).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn as_json(&self) -> Result<String> {
        self.event.as_json().map_err(into_err)
    }
}
