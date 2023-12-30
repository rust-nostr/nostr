// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip07;
use nostr::secp256k1::XOnlyPublicKey;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::JsUnsignedEvent;
use crate::{JsEvent, JsPublicKey};

/// NIP07 Signer for interaction with browser extensions (ex. Alby)
///
/// <https://github.com/aljazceru/awesome-nostr#nip-07-browser-extensions>
#[wasm_bindgen]
pub struct Nip07Signer {
    inner: nip07::Signer,
}

#[wasm_bindgen]
impl Nip07Signer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Nip07Signer> {
        Ok(Self {
            inner: nip07::Signer::new().map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = getPublicKey)]
    pub async fn get_public_key(&self) -> Result<JsPublicKey> {
        let public_key: XOnlyPublicKey = self.inner.get_public_key().await.map_err(into_err)?;
        Ok(public_key.into())
    }

    #[wasm_bindgen(js_name = signEvent)]
    pub async fn sign_event(&self, unsigned: JsUnsignedEvent) -> Result<JsEvent> {
        Ok(self
            .inner
            .sign_event(unsigned.into())
            .await
            .map_err(into_err)?
            .into())
    }

    #[wasm_bindgen(js_name = nip04Encrypt)]
    pub async fn nip04_encrypt(
        &self,
        public_key: &JsPublicKey,
        plaintext: String,
    ) -> Result<String> {
        self.inner
            .nip04_encrypt(**public_key, plaintext)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip04Decrypt)]
    pub async fn nip04_decrypt(
        &self,
        public_key: &JsPublicKey,
        ciphertext: String,
    ) -> Result<String> {
        self.inner
            .nip04_encrypt(**public_key, ciphertext)
            .await
            .map_err(into_err)
    }
}
