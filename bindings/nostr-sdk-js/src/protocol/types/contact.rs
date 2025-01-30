// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::key::JsPublicKey;

#[wasm_bindgen(js_name = Contact)]
pub struct JsContact {
    inner: Contact,
}

impl Deref for JsContact {
    type Target = Contact;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Contact> for JsContact {
    fn from(contact: Contact) -> Self {
        Self { inner: contact }
    }
}

impl From<JsContact> for Contact {
    fn from(contact: JsContact) -> Self {
        contact.inner
    }
}

#[wasm_bindgen(js_class = Contact)]
impl JsContact {
    #[wasm_bindgen(constructor)]
    pub fn new(
        public_key: &JsPublicKey,
        relay_url: Option<String>,
        alias: Option<String>,
    ) -> Result<JsContact> {
        let relay_url = match relay_url {
            Some(url) => Some(RelayUrl::parse(&url).map_err(into_err)?),
            None => None,
        };
        Ok(Self {
            inner: Contact {
                public_key: **public_key,
                relay_url,
                alias,
            },
        })
    }

    #[wasm_bindgen(getter)]
    pub fn alias(&self) -> Option<String> {
        self.inner.alias.clone()
    }

    #[wasm_bindgen(js_name = publicKey, getter)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    #[wasm_bindgen(js_name = relayUrl, getter)]
    pub fn relay_url(&self) -> Option<String> {
        self.inner.relay_url.as_ref().map(|u| u.to_string())
    }
}
