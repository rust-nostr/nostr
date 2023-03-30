// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use js_sys::Promise;
use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::into_err;
use crate::future::future_to_promise;

#[wasm_bindgen(js_name = RelayInformationDocument)]
pub struct JsRelayInformationDocument {
    inner: RelayInformationDocument,
}

impl From<RelayInformationDocument> for JsRelayInformationDocument {
    fn from(document: RelayInformationDocument) -> Self {
        Self { inner: document }
    }
}

#[wasm_bindgen(js_class = RelayInformationDocument)]
impl JsRelayInformationDocument {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RelayInformationDocument::new(),
        }
    }

    #[wasm_bindgen]
    pub fn get(url: String) -> Promise {
        future_to_promise(async move {
            let url = Url::parse(&url).map_err(into_err)?;
            Ok(Self {
                inner: RelayInformationDocument::get(url).await.map_err(into_err)?,
            })
        })
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn pubkey(&self) -> Option<String> {
        self.inner.pubkey.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn contact(&self) -> Option<String> {
        self.inner.contact.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn supported_nips(&self) -> Option<Vec<u16>> {
        self.inner.supported_nips.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn software(&self) -> Option<String> {
        self.inner.software.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn version(&self) -> Option<String> {
        self.inner.version.clone()
    }
}
