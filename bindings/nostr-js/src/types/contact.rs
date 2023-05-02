// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::key::JsPublicKey;

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

impl From<&JsContact> for Contact {
    fn from(contact: &JsContact) -> Self {
        contact.inner.clone()
    }
}

#[wasm_bindgen(js_class = Contact)]
impl JsContact {
    #[wasm_bindgen(constructor)]
    pub fn new(public_key: &JsPublicKey, relay_url: Option<String>, alias: Option<String>) -> Self {
        let relay_url: Option<UncheckedUrl> = match relay_url {
            Some(relay_url) => Some(UncheckedUrl::from(&relay_url)),
            None => None,
        };
        Self {
            inner: Contact::new(public_key.into(), relay_url, alias),
        }
    }
}

impl JsContact {
    pub fn inner(&self) -> Contact {
        self.inner.clone()
    }
}
