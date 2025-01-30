// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::async_utility::futures_util::stream::AbortHandle;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = AbortHandle)]
pub struct JsAbortHandle {
    inner: AbortHandle,
}

impl From<AbortHandle> for JsAbortHandle {
    fn from(inner: AbortHandle) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = AbortHandle)]
impl JsAbortHandle {
    /// Abort thread
    pub fn abort(&self) {
        if self.is_aborted() {
            tracing::warn!("Thread already aborted");
        } else {
            self.inner.abort();
            tracing::info!("Thread aborted!");
        }
    }

    /// Check if thread is aborted
    pub fn is_aborted(&self) -> bool {
        self.inner.is_aborted()
    }
}
