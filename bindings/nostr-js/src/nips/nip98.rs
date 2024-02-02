// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip98::HttpData;
use nostr::UncheckedUrl;
use wasm_bindgen::prelude::*;

use crate::event::tag::JsHttpMethod;

#[wasm_bindgen(js_name = HttpData)]
pub struct JsHttpData {
    inner: HttpData,
}

impl From<HttpData> for JsHttpData {
    fn from(inner: HttpData) -> Self {
        Self { inner }
    }
}

impl From<JsHttpData> for HttpData {
    fn from(value: JsHttpData) -> Self {
        value.inner
    }
}

#[wasm_bindgen(js_class = HttpData)]
impl JsHttpData {
    pub fn new(url: &str, method: JsHttpMethod) -> Self {
        HttpData::new(UncheckedUrl::from(url), method.into()).into()
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
