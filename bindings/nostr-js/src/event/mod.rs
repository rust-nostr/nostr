// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

pub mod builder;
pub mod id;
pub mod tag;
pub mod unsigned;

pub use self::builder::JsEventBuilder;
pub use self::id::JsEventId;
pub use self::tag::JsTag;
pub use self::unsigned::JsUnsignedEvent;
use crate::error::{into_err, Result};
use crate::key::JsPublicKey;
use crate::nips::nip01::JsCoordinate;
use crate::types::JsTimestamp;

#[wasm_bindgen]
extern "C" {
    /// Event array
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
    pub fn kind(&self) -> u16 {
        self.inner.kind.as_u16()
    }

    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> Vec<JsTag> {
        self.inner.tags.iter().cloned().map(JsTag::from).collect()
    }

    /// Get content of **first** tag that match tag kind (ex. `e`, `p`, `title`, ...)
    #[wasm_bindgen(js_name = getTagContent)]
    pub fn get_tag_content(&self, kind: &str) -> Option<String> {
        self.inner
            .get_tag_content(TagKind::from(kind))
            .map(|c| c.to_string())
    }

    /// Get content of all tags that match `TagKind`.
    #[wasm_bindgen(js_name = getTagsContent)]
    pub fn get_tags_content(&self, kind: &str) -> Vec<String> {
        self.inner
            .get_tags_content(TagKind::from(kind))
            .into_iter()
            .map(|c| c.to_string())
            .collect()
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
        self.inner.verify_id().is_ok()
    }

    /// Verify only event `Signature`
    #[wasm_bindgen(js_name = verifySignature)]
    pub fn verify_signature(&self) -> bool {
        self.inner.verify_signature().is_ok()
    }

    /// Check POW
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    #[wasm_bindgen(js_name = checkPow)]
    pub fn check_pow(&self, difficulty: u8) -> bool {
        self.inner.check_pow(difficulty)
    }

    /// Get `Timestamp` expiration if set
    #[wasm_bindgen]
    pub fn expiration(&self) -> Option<JsTimestamp> {
        self.inner.expiration().map(|t| (*t).into())
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

    /// Check if `Kind` is a NIP90 job request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = isJobRequest)]
    pub fn is_job_request(&self) -> bool {
        self.inner.is_job_request()
    }

    /// Check if `Kind` is a NIP90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = isJobResult)]
    pub fn is_job_result(&self) -> bool {
        self.inner.is_job_result()
    }

    /// Check if event `Kind` is `Regular`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isRegular)]
    pub fn is_regular(&self) -> bool {
        self.inner.is_regular()
    }

    /// Check if event `Kind` is `Replaceable`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isReplaceable)]
    pub fn is_replaceable(&self) -> bool {
        self.inner.is_replaceable()
    }

    /// Check if event `Kind` is `Ephemeral`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isEphemeral)]
    pub fn is_ephemeral(&self) -> bool {
        self.inner.is_ephemeral()
    }

    /// Check if event `Kind` is `Parameterized replaceable`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isParametrizedReplaceable)]
    pub fn is_parameterized_replaceable(&self) -> bool {
        self.inner.is_parameterized_replaceable()
    }

    /// Extract identifier (`d` tag), if exists.
    #[wasm_bindgen]
    pub fn identifier(&self) -> Option<String> {
        self.inner.identifier().map(|i| i.to_string())
    }

    /// Extract public keys from tags (`p` tag)
    ///
    /// **This method extract ONLY supported standard variants**
    #[wasm_bindgen(js_name = publicKeys)]
    pub fn public_keys(&self) -> Vec<JsPublicKey> {
        self.inner.public_keys().map(|p| (*p).into()).collect()
    }

    /// Extract event IDs from tags (`e` tag)
    ///
    /// **This method extract ONLY supported standard variants**
    #[wasm_bindgen(js_name = eventIds)]
    pub fn event_ids(&self) -> Vec<JsEventId> {
        self.inner.event_ids().map(|e| (*e).into()).collect()
    }

    /// Extract coordinates from tags (`a` tag)
    ///
    /// **This method extract ONLY supported standard variants**
    #[wasm_bindgen]
    pub fn coordinates(&self) -> Vec<JsCoordinate> {
        self.inner.coordinates().map(|c| c.clone().into()).collect()
    }

    /// Extract hashtags from tags (`t` tag)
    ///
    /// **This method extract ONLY supported standard variants**
    #[wasm_bindgen]
    pub fn hashtags(&self) -> Vec<String> {
        self.inner.hashtags().map(|t| t.to_owned()).collect()
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
