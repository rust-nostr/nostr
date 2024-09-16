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

    #[wasm_bindgen(js_name = waitForSend)]
    pub fn wait_for_send(self, wait: bool) -> Self {
        self.inner.wait_for_send(wait).into()
    }

    #[wasm_bindgen(js_name = waitForSubscription)]
    pub fn wait_for_subscription(self, wait: bool) -> Self {
        self.inner.wait_for_subscription(wait).into()
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

#[wasm_bindgen(js_name = EventSource)]
pub struct JsEventSource {
    inner: EventSource,
}

impl Deref for JsEventSource {
    type Target = EventSource;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = EventSource)]
impl JsEventSource {
    /// Database only
    pub fn database() -> Self {
        Self {
            inner: nostr_sdk::EventSource::Database,
        }
    }

    /// Relays only
    pub fn relays(timeout: Option<JsDuration>) -> Self {
        Self {
            inner: nostr_sdk::EventSource::relays(timeout.map(|t| *t)),
        }
    }

    /// From specific relays only
    #[wasm_bindgen(js_name = specificRelays)]
    pub fn specific_relays(urls: Vec<String>, timeout: Option<JsDuration>) -> Self {
        Self {
            inner: nostr_sdk::EventSource::specific_relays(urls, timeout.map(|t| *t)),
        }
    }

    /// Both from database and relays
    pub fn both(timeout: Option<JsDuration>) -> Self {
        Self {
            inner: nostr_sdk::EventSource::both(timeout.map(|t| *t)),
        }
    }

    /// Both from database and specific relays
    #[wasm_bindgen(js_name = bothWithSpecificRelays)]
    pub fn both_with_specific_relays(urls: Vec<String>, timeout: Option<JsDuration>) -> Self {
        Self {
            inner: nostr_sdk::EventSource::both_with_specific_relays(urls, timeout.map(|t| *t)),
        }
    }
}
