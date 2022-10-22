// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use nostr_sdk_base::{Event as EventSdk, KindBase};

use crate::base::key::Keys;

pub struct Event {
    event: EventSdk,
}

impl Deref for Event {
    type Target = EventSdk;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl Event {
    /* /// Create a generic type of event
    pub fn new_generic(content: String, keys: Keys, tags: Vec<Tag>, kind: Kind) -> Result<Self> {
        Ok(Self {
            event: EventSdk::new_generic(&content, keys.deref(), tags, *kind.deref())?,
        })
    }

    /// Create a new TextNote Event
    pub fn new_textnote(content: String, keys: Keys, tags: Vec<Tag>) -> Result<Self> {
        Ok(Self {
            event: EventSdk::new_textnote(&content, keys.deref(), &tags)?,
        })
    } */

    /* pub fn backup_contacts(keys: &Keys, list: Vec<Contact>) -> Result<Self> {
        let tags: Vec<Tag> = list
            .iter()
            .map(|contact| {
                Tag::new(TagData::ContactList {
                    pk: contact.pk,
                    relay_url: contact.relay_url.clone(),
                    alias: contact.alias.clone(),
                })
            })
            .collect();

        Self::new_generic("", keys, &tags, Kind::Base(KindBase::ContactList))
    } */

    /// Create encrypted direct msg event
    pub fn new_encrypted_direct_msg(
        sender_keys: Arc<Keys>,
        receiver_keys: Arc<Keys>,
        content: String,
    ) -> Result<Self> {
        Ok(Self {
            event: EventSdk::new_encrypted_direct_msg(
                sender_keys.deref(),
                receiver_keys.deref(),
                &content,
            )?,
        })
    }

    /* /// Create delete event
    pub fn delete(keys: &Keys, ids: Vec<sha256::Hash>, content: &str) -> Result<Self> {
        let tags: Vec<Tag> = ids
            .iter()
            .map(|id| Tag::new(TagData::EventId(id.to_string())))
            .collect();

        Self::new_generic(content, keys, &tags, Kind::Base(KindBase::EventDeletion))
    }

    pub fn verify(&self) -> Result<(), secp256k1::Error> {
        let secp = Secp256k1::new();
        let id = Self::gen_id(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content,
        );
        let message = secp256k1::Message::from_slice(&id)?;
        secp.verify_schnorr(&self.sig, &message, &self.pubkey)
    } */

    pub fn new_from_json(json: String) -> Result<Self> {
        Ok(Self {
            event: EventSdk::new_from_json(json)?,
        })
    }

    pub fn as_json(&self) -> String {
        self.event.as_json()
    }
}

pub enum Kind {
    Base { kind: KindBase },
    Custom { kind: u16 },
}

/* impl Deref for Kind {
    type Target = nostr::Kind;
    fn deref(&self) -> &Self::Target {
        match &self {
            Self::Base { kind } => &nostr::Kind::Base(*kind),
            Self::Custom { kind } => &nostr::Kind::Custom(*kind),
        }
    }
} */
