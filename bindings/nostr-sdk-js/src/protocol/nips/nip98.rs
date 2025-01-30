// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};

#[wasm_bindgen(js_name = HttpMethod)]
pub enum JsHttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
}

impl From<HttpMethod> for JsHttpMethod {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::GET => Self::GET,
            HttpMethod::POST => Self::POST,
            HttpMethod::PUT => Self::PUT,
            HttpMethod::PATCH => Self::PATCH,
        }
    }
}

impl From<JsHttpMethod> for HttpMethod {
    fn from(value: JsHttpMethod) -> Self {
        match value {
            JsHttpMethod::GET => Self::GET,
            JsHttpMethod::POST => Self::POST,
            JsHttpMethod::PUT => Self::PUT,
            JsHttpMethod::PATCH => Self::PATCH,
        }
    }
}

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
    pub fn new(url: &str, method: JsHttpMethod) -> Result<JsHttpData> {
        Ok(Self {
            inner: HttpData::new(Url::parse(url).map_err(into_err)?, method.into()),
        })
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
