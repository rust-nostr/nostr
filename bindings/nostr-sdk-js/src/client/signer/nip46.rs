// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use js_sys::Array;
use nostr_sdk::signer::Nip46Signer;
use wasm_bindgen::prelude::*;

use crate::duration::JsDuration;
use crate::error::{into_err, Result};
use crate::protocol::key::{JsKeys, JsPublicKey};
use crate::protocol::nips::nip46::JsNostrConnectURI;
use crate::JsStringArray;

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
    /// Construct Nostr Connect client
    #[wasm_bindgen]
    pub fn new(
        uri: &JsNostrConnectURI,
        app_keys: &JsKeys,
        timeout: &JsDuration,
    ) -> Result<JsNip46Signer> {
        Ok(Self {
            inner: Nip46Signer::new(
                uri.deref().clone(),
                app_keys.deref().clone(),
                **timeout,
                None,
            )
            .map_err(into_err)?,
        })
    }

    /// Get signer relays
    #[wasm_bindgen]
    pub fn relays(&self) -> JsStringArray {
        self.inner
            .relays()
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
            .copied()
            .map_err(into_err)?
            .into())
    }

    /// Get `bunker` URI
    #[wasm_bindgen(js_name = bunkerUri)]
    pub async fn bunker_uri(&self) -> Result<JsNostrConnectURI> {
        Ok(self.inner.bunker_uri().await.map_err(into_err)?.into())
    }
}
