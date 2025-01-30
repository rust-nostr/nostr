// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod builder;
pub mod id;
pub mod kind;
pub mod tag;
pub mod unsigned;

pub use self::builder::JsEventBuilder;
pub use self::id::JsEventId;
pub use self::kind::JsKind;
pub use self::tag::{JsTag, JsTags};
pub use self::unsigned::JsUnsignedEvent;
use super::key::JsPublicKey;
use super::types::JsTimestamp;
use crate::error::{into_err, Result};

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

    /// Get event author (`pubkey` field)
    #[wasm_bindgen(getter)]
    pub fn author(&self) -> JsPublicKey {
        self.inner.pubkey.into()
    }

    #[wasm_bindgen(js_name = createdAt, getter)]
    pub fn created_at(&self) -> JsTimestamp {
        self.inner.created_at.into()
    }

    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> JsKind {
        self.inner.kind.into()
    }

    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> JsTags {
        self.inner.tags.clone().into()
    }

    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.inner.content.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn signature(&self) -> String {
        self.inner.sig.to_string()
    }

    /// Verify both `EventId` and `Signature`
    #[wasm_bindgen]
    pub fn verify(&self) -> bool {
        self.inner.verify().is_ok()
    }

    /// Verify if the `EventId` it's composed correctly
    #[wasm_bindgen(js_name = verifyId)]
    pub fn verify_id(&self) -> bool {
        self.inner.verify_id()
    }

    /// Verify only event `Signature`
    #[wasm_bindgen(js_name = verifySignature)]
    pub fn verify_signature(&self) -> bool {
        self.inner.verify_signature()
    }

    /// Check POW
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    #[wasm_bindgen(js_name = checkPow)]
    pub fn check_pow(&self, difficulty: u8) -> bool {
        self.inner.check_pow(difficulty)
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[wasm_bindgen(js_name = isExpired)]
    pub fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[wasm_bindgen(js_name = isExpiredAt)]
    pub fn is_expired_at(&self, now: &JsTimestamp) -> bool {
        self.inner.is_expired_at(now.deref())
    }

    /// Check if it's a protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    #[inline]
    #[wasm_bindgen(js_name = isProtected)]
    pub fn is_protected(&self) -> bool {
        self.inner.is_protected()
    }

    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<JsEvent> {
        Ok(Self {
            inner: Event::from_json(json).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> Result<String> {
        self.inner.try_as_json().map_err(into_err)
    }

    #[wasm_bindgen(js_name = asPrettyJson)]
    pub fn as_pretty_json(&self) -> Result<String> {
        self.inner.try_as_pretty_json().map_err(into_err)
    }
}
