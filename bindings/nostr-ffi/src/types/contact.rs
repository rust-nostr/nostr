// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::UncheckedUrl;
use uniffi::Object;

use crate::PublicKey;

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Contact {
    inner: nostr::Contact,
}

impl Deref for Contact {
    type Target = nostr::Contact;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Contact {
    #[uniffi::constructor]
    pub fn new(pk: &PublicKey, relay_url: Option<String>, alias: Option<String>) -> Self {
        let relay_url = relay_url.map(|relay_url| UncheckedUrl::from(&relay_url));
        Self {
            inner: nostr::Contact::new(**pk, relay_url, alias),
        }
    }

    pub fn alias(&self) -> Option<String> {
        self.inner.alias.clone()
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.public_key.into())
    }

    pub fn relay_url(&self) -> Option<String> {
        self.inner.relay_url.clone().map(|u| u.to_string())
    }
}
