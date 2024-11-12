// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::duration::JsDuration;
use crate::relay::filtering::JsRelayFilteringMode;
use crate::relay::limits::JsRelayLimits;

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
            inner: Options::new(),
        }
    }

    /// Automatically start connection with relays (default: false)
    ///
    /// When set to `true`, there isn't the need of calling the connect methods.
    pub fn autoconnect(self, val: bool) -> Self {
        self.inner.autoconnect(val).into()
    }

    pub fn difficulty(self, difficulty: u8) -> Self {
        self.inner.difficulty(difficulty).into()
    }

    /// Minimum POW difficulty for received events
    #[wasm_bindgen(js_name = minPow)]
    pub fn min_pow(self, difficulty: u8) -> Self {
        self.inner.min_pow(difficulty).into()
    }

    #[wasm_bindgen(js_name = reqFiltersChunkSize)]
    pub fn req_filters_chunk_size(self, req_filters_chunk_size: u8) -> Self {
        self.inner
            .req_filters_chunk_size(req_filters_chunk_size)
            .into()
    }

    pub fn timeout(self, timeout: &JsDuration) -> Self {
        self.inner.timeout(**timeout).into()
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[wasm_bindgen(js_name = automaticAuthentication)]
    pub fn automatic_authentication(self, enabled: bool) -> Self {
        self.inner.automatic_authentication(enabled).into()
    }

    /// Enable gossip model (default: false)
    pub fn gossip(self, enable: bool) -> Self {
        self.inner.gossip(enable).into()
    }

    /// Set custom relay limits
    #[wasm_bindgen(js_name = relayLimits)]
    pub fn relay_limits(self, limits: &JsRelayLimits) -> Self {
        self.inner.relay_limits(limits.deref().clone()).into()
    }

    /// Set filtering mode (default: blacklist)
    #[wasm_bindgen(js_name = filteringMode)]
    pub fn filtering_mode(self, mode: JsRelayFilteringMode) -> Self {
        self.inner.filtering_mode(mode.into()).into()
    }
}
