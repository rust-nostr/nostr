// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

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
    pub fn new(alias: String, pk: String, relay_url: String) -> Result<Self> {
        let pk = XOnlyPublicKey::from_str(&pk)?;

        Ok(Self {
            contact: ContactSdk::new(pk, &relay_url, &alias),
        })
    }

    pub fn alias(&self) -> String {
        self.contact.alias.clone()
    }

    pub fn public_key(&self) -> String {
        self.contact.pk.to_string()
    }

    pub fn relay_url(&self) -> String {
        self.contact.relay_url.clone()
    }
}
