// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};

#[wasm_bindgen(js_name = Metadata)]
pub struct JsMetadata {
    inner: Metadata,
}

impl From<Metadata> for JsMetadata {
    fn from(metadata: Metadata) -> Self {
        Self { inner: metadata }
    }
}

impl Deref for JsMetadata {
    type Target = Metadata;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = Metadata)]
impl JsMetadata {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Metadata::new(),
        }
    }

    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<JsMetadata> {
        Ok(Self {
            inner: Metadata::from_json(json).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }

    pub fn name(&self, name: &str) -> Self {
        Self {
            inner: self.inner.to_owned().name(name),
        }
    }

    #[wasm_bindgen(js_name = displayName)]
    pub fn display_name(&self, display_name: &str) -> Self {
        Self {
            inner: self.inner.to_owned().display_name(display_name),
        }
    }

    pub fn about(&self, about: &str) -> Self {
        Self {
            inner: self.inner.to_owned().about(about),
        }
    }

    pub fn website(&self, url: &str) -> Result<JsMetadata> {
        let url = Url::parse(url).map_err(into_err)?;
        Ok(Self {
            inner: self.inner.to_owned().website(url),
        })
    }

    pub fn picture(&self, url: &str) -> Result<JsMetadata> {
        let url = Url::parse(url).map_err(into_err)?;
        Ok(Self {
            inner: self.inner.to_owned().picture(url),
        })
    }

    pub fn banner(&self, url: &str) -> Result<JsMetadata> {
        let url = Url::parse(url).map_err(into_err)?;
        Ok(Self {
            inner: self.inner.to_owned().banner(url),
        })
    }

    pub fn nip05(&self, nip05: &str) -> Self {
        Self {
            inner: self.inner.to_owned().nip05(nip05),
        }
    }

    pub fn lud06(&self, lud06: &str) -> Self {
        Self {
            inner: self.inner.to_owned().lud06(lud06),
        }
    }

    pub fn lud16(&self, lud16: &str) -> Self {
        Self {
            inner: self.inner.to_owned().lud16(lud16),
        }
    }
}
