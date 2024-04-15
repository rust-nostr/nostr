// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip98::HttpData;
use nostr::UncheckedUrl;
use wasm_bindgen::prelude::*;

use crate::event::tag::JsHttpMethod;

#[wasm_bindgen(js_name = HttpData)]
pub struct JsHttpData {
    inner: HttpData,
}

impl Deref for JsHttpData {
    type Target = HttpData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<HttpData> for JsHttpData {
    fn from(inner: HttpData) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = HttpData)]
impl JsHttpData {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str, method: JsHttpMethod) -> Self {
        Self {
            inner: HttpData::new(UncheckedUrl::from(url), method.into()),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn urls(&self) -> String {
        self.inner.url.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn method(&self) -> JsHttpMethod {
        self.inner.method.clone().into()
    }

    #[wasm_bindgen(getter)]
    pub fn payload(&self) -> Option<String> {
        self.inner.payload.map(|s| s.to_string())
    }
}
