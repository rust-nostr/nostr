// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_js::error::{into_err, Result};
use nostr_js::event::{JsEvent, JsUnsignedEvent};
use nostr_js::key::{JsKeys, JsPublicKey};
use nostr_js::nips::nip07::JsNip07Signer;
use nostr_js::nips::nip44::JsNIP44Version;
use nostr_sdk::NostrSigner;
use wasm_bindgen::prelude::*;

pub mod nip46;

use self::nip46::JsNip46Signer;

#[wasm_bindgen(js_name = NostrSigner)]
pub struct JsNostrSigner {
    inner: nostr_sdk::NostrSigner,
}

impl Deref for JsNostrSigner {
    type Target = NostrSigner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NostrSigner> for JsNostrSigner {
    fn from(inner: NostrSigner) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NostrSigner)]
impl JsNostrSigner {
    /// Private keys
    pub fn keys(keys: &JsKeys) -> Self {
        Self {
            inner: NostrSigner::Keys(keys.deref().clone()),
        }
    }

    /// NIP07
    pub fn nip07(signer: &JsNip07Signer) -> Self {
        Self {
            inner: NostrSigner::NIP07(signer.deref().clone()),
        }
    }

    /// NIP46
    pub fn nip46(signer: &JsNip46Signer) -> Self {
        Self {
            inner: NostrSigner::nip46(signer.deref().clone()),
        }
    }

    /// Get signer public key
    #[wasm_bindgen(js_name = publicKey)]
    pub async fn public_key(&self) -> Result<JsPublicKey> {
        Ok(self.inner.public_key().await.map_err(into_err)?.into())
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
    pub async fn nip04_encrypt(&self, public_key: &JsPublicKey, content: String) -> Result<String> {
        self.inner
            .nip04_encrypt(**public_key, content)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip04Decrypt)]
    pub async fn nip04_decrypt(
        &self,
        public_key: &JsPublicKey,
        encrypted_content: String,
    ) -> Result<String> {
        self.inner
            .nip04_decrypt(**public_key, encrypted_content)
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip44Encrypt)]
    pub async fn nip44_encrypt(
        &self,
        public_key: &JsPublicKey,
        content: String,
        version: JsNIP44Version,
    ) -> Result<String> {
        self.inner
            .nip44_encrypt(**public_key, content, version.into())
            .await
            .map_err(into_err)
    }

    #[wasm_bindgen(js_name = nip44Decrypt)]
    pub async fn nip44_decrypt(&self, public_key: &JsPublicKey, content: String) -> Result<String> {
        self.inner
            .nip44_decrypt(**public_key, content)
            .await
            .map_err(into_err)
    }
}
