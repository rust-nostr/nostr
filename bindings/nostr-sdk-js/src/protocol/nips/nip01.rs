// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::event::JsKind;
use crate::protocol::key::JsPublicKey;

#[wasm_bindgen(js_name = Coordinate)]
#[derive(Clone)]
pub struct JsCoordinate {
    inner: Coordinate,
}

impl Deref for JsCoordinate {
    type Target = Coordinate;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Coordinate> for JsCoordinate {
    fn from(inner: Coordinate) -> Self {
        Self { inner }
    }
}

impl From<JsCoordinate> for Coordinate {
    fn from(coordinate: JsCoordinate) -> Self {
        coordinate.inner
    }
}

#[wasm_bindgen(js_class = Coordinate)]
impl JsCoordinate {
    #[wasm_bindgen(constructor)]
    pub fn new(kind: &JsKind, public_key: &JsPublicKey, identifier: Option<String>) -> Self {
        Self {
            inner: Coordinate {
                kind: **kind,
                public_key: **public_key,
                identifier: identifier.unwrap_or_default(),
            },
        }
    }

    /// Parse coordinate from `<kind>:<pubkey>:[<d-tag>]` format, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    #[wasm_bindgen]
    pub fn parse(coordinate: &str) -> Result<JsCoordinate> {
        Ok(Self {
            inner: Coordinate::parse(coordinate).map_err(into_err)?,
        })
    }

    #[inline]
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> JsKind {
        self.inner.kind.into()
    }

    #[wasm_bindgen(getter, js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    #[wasm_bindgen(getter)]
    pub fn identifier(&self) -> String {
        self.inner.identifier.clone()
    }

    /// Check if the coordinate is valid.
    ///
    /// Returns `false` if:
    /// - the `Kind` is `replaceable` and the identifier is not empty
    /// - the `Kind` is `addressable` and the identifier is empty
    pub fn verify(&self) -> bool {
        self.inner.verify().is_ok()
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn _to_string(&self) -> String {
        self.inner.to_string()
    }
}

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

    #[wasm_bindgen(js_name = asPrettyJson)]
    pub fn as_pretty_json(&self) -> Result<String> {
        self.inner.try_as_pretty_json().map_err(into_err)
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
