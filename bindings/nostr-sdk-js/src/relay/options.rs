// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::{NegentropyDirection, NegentropyOptions};
use wasm_bindgen::prelude::*;

use crate::duration::JsDuration;

#[wasm_bindgen(js_name = NegentropyDirection)]
pub enum JsNegentropyDirection {
    Up,
    Down,
    Both,
}

impl From<JsNegentropyDirection> for NegentropyDirection {
    fn from(value: JsNegentropyDirection) -> Self {
        match value {
            JsNegentropyDirection::Up => Self::Up,
            JsNegentropyDirection::Down => Self::Down,
            JsNegentropyDirection::Both => Self::Both,
        }
    }
}

#[wasm_bindgen(js_name = NegentropyOptions)]
pub struct JsNegentropyOptions {
    inner: NegentropyOptions,
}

impl Deref for JsNegentropyOptions {
    type Target = NegentropyOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NegentropyOptions> for JsNegentropyOptions {
    fn from(inner: NegentropyOptions) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NegentropyOptions)]
impl JsNegentropyOptions {
    /// New default options
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: NegentropyOptions::new(),
        }
    }

    /// Timeout to check if negentropy it's supported (default: 10 secs)
    #[wasm_bindgen(js_name = initialTimeout)]
    pub fn initial_timeout(self, timeout: JsDuration) -> Self {
        self.inner.initial_timeout(*timeout).into()
    }

    /// Negentropy Sync direction (default: down)
    pub fn direction(self, direction: JsNegentropyDirection) -> Self {
        self.inner.direction(direction.into()).into()
    }
}
