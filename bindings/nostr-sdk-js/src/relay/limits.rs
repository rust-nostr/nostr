// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
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
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RelayLimits::default(),
        }
    }

    /// Maximum size of normalised JSON, in bytes (default: 5_250_000)
    #[wasm_bindgen(js_name = messageMaxSize)]
    pub fn message_max_size(mut self, max_size: u32) -> Self {
        self.inner.messages.max_size = max_size;
        self
    }

    /// Maximum size of normalised JSON, in bytes (default: 70_000)
    #[wasm_bindgen(js_name = eventMaxSize)]
    pub fn event_max_size(mut self, max_size: u32) -> Self {
        self.inner.events.max_size = max_size;
        self
    }

    /// Maximum number of tags allowed (default: 2_000)
    #[wasm_bindgen(js_name = eventMaxNumTags)]
    pub fn event_max_num_tags(mut self, max_num_tags: u16) -> Self {
        self.inner.events.max_num_tags = max_num_tags;
        self
    }
}
