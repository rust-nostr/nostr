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
    #[inline]
    fn from(inner: Metadata) -> Self {
        Self { inner }
    }
}

impl Deref for JsMetadata {
    type Target = Metadata;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = Metadata)]
impl JsMetadata {
    #[inline]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Metadata::new(),
        }
    }

    #[inline]
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<JsMetadata> {
        Ok(Self {
            inner: Metadata::from_json(json).map_err(into_err)?,
        })
    }

    #[inline]
    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> Result<String> {
        self.inner.try_as_json().map_err(into_err)
    }

    #[inline]
    #[wasm_bindgen(js_name = name)]
    pub fn set_name(self, name: &str) -> Self {
        self.inner.name(name).into()
    }

    #[inline]
    #[wasm_bindgen(js_name = getName)]
    pub fn get_name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    #[inline]
    #[wasm_bindgen(js_name = displayName)]
    pub fn set_display_name(self, display_name: &str) -> Self {
        self.inner.display_name(display_name).into()
    }

    #[inline]
    #[wasm_bindgen(js_name = getDisplayName)]
    pub fn get_display_name(&self) -> Option<String> {
        self.inner.display_name.clone()
    }

    #[inline]
    #[wasm_bindgen(js_name = about)]
    pub fn set_about(self, about: &str) -> Self {
        self.inner.about(about).into()
    }

    #[inline]
    #[wasm_bindgen(js_name = getAbout)]
    pub fn get_about(&self) -> Option<String> {
        self.inner.about.clone()
    }

    #[inline]
    #[wasm_bindgen(js_name = website)]
    pub fn set_website(self, url: &str) -> Result<JsMetadata> {
        let url = Url::parse(url).map_err(into_err)?;
        Ok(self.inner.website(url).into())
    }

    #[inline]
    #[wasm_bindgen(js_name = getWebsite)]
    pub fn get_website(&self) -> Option<String> {
        self.inner.website.clone()
    }

    #[inline]
    #[wasm_bindgen(js_name = picture)]
    pub fn set_picture(self, url: &str) -> Result<JsMetadata> {
        let url = Url::parse(url).map_err(into_err)?;
        Ok(self.inner.picture(url).into())
    }

    #[inline]
    #[wasm_bindgen(js_name = getPicture)]
    pub fn get_picture(&self) -> Option<String> {
        self.inner.picture.clone()
    }

    #[inline]
    #[wasm_bindgen(js_name = banner)]
    pub fn set_banner(self, url: &str) -> Result<JsMetadata> {
        let url = Url::parse(url).map_err(into_err)?;
        Ok(self.inner.banner(url).into())
    }

    #[inline]
    #[wasm_bindgen(js_name = getBanner)]
    pub fn get_banner(&self) -> Option<String> {
        self.inner.banner.clone()
    }

    #[inline]
    #[wasm_bindgen(js_name = nip05)]
    pub fn set_nip05(self, nip05: &str) -> Self {
        self.inner.nip05(nip05).into()
    }

    #[inline]
    #[wasm_bindgen(js_name = getNip05)]
    pub fn get_nip05(&self) -> Option<String> {
        self.inner.nip05.clone()
    }

    #[inline]
    #[wasm_bindgen(js_name = lud06)]
    pub fn set_lud06(self, lud06: &str) -> Self {
        self.inner.lud06(lud06).into()
    }

    #[inline]
    #[wasm_bindgen(js_name = getLud06)]
    pub fn get_lud06(&self) -> Option<String> {
        self.inner.lud06.clone()
    }

    #[inline]
    #[wasm_bindgen(js_name = lud16)]
    pub fn set_lud16(self, lud16: &str) -> Self {
        self.inner.lud16(lud16).into()
    }

    #[inline]
    #[wasm_bindgen(js_name = getLud16)]
    pub fn get_lud16(&self) -> Option<String> {
        self.inner.lud16.clone()
    }
}
