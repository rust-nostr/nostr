// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_js::error::{into_err, Result};
use nostr_js::key::{JsKeys, JsPublicKey};
use nostr_js::nips::nip46::{JsNostrConnectMetadata, JsNostrConnectURI};
use nostr_sdk::signer::Nip46Signer;
use nostr_sdk::{RelayPoolOptions, Url};
use wasm_bindgen::prelude::*;

use crate::duration::JsDuration;

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
    pub async fn new(
        relay_url: String,
        app_keys: &JsKeys,
        signer_public_key: Option<JsPublicKey>,
        timeout: JsDuration,
    ) -> Result<JsNip46Signer> {
        let relay_url: Url = Url::parse(&relay_url).map_err(into_err)?;
        Ok(Self {
            inner: Nip46Signer::with_opts(
                relay_url,
                app_keys.deref().clone(),
                signer_public_key.map(|p| *p),
                *timeout,
                RelayPoolOptions::new().shutdown_on_drop(true),
            )
            .await
            .map_err(into_err)?,
        })
    }

    /// Get signer relay url
    #[wasm_bindgen(js_name = relayUrl)]
    pub fn relay_url(&self) -> String {
        self.inner.relay_url().to_string()
    }

    /// Get signer public key
    #[wasm_bindgen(js_name = signerPublicKey)]
    pub async fn signer_public_key(&self) -> Result<JsPublicKey> {
        Ok(self
            .inner
            .signer_public_key()
            .await
            .map_err(into_err)?
            .into())
    }

    #[wasm_bindgen(js_name = nostrConnectUri)]
    pub fn nostr_connect_uri(&self, metadata: &JsNostrConnectMetadata) -> JsNostrConnectURI {
        self.inner
            .nostr_connect_uri(metadata.deref().clone())
            .into()
    }
}
