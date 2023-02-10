// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

// use napi::Result;
use nostr::prelude::*;

// use crate::error::into_err;

#[napi(js_name = "RelayInformationDocument")]
pub struct JsRelayInformationDocument {
    inner: RelayInformationDocument,
}

impl From<RelayInformationDocument> for JsRelayInformationDocument {
    fn from(document: RelayInformationDocument) -> Self {
        Self { inner: document }
    }
}

#[napi]
impl JsRelayInformationDocument {
    #[allow(clippy::new_without_default)]
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RelayInformationDocument::new(),
        }
    }

    /* #[napi(factory)]
    pub async fn get(url: String) -> Result<Self> {
        let url = Url::parse(&url).map_err(into_err)?;

        Ok(Self {
            inner: RelayInformationDocument::new(),
        })
    } */

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    #[napi(getter)]
    pub fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    #[napi(getter)]
    pub fn pubkey(&self) -> Option<String> {
        self.inner.pubkey.clone()
    }

    #[napi(getter)]
    pub fn contact(&self) -> Option<String> {
        self.inner.contact.clone()
    }

    #[napi(getter)]
    pub fn supported_nips(&self) -> Option<Vec<u16>> {
        self.inner.supported_nips.clone()
    }

    #[napi(getter)]
    pub fn software(&self) -> Option<String> {
        self.inner.software.clone()
    }

    #[napi(getter)]
    pub fn version(&self) -> Option<String> {
        self.inner.version.clone()
    }
}
