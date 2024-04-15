// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::nips::nip07::Nip07Signer;
use nostr::PublicKey;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::{JsEvent, JsUnsignedEvent};
use crate::key::JsPublicKey;

/// NIP07 Signer for interaction with browser extensions (ex. Alby)
///
/// <https://github.com/aljazceru/awesome-nostr#nip-07-browser-extensions>
#[wasm_bindgen(js_name = Nip07Signer)]
pub struct JsNip07Signer {
    inner: Nip07Signer,
}

impl Deref for JsNip07Signer {
    type Target = Nip07Signer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = Nip07Signer)]
impl JsNip07Signer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<JsNip07Signer> {
        Ok(Self {
            inner: Nip07Signer::new().map_err(into_err)?,
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
            .nip04_encrypt(**public_key, plaintext)
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
            .nip04_decrypt(**public_key, ciphertext)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip44Encrypt)]
    pub async fn nip44_encrypt(&self, public_key: &JsPublicKey, plaintext: &str) -> Result<String> {
        self.inner
            .nip44_encrypt(**public_key, plaintext)
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
            .nip44_decrypt(**public_key, ciphertext)
            .await
            .map_err(into_err)
    }
}
