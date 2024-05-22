// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};

#[wasm_bindgen(js_name = NostrConnectMetadata)]
pub struct JsNostrConnectMetadata {
    inner: NostrConnectMetadata,
}

impl Deref for JsNostrConnectMetadata {
    type Target = NostrConnectMetadata;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NostrConnectMetadata> for JsNostrConnectMetadata {
    fn from(inner: NostrConnectMetadata) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NostrConnectMetadata)]
impl JsNostrConnectMetadata {
    /// New Nostr Connect Metadata
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        Self {
            inner: NostrConnectMetadata::new(name),
        }
    }

    /// URL of the website requesting the connection
    pub fn url(self, url: &str) -> Result<JsNostrConnectMetadata> {
        let url: Url = Url::parse(url).map_err(into_err)?;
        Ok(self.inner.url(url).into())
    }

    /// Description of the `App`
    pub fn description(self, description: &str) -> Self {
        self.inner.description(description).into()
    }

    /// List of URLs for icons of the `App`
    pub fn icons(self, icons: Vec<String>) -> Self {
        let icons: Vec<Url> = icons
            .into_iter()
            .filter_map(|u| Url::parse(&u).ok())
            .collect();
        self.inner.icons(icons).into()
    }

    /// Serialize as JSON string
    pub fn as_json(&self) -> Result<String> {
        self.inner.try_as_json().map_err(into_err)
    }
}

#[wasm_bindgen(js_name = NostrConnectURI)]
pub struct JsNostrConnectURI {
    inner: NostrConnectURI,
}

impl Deref for JsNostrConnectURI {
    type Target = NostrConnectURI;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NostrConnectURI> for JsNostrConnectURI {
    fn from(inner: NostrConnectURI) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NostrConnectURI)]
impl JsNostrConnectURI {
    #[wasm_bindgen]
    pub fn parse(uri: &str) -> Result<JsNostrConnectURI> {
        Ok(Self {
            inner: NostrConnectURI::parse(uri).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asString)]
    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }
}
