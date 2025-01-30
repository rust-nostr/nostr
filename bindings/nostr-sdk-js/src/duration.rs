// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::time::Duration;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Duration)]
pub struct JsDuration {
    inner: Duration,
}

impl Deref for JsDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Duration> for JsDuration {
    fn from(inner: Duration) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Duration)]
impl JsDuration {
    #[wasm_bindgen(js_name = fromSecs)]
    pub fn from_secs(secs: f64) -> Self {
        Self {
            inner: Duration::from_secs_f64(secs),
        }
    }

    #[wasm_bindgen(js_name = fromMillis)]
    pub fn from_millis(millis: u64) -> Self {
        Self {
            inner: Duration::from_millis(millis),
        }
    }

    #[wasm_bindgen(js_name = asSecs)]
    pub fn as_secs(&self) -> f64 {
        self.inner.as_secs_f64()
    }

    #[wasm_bindgen(js_name = asMillis)]
    pub fn as_millis(&self) -> usize {
        self.inner.as_millis() as usize
    }
}
