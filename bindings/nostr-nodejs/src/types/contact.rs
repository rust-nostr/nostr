// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use napi::Result;
use nostr::prelude::*;

use crate::error::into_err;

#[napi(js_name = "Contact")]
pub struct JsContact {
    contact: Contact,
}

impl Deref for JsContact {
    type Target = Contact;
    fn deref(&self) -> &Self::Target {
        &self.contact
    }
}

#[napi]
impl JsContact {
    #[napi(constructor)]
    pub fn new(
        public_key: String,
        relay_url: Option<String>,
        alias: Option<String>,
    ) -> Result<Self> {
        let pk = XOnlyPublicKey::from_str(&public_key).map_err(into_err)?;
        Ok(Self {
            contact: Contact::new(pk, relay_url, alias),
        })
    }
}
