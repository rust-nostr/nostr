// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use js_sys::Array;
use nostr::prelude::*;
use wasm_bindgen::prelude::*;

mod builder;
mod id;
mod tag;
mod unsigned;

pub use self::builder::JsEventBuilder;
pub use self::id::JsEventId;
pub use self::tag::{JsTag, JsTagArray};
pub use self::unsigned::JsUnsignedEvent;
use crate::error::{into_err, Result};
use crate::key::JsPublicKey;
use crate::types::JsTimestamp;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Event[]")]
    pub type JsEventArray;
}

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
        self.inner.id().into()
    }

    /// Get event author (`pubkey` field)
    #[wasm_bindgen(getter)]
    pub fn author(&self) -> JsPublicKey {
        self.inner.author().into()
    }

    #[wasm_bindgen(js_name = createdAt, getter)]
    pub fn created_at(&self) -> JsTimestamp {
        self.inner.created_at().into()
    }

    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> f64 {
        self.inner.kind().as_f64()
    }

    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> JsTagArray {
        self.inner
            .iter_tags()
            .cloned()
            .map(|t| {
                let e: JsTag = t.into();
                JsValue::from(e)
            })
            .collect::<Array>()
            .unchecked_into()
    }

    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.inner.content().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn signature(&self) -> String {
        self.inner.signature().to_string()
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
