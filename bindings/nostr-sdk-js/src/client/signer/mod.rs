// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_js::key::JsKeys;
use nostr_js::nips::nip07::JsNip07Signer;
use nostr_sdk::ClientSigner;
use wasm_bindgen::prelude::*;

pub mod nip46;

use self::nip46::JsNip46Signer;

#[wasm_bindgen(js_name = ClientSigner)]
pub struct JsClientSigner {
    inner: nostr_sdk::ClientSigner,
}

impl Deref for JsClientSigner {
    type Target = ClientSigner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<ClientSigner> for JsClientSigner {
    fn from(inner: ClientSigner) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = ClientSigner)]
impl JsClientSigner {
    /// Private Key Client Signer
    pub fn keys(keys: &JsKeys) -> Self {
        Self {
            inner: ClientSigner::Keys(keys.deref().clone()),
        }
    }

    /// NIP07 Client Signer
    pub fn nip07(signer: &JsNip07Signer) -> Self {
        Self {
            inner: ClientSigner::NIP07(signer.deref().clone()),
        }
    }

    /// NIP46 Client Signer
    pub fn nip46(signer: &JsNip46Signer) -> Self {
        Self {
            inner: ClientSigner::NIP46(Box::new(signer.deref().clone())),
        }
    }
}
