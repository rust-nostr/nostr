// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::event::{JsEvent, JsUnsignedEvent};
use crate::protocol::key::JsPublicKey;

/// Signer for interaction with browser extensions (ex. Alby)
///
/// <https://github.com/aljazceru/awesome-nostr#nip-07-browser-extensions>
///
/// <https://github.com/nostr-protocol/nips/blob/master/07.md>
#[wasm_bindgen(js_name = BrowserSigner)]
pub struct JsBrowserSigner {
    inner: BrowserSigner,
}

impl Deref for JsBrowserSigner {
    type Target = BrowserSigner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = BrowserSigner)]
impl JsBrowserSigner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<JsBrowserSigner> {
        Ok(Self {
            inner: BrowserSigner::new().map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = getPublicKey)]
    pub async fn get_public_key(&self) -> Result<JsPublicKey> {
        let public_key: PublicKey = self.inner.get_public_key().await.map_err(into_err)?;
        Ok(public_key.into())
    }

    #[wasm_bindgen(js_name = signEvent)]
    pub async fn sign_event(&self, unsigned: &JsUnsignedEvent) -> Result<JsEvent> {
        Ok(self
            .inner
            .sign_event(unsigned.deref().clone())
            .await
            .map_err(into_err)?
            .into())
    }

    #[wasm_bindgen(js_name = nip04Encrypt)]
    pub async fn nip04_encrypt(&self, public_key: &JsPublicKey, plaintext: &str) -> Result<String> {
        self.inner
            .nip04_encrypt(public_key.deref(), plaintext)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip04Decrypt)]
    pub async fn nip04_decrypt(
        &self,
        public_key: &JsPublicKey,
        ciphertext: &str,
    ) -> Result<String> {
        self.inner
            .nip04_decrypt(public_key.deref(), ciphertext)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip44Encrypt)]
    pub async fn nip44_encrypt(&self, public_key: &JsPublicKey, plaintext: &str) -> Result<String> {
        self.inner
            .nip44_encrypt(public_key.deref(), plaintext)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip44Decrypt)]
    pub async fn nip44_decrypt(
        &self,
        public_key: &JsPublicKey,
        ciphertext: &str,
    ) -> Result<String> {
        self.inner
            .nip44_decrypt(public_key.deref(), ciphertext)
            .await
            .map_err(into_err)
    }
}
