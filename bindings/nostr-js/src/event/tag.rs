// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::Tag;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Tag[]")]
    pub type JsTagArray;
}

#[wasm_bindgen(js_name = Tag)]
pub struct JsTag {
    inner: Vec<String>,
}

impl From<Tag> for JsTag {
    fn from(tag: Tag) -> Self {
        Self {
            inner: tag.to_vec(),
        }
    }
}

#[wasm_bindgen(js_class = Tag)]
impl JsTag {
    #[wasm_bindgen(constructor)]
    pub fn new(tag: Vec<String>) -> Self {
        Self { inner: tag }
    }

    /// Get tag as vector of string
    #[wasm_bindgen(js_name = asVec)]
    pub fn as_vec(&self) -> Vec<String> {
        self.inner.clone()
    }

    /// Consume the tag and return vector of string
    #[wasm_bindgen(js_name = toVec)]
    pub fn to_vec(self) -> Vec<String> {
        self.inner
    }
}
