// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::Tag;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};

#[wasm_bindgen(js_name = Tag)]
pub struct JsTag {
    inner: Tag,
}

impl From<Tag> for JsTag {
    fn from(inner: Tag) -> Self {
        Self { inner }
    }
}

impl From<JsTag> for Tag {
    fn from(tag: JsTag) -> Self {
        tag.inner
    }
}

#[wasm_bindgen(js_class = Tag)]
impl JsTag {
    #[wasm_bindgen]
    pub fn parse(tag: Vec<String>) -> Result<JsTag> {
        Ok(Self {
            inner: Tag::parse(tag).map_err(into_err)?,
        })
    }

    /// Get tag as vector of string
    #[wasm_bindgen(js_name = asVec)]
    pub fn as_vec(&self) -> Vec<String> {
        self.inner.as_vec()
    }

    /// Consume the tag and return vector of string
    #[wasm_bindgen(js_name = toVec)]
    pub fn to_vec(self) -> Vec<String> {
        self.inner.to_vec()
    }
}
