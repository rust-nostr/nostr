// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_js::JsKeys;
use nostr_sdk::ClientSigner;
use wasm_bindgen::prelude::*;

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
    /// Compose client signer using `Keys`
    pub fn keys(keys: &JsKeys) -> Self {
        Self {
            inner: ClientSigner::Keys(keys.deref().clone()),
        }
    }

    // TODO
}
