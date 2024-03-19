// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use js_sys::Array;
use nostr_js::error::{into_err, Result};
use nostr_js::key::{JsKeys, JsPublicKey};
use nostr_js::nips::nip46::{JsNostrConnectMetadata, JsNostrConnectURI};
use nostr_js::JsStringArray;
use nostr_sdk::signer::Nip46Signer;
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
        uri: &JsNostrConnectURI,
        app_keys: Option<JsKeys>,
        timeout: &JsDuration,
    ) -> Result<JsNip46Signer> {
        Ok(Self {
            inner: Nip46Signer::new(
                uri.deref().clone(),
                app_keys.map(|k| k.deref().clone()),
                **timeout,
                None,
            )
            .await
            .map_err(into_err)?,
        })
    }

    /// Get signer relays
    #[wasm_bindgen]
    pub async fn relays(&self) -> JsStringArray {
        self.inner
            .relays()
            .await
            .into_iter()
            .map(|u| JsValue::from(u.to_string()))
            .collect::<Array>()
            .unchecked_into()
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
    pub async fn nostr_connect_uri(&self, metadata: &JsNostrConnectMetadata) -> JsNostrConnectURI {
        self.inner
            .nostr_connect_uri(metadata.deref().clone())
            .await
            .into()
    }
}
