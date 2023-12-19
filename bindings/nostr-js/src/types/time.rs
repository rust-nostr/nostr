// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

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
    pub fn from_secs(secs: u64) -> Self {
        Self {
            inner: Timestamp::from(secs),
        }
    }

    /// Get timestamp as seconds
    #[wasm_bindgen(js_name = asSecs)]
    pub fn as_secs(&self) -> u64 {
        self.inner.as_u64()
    }

    /// Convert `Timestamp` to human datetime
    #[wasm_bindgen(js_name = toHumanDatetime)]
    pub fn to_human_datetime(&self) -> String {
        self.inner.to_human_datetime()
    }
}
