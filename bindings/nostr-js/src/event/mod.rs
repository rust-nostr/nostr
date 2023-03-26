// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

mod builder;
mod id;
mod tag;

pub use self::builder::JsEventBuilder;
pub use self::id::JsEventId;
pub use self::tag::JsTags;
use crate::error::{into_err, Result};
use crate::key::JsPublicKey;

#[wasm_bindgen(js_name = Event)]
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

impl From<JsEvent> for Event {
    fn from(event: JsEvent) -> Self {
        event.inner
    }
}

#[wasm_bindgen(js_class = Event)]
impl JsEvent {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> JsEventId {
        self.inner.id.into()
    }

    #[wasm_bindgen(getter)]
    pub fn pubkey(&self) -> JsPublicKey {
        self.inner.pubkey.into()
    }

    #[wasm_bindgen(js_name = createdAt, getter)]
    pub fn created_at(&self) -> u64 {
        self.inner.created_at.as_u64()
    }

    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> u64 {
        self.inner.kind.into()
    }

    /* #[wasm_bindgen(getter)]
    pub fn tags(&self) -> JsTags {
        self.inner.tags.iter().map(|t| t.as_vec()).collect()
    } */

    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.inner.content.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn signature(&self) -> String {
        self.inner.sig.to_string()
    }

    #[wasm_bindgen]
    pub fn verify(&self) -> bool {
        self.inner.verify().is_ok()
    }

    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: String) -> Result<JsEvent> {
        Ok(Self {
            inner: Event::from_json(json).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }
}
