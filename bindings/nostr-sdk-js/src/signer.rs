// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::connect::JsNostrConnect;
use crate::error::{into_err, Result};
use crate::protocol::event::{JsEvent, JsUnsignedEvent};
use crate::protocol::key::{JsKeys, JsPublicKey};
use crate::protocol::nips::nip07::JsBrowserSigner;

#[wasm_bindgen(js_name = NostrSigner)]
pub struct JsNostrSigner {
    inner: Arc<dyn NostrSigner>,
}

impl Deref for JsNostrSigner {
    type Target = Arc<dyn NostrSigner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Arc<dyn NostrSigner>> for JsNostrSigner {
    fn from(inner: Arc<dyn NostrSigner>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NostrSigner)]
impl JsNostrSigner {
    /// Private keys
    pub fn keys(keys: &JsKeys) -> Self {
        Self {
            inner: keys.deref().clone().into_nostr_signer(),
        }
    }

    /// NIP07
    pub fn nip07(signer: &JsBrowserSigner) -> Self {
        Self {
            inner: signer.deref().clone().into_nostr_signer(),
        }
    }

    /// NIP46
    pub fn nip46(signer: &JsNostrConnect) -> Self {
        Self {
            inner: signer.deref().clone().into_nostr_signer(),
        }
    }

    /// Get signer public key
    #[wasm_bindgen(js_name = publicKey)]
    pub async fn get_public_key(&self) -> Result<JsPublicKey> {
        Ok(self.inner.get_public_key().await.map_err(into_err)?.into())
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
    pub async fn nip04_encrypt(&self, public_key: &JsPublicKey, content: &str) -> Result<String> {
        self.inner
            .nip04_encrypt(public_key.deref(), content)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip04Decrypt)]
    pub async fn nip04_decrypt(
        &self,
        public_key: &JsPublicKey,
        encrypted_content: &str,
    ) -> Result<String> {
        self.inner
            .nip04_decrypt(public_key.deref(), encrypted_content)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip44Encrypt)]
    pub async fn nip44_encrypt(&self, public_key: &JsPublicKey, content: &str) -> Result<String> {
        self.inner
            .nip44_encrypt(public_key.deref(), content)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip44Decrypt)]
    pub async fn nip44_decrypt(&self, public_key: &JsPublicKey, content: &str) -> Result<String> {
        self.inner
            .nip44_decrypt(public_key.deref(), content)
            .await
            .map_err(into_err)
    }
}
