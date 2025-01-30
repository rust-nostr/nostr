// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
#[wasm_bindgen(js_name = Timestamp)]
pub struct JsTimestamp {
    inner: Timestamp,
}

impl From<Timestamp> for JsTimestamp {
    fn from(inner: Timestamp) -> Self {
        Self { inner }
    }
}

impl Deref for JsTimestamp {
    type Target = Timestamp;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = Timestamp)]
impl JsTimestamp {
    /// Get UNIX timestamp (seconds)
    #[wasm_bindgen]
    pub fn now() -> Self {
        Self {
            inner: Timestamp::now(),
        }
    }

    #[wasm_bindgen(js_name = fromSecs)]
    pub fn from_secs(secs: f64) -> Self {
        Self {
            inner: Timestamp::from_secs(secs as u64),
        }
    }

    /// Get timestamp as seconds
    #[wasm_bindgen(js_name = asSecs)]
    pub fn as_secs(&self) -> f64 {
        self.inner.as_u64() as f64
    }

    /// Convert `Timestamp` to human datetime
    #[wasm_bindgen(js_name = toHumanDatetime)]
    pub fn to_human_datetime(&self) -> String {
        self.inner.to_human_datetime()
    }
}
