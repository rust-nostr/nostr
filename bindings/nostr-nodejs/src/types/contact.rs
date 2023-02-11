// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::prelude::*;

use crate::key::JsPublicKey;

#[napi(js_name = "Contact")]
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

#[napi]
impl JsContact {
    #[napi(constructor)]
    pub fn new(public_key: &JsPublicKey, relay_url: Option<String>, alias: Option<String>) -> Self {
        Self {
            inner: Contact::new(public_key.into(), relay_url, alias),
        }
    }
}
