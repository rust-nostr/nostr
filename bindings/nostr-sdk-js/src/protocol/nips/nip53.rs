// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::protocol::key::JsPublicKey;
use crate::protocol::types::image::JsImageDimensions;

#[wasm_bindgen(js_name = Image)]
pub struct JsImage {
    #[wasm_bindgen(getter_with_clone)]
    pub url: String,
    pub dimensions: Option<JsImageDimensions>,
}

impl From<(Url, Option<ImageDimensions>)> for JsImage {
    fn from(value: (Url, Option<ImageDimensions>)) -> Self {
        Self {
            url: value.0.to_string(),
            dimensions: value.1.map(|d| d.into()),
        }
    }
}

#[wasm_bindgen(js_class = Image)]
impl JsImage {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, dimensions: Option<JsImageDimensions>) -> Self {
        Self { url, dimensions }
    }
}

#[wasm_bindgen(js_name = User)]
pub struct JsUser {
    #[wasm_bindgen(js_name = publicKey)]
    pub public_key: JsPublicKey,
    #[wasm_bindgen(getter_with_clone)]
    pub url: Option<String>,
}

impl From<(PublicKey, Option<RelayUrl>)> for JsUser {
    fn from(value: (PublicKey, Option<RelayUrl>)) -> Self {
        Self {
            public_key: value.0.into(),
            url: value.1.map(|url| url.to_string()),
        }
    }
}

#[wasm_bindgen(js_class = User)]
impl JsUser {
    #[wasm_bindgen(constructor)]
    pub fn new(public_key: JsPublicKey, url: Option<String>) -> Self {
        Self { public_key, url }
    }
}

#[wasm_bindgen(js_name = LiveEventStatus)]
pub struct JsLiveEventStatus {
    inner: LiveEventStatus,
}

impl From<LiveEventStatus> for JsLiveEventStatus {
    fn from(inner: LiveEventStatus) -> Self {
        Self { inner }
    }
}

impl Deref for JsLiveEventStatus {
    type Target = LiveEventStatus;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = LiveEventStatus)]
impl JsLiveEventStatus {
    pub fn planned() -> Self {
        Self {
            inner: LiveEventStatus::Planned,
        }
    }

    pub fn live() -> Self {
        Self {
            inner: LiveEventStatus::Live,
        }
    }

    pub fn ended() -> Self {
        Self {
            inner: LiveEventStatus::Ended,
        }
    }

    pub fn custom(string: String) -> Self {
        Self {
            inner: LiveEventStatus::Custom(string),
        }
    }
}

#[wasm_bindgen(js_name = LiveEventHost)]
pub struct JsLiveEventHost {
    inner: LiveEventHost,
}

impl From<LiveEventHost> for JsLiveEventHost {
    fn from(inner: LiveEventHost) -> Self {
        Self { inner }
    }
}

impl Deref for JsLiveEventHost {
    type Target = LiveEventHost;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = LiveEventHost)]
impl JsLiveEventHost {
    #[wasm_bindgen(getter, js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    #[wasm_bindgen(getter, js_name = relayUrl)]
    pub fn relay_url(&self) -> Option<String> {
        self.inner.relay_url.clone().map(|url| url.to_string())
    }

    #[wasm_bindgen(getter)]
    pub fn proof(&self) -> Option<String> {
        self.inner.proof.map(|s| s.to_string())
    }
}

#[wasm_bindgen(js_name = LiveEvent)]
pub struct JsLiveEvent {
    inner: LiveEvent,
}

impl Deref for JsLiveEvent {
    type Target = LiveEvent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<LiveEvent> for JsLiveEvent {
    fn from(inner: LiveEvent) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = LiveEvent)]
impl JsLiveEvent {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> Option<String> {
        self.inner.title.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn summary(&self) -> Option<String> {
        self.inner.summary.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn image(&self) -> Option<JsImage> {
        self.inner.image.clone().map(|i| i.into())
    }

    #[wasm_bindgen(getter)]
    pub fn hashtags(&self) -> Vec<String> {
        self.inner.hashtags.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn streaming(&self) -> Option<String> {
        self.inner.streaming.clone().map(|url| url.to_string())
    }

    #[wasm_bindgen(getter)]
    pub fn recording(&self) -> Option<String> {
        self.inner.recording.clone().map(|url| url.to_string())
    }

    #[wasm_bindgen(getter)]
    pub fn starts(&self) -> Option<f64> {
        self.inner.starts.map(|t| t.as_u64() as f64)
    }

    #[wasm_bindgen(getter)]
    pub fn ends(&self) -> Option<f64> {
        self.inner.ends.map(|t| t.as_u64() as f64)
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> Option<JsLiveEventStatus> {
        self.inner.status.clone().map(|s| s.into())
    }

    #[wasm_bindgen(getter, js_name = currentPartecipants)]
    pub fn current_participants(&self) -> Option<f64> {
        self.inner.current_participants.map(|s| s as f64)
    }

    #[wasm_bindgen(getter, js_name = totalPartecipants)]
    pub fn total_participants(&self) -> Option<f64> {
        self.inner.total_participants.map(|s| s as f64)
    }

    #[wasm_bindgen(getter)]
    pub fn relays(&self) -> Vec<String> {
        self.inner
            .relays
            .clone()
            .into_iter()
            .map(|url| url.to_string())
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn host(&self) -> Option<JsLiveEventHost> {
        self.inner.host.clone().map(|s| s.into())
    }

    #[wasm_bindgen(getter)]
    pub fn speakers(&self) -> Vec<JsUser> {
        self.inner
            .speakers
            .clone()
            .into_iter()
            .map(|u| u.into())
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn participants(&self) -> Vec<JsUser> {
        self.inner
            .participants
            .clone()
            .into_iter()
            .map(|u| u.into())
            .collect()
    }
}
