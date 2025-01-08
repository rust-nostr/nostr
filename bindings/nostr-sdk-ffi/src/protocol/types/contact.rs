// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use nostr::RelayUrl;
use uniffi::Record;

use crate::error::{NostrSdkError, Result};
use crate::protocol::key::PublicKey;

#[derive(Record)]
pub struct Contact {
    pub public_key: Arc<PublicKey>,
    pub relay_url: Option<String>,
    pub alias: Option<String>,
}

impl TryFrom<Contact> for nostr::Contact {
    type Error = NostrSdkError;

    fn try_from(contact: Contact) -> Result<Self, Self::Error> {
        let relay_url = match contact.relay_url {
            Some(url) => Some(RelayUrl::parse(&url)?),
            None => None,
        };
        Ok(nostr::Contact::new(
            **contact.public_key,
            relay_url,
            contact.alias,
        ))
    }
}
