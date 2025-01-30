// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

/// Relay Limits
#[wasm_bindgen(js_name = RelayLimits)]
pub struct JsRelayLimits {
    inner: RelayLimits,
}

impl Deref for JsRelayLimits {
    type Target = RelayLimits;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = RelayLimits)]
impl JsRelayLimits {
    /// Construct with default limits
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RelayLimits::default(),
        }
    }

    /// Disable all limits
    pub fn disable() -> Self {
        Self {
            inner: RelayLimits::disable(),
        }
    }

    /// Maximum size of normalised JSON, in bytes (default: 5MB)
    #[wasm_bindgen(js_name = messageMaxSize)]
    pub fn message_max_size(mut self, max_size: Option<u32>) -> Self {
        self.inner.messages.max_size = max_size;
        self
    }

    /// Maximum size of normalised JSON, in bytes (default: 70_000)
    #[wasm_bindgen(js_name = eventMaxSize)]
    pub fn event_max_size(mut self, max_size: Option<u32>) -> Self {
        self.inner.events.max_size = max_size;
        self
    }

    /// Maximum size per kind of normalised JSON, in bytes
    #[wasm_bindgen(js_name = eventMaxSizePerKind)]
    pub fn event_max_size_per_kind(mut self, kind: u16, max_size: Option<u32>) -> Self {
        self.inner.events = self
            .inner
            .events
            .set_max_size_per_kind(Kind::from(kind), max_size);
        self
    }

    /// Maximum number of tags allowed (default: 2_000)
    #[wasm_bindgen(js_name = eventMaxNumTags)]
    pub fn event_max_num_tags(mut self, max_num_tags: Option<u16>) -> Self {
        self.inner.events.max_num_tags = max_num_tags;
        self
    }

    /// Maximum number of tags per kind allowed
    #[wasm_bindgen(js_name = eventMaxNumTagsPerKind)]
    pub fn event_max_num_tags_per_kind(mut self, kind: u16, max_num_tags: Option<u16>) -> Self {
        self.inner.events = self
            .inner
            .events
            .set_max_num_tags_per_kind(Kind::from(kind), max_num_tags);
        self
    }
}
