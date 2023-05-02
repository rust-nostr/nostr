// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::event::tag::UncheckedUrl;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::Contact as ContactSdk;

use crate::error::Result;

pub struct Contact {
    contact: ContactSdk,
}

impl Deref for Contact {
    type Target = ContactSdk;
    fn deref(&self) -> &Self::Target {
        &self.contact
    }
}

impl Contact {
    pub fn new(pk: String, relay_url: Option<String>, alias: Option<String>) -> Result<Self> {
        let pk = XOnlyPublicKey::from_str(&pk)?;
        let relay_url = relay_url.map(|relay_url| UncheckedUrl::from(&relay_url));
        Ok(Self {
            contact: ContactSdk::new(pk, relay_url, alias),
        })
    }

    pub fn alias(&self) -> Option<String> {
        self.contact.alias.clone()
    }

    pub fn public_key(&self) -> String {
        self.contact.pk.to_string()
    }

    pub fn relay_url(&self) -> Option<String> {
        self.contact.relay_url.clone().map(|u| u.to_string())
    }
}
