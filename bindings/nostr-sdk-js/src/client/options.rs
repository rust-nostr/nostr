// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::Options;
use wasm_bindgen::prelude::*;

use crate::duration::JsDuration;

#[wasm_bindgen(js_name = Options)]
pub struct JsOptions {
    inner: Options,
}

impl Deref for JsOptions {
    type Target = Options;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Options> for JsOptions {
    fn from(inner: Options) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Options)]
impl JsOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Options::new().shutdown_on_drop(true),
        }
    }

    #[wasm_bindgen(js_name = waitForSend)]
    pub fn wait_for_send(self, wait: bool) -> Self {
        self.inner.wait_for_send(wait).into()
    }

    #[wasm_bindgen(js_name = waitForSubscription)]
    pub fn wait_for_subscription(self, wait: bool) -> Self {
        self.inner.wait_for_subscription(wait).into()
    }

    pub fn difficulty(self, difficulty: u8) -> Self {
        self.inner.difficulty(difficulty).into()
    }

    #[wasm_bindgen(js_name = reqFiltersChunkSize)]
    pub fn req_filters_chunk_size(self, req_filters_chunk_size: u8) -> Self {
        self.inner
            .req_filters_chunk_size(req_filters_chunk_size)
            .into()
    }

    #[wasm_bindgen(js_name = skipDisconnectedRelays)]
    pub fn skip_disconnected_relays(self, skip: bool) -> Self {
        self.inner.skip_disconnected_relays(skip).into()
    }

    pub fn timeout(self, timeout: &JsDuration) -> Self {
        self.inner.timeout(**timeout).into()
    }

    /// Connection timeout (default: None)
    ///
    /// If set to `None`, the client will try to connect to the relays without waiting.
    #[wasm_bindgen(js_name = connectionTimeout)]
    pub fn connection_timeout(self, connection_timeout: Option<JsDuration>) -> Self {
        self.inner
            .connection_timeout(connection_timeout.map(|d| *d))
            .into()
    }

    #[wasm_bindgen(js_name = sendTimeout)]
    pub fn send_timeout(self, send_timeout: Option<JsDuration>) -> Self {
        self.inner.send_timeout(send_timeout.map(|d| *d)).into()
    }
}
