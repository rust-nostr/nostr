// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nwc::prelude::*;
use wasm_bindgen::prelude::*;

use crate::duration::JsDuration;

/// NWC options
#[wasm_bindgen(js_name = NostrWalletConnectOptions)]
pub struct JsNostrWalletConnectOptions {
    inner: NostrWalletConnectOptions,
}

impl Deref for JsNostrWalletConnectOptions {
    type Target = NostrWalletConnectOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NostrWalletConnectOptions> for JsNostrWalletConnectOptions {
    fn from(inner: NostrWalletConnectOptions) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NostrWalletConnectOptions)]
impl JsNostrWalletConnectOptions {
    /// New default NWC options
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: NostrWalletConnectOptions::new(),
        }
    }

    /// Set NWC requests timeout (default: 10 secs)
    pub fn timeout(self, timeout: &JsDuration) -> Self {
        self.inner.timeout(**timeout).into()
    }
}
