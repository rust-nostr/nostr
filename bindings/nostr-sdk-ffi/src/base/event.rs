// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use bitcoin_hashes::sha256;
//use nostr_sdk_base::event::TagKind;
use nostr_sdk_base::{Contact as ContactSdk, Event as EventSdk, KindBase};
use secp256k1::XOnlyPublicKey;

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

    pub fn backup_contacts(keys: Arc<Keys>, list: Vec<Arc<Contact>>) -> Result<Self> {
        let list: Vec<ContactSdk> = list
            .into_iter()
            .map(|c| c.as_ref().deref().clone())
            .collect();

        Ok(Self {
            event: EventSdk::backup_contacts(keys.deref(), list)?,
        })
    }

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

    /// Create delete event
    pub fn delete(keys: Arc<Keys>, ids: Vec<String>, reason: Option<String>) -> Result<Self> {
        let mut new_ids: Vec<sha256::Hash> = Vec::with_capacity(ids.len());

        for id in ids.into_iter() {
            new_ids.push(sha256::Hash::from_str(&id)?);
        }

        Ok(Self {
            event: EventSdk::delete(keys.deref(), new_ids, reason.as_deref())?,
        })
    }

    pub fn verify(&self) -> bool {
        self.event.verify().is_ok()
    }

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

/* pub enum TagData {
    Generic(TagKind, Vec<String>),
    EventId(String),
    ContactList {
        pk: XOnlyPublicKey,
        relay_url: String,
        alias: String,
    },
    EncryptedDirectMessage {
        pk: XOnlyPublicKey,
    },
    POW {
        nonce: u128,
        difficulty: u8,
    },
}

impl From<TagData> for Vec<String> {
    fn from(data: TagData) -> Self {
        match data {
            TagData::Generic(kind, data) => vec![vec![kind.to_string()], data].concat(),
            TagData::EventId(id) => vec![TagKind::E.to_string(), id],
            TagData::ContactList {
                pk,
                relay_url,
                alias,
            } => vec![TagKind::P.to_string(), pk.to_string(), relay_url, alias],
            TagData::EncryptedDirectMessage { pk } => vec![TagKind::P.to_string(), pk.to_string()],
            TagData::POW { nonce, difficulty } => vec![
                TagKind::Nonce.to_string(),
                nonce.to_string(),
                difficulty.to_string(),
            ],
        }
    }
}

pub struct Tag(Vec<String>);

impl Tag {
    pub fn new(data: TagData) -> Self {
        Self(data.into())
    }

    pub fn kind(&self) -> &str {
        &self.0[0]
    }

    pub fn content(&self) -> &str {
        &self.0[1]
    }
} */

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
            contact: ContactSdk::new(&alias, pk, &relay_url),
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
