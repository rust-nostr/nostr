// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use core::str::FromStr;

use nostr::nips::nip46::{NostrConnectMetadata, NostrConnectURI};
use nostr::Url;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::JsPublicKey;

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
    pub fn new(name: String) -> Self {
        Self {
            inner: NostrConnectMetadata::new(name),
        }
    }

    /// URL of the website requesting the connection
    pub fn url(self, url: String) -> Result<JsNostrConnectMetadata> {
        let url: Url = Url::parse(&url).map_err(into_err)?;
        Ok(self.inner.url(url).into())
    }

    /// Description of the `App`
    pub fn description(self, description: String) -> Self {
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
    pub fn as_json(&self) -> String {
        self.inner.as_json()
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
    #[wasm_bindgen(constructor)]
    pub fn parse(uri: String) -> Result<JsNostrConnectURI> {
        Ok(Self {
            inner: NostrConnectURI::from_str(&uri).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    #[wasm_bindgen(js_name = relayUrl)]
    pub fn relay_url(&self) -> String {
        self.inner.relay_url.to_string()
    }

    pub fn name(&self) -> String {
        self.inner.metadata.name.clone()
    }

    pub fn url(&self) -> Option<String> {
        self.inner.metadata.url.as_ref().map(|u| u.to_string())
    }

    pub fn description(&self) -> Option<String> {
        self.inner.metadata.description.clone()
    }
}
