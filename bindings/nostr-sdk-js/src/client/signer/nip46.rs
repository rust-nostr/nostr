// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_js::error::{into_err, Result};
use nostr_js::key::{JsKeys, JsPublicKey};
use nostr_js::nips::nip46::{JsNostrConnectMetadata, JsNostrConnectURI};
use nostr_sdk::client::Nip46Signer;
use nostr_sdk::Url;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Nip46Signer)]
pub struct JsNip46Signer {
    inner: Nip46Signer,
}

impl Deref for JsNip46Signer {
    type Target = Nip46Signer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Nip46Signer> for JsNip46Signer {
    fn from(inner: Nip46Signer) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Nip46Signer)]
impl JsNip46Signer {
    /// New NIP46 remote signer
    #[wasm_bindgen(constructor)]
    pub fn new(
        relay_url: String,
        app_keys: &JsKeys,
        signer_public_key: Option<JsPublicKey>,
    ) -> Result<JsNip46Signer> {
        let relay_url: Url = Url::parse(&relay_url).map_err(into_err)?;
        Ok(Self {
            inner: Nip46Signer::new(
                relay_url,
                app_keys.deref().clone(),
                signer_public_key.map(|p| *p),
            ),
        })
    }

    /// Get signer relay url
    #[wasm_bindgen(js_name = relayUrl)]
    pub fn relay_url(&self) -> String {
        self.inner.relay_url().to_string()
    }

    /// Get signer [`XOnlyPublicKey`]
    #[wasm_bindgen(js_name = signerPublicKey)]
    pub async fn signer_public_key(&self) -> Option<JsPublicKey> {
        self.inner.signer_public_key().await.map(|p| p.into())
    }

    #[wasm_bindgen(js_name = nostrConnectUri)]
    pub fn nostr_connect_uri(&self, metadata: &JsNostrConnectMetadata) -> JsNostrConnectURI {
        self.inner
            .nostr_connect_uri(metadata.deref().clone())
            .into()
    }
}
