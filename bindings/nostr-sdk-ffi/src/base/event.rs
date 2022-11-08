// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use bitcoin_hashes::sha256;
use nostr_sdk_base::event::Kind as KindSdk;
use nostr_sdk_base::{Contact as ContactSdk, Event as EventSdk, KindBase};
use secp256k1::XOnlyPublicKey;
use url::Url;

use crate::base::key::Keys;

pub struct Event {
    event: EventSdk,
}

impl From<EventSdk> for Event {
    fn from(event: EventSdk) -> Self {
        Self { event }
    }
}

impl Deref for Event {
    type Target = EventSdk;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl Event {
    pub fn pubkey(&self) -> String {
        self.event.pubkey.to_string()
    }

    pub fn kind(&self) -> Kind {
        self.event.kind.into()
    }

    pub fn content(&self) -> String {
        self.event.content.clone()
    }
}

impl Event {
    /// Create a generic type of event
    /* pub fn new_generic(content: String, keys: Arc<Keys>, tags: Vec<String>, kind: Kind) -> Result<Self> {
        Ok(Self {
            event: EventSdk::new_generic(&content, keys.deref(), tags, kind.into())?,
        })
    } */

    pub fn set_metadata(
        keys: Arc<Keys>,
        username: String,
        display_name: String,
        about: Option<String>,
        picture: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            event: EventSdk::set_metadata(
                keys.deref(),
                &username,
                &display_name,
                about.as_deref(),
                picture.as_deref(),
            )?,
        })
    }

    pub fn add_recommended_relay(keys: Arc<Keys>, url: String) -> Result<Self> {
        let url = Url::from_str(&url)?;

        Ok(Self {
            event: EventSdk::add_recommended_relay(keys.deref(), &url)?,
        })
    }

    /* /// Create a new TextNote Event
    pub fn new_textnote(content: String, keys: Arc<Keys>, tags: Vec<Tag>) -> Result<Self> {
        Ok(Self {
            event: EventSdk::new_textnote(&content, keys.deref(), &tags)?,
        })
    } */

    pub fn set_contact_list(keys: Arc<Keys>, list: Vec<Arc<Contact>>) -> Result<Self> {
        let list: Vec<ContactSdk> = list
            .into_iter()
            .map(|c| c.as_ref().deref().clone())
            .collect();

        Ok(Self {
            event: EventSdk::set_contact_list(keys.deref(), list)?,
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

    /// Add reaction (like/upvote, dislike/downvote) to an event
    pub fn new_reaction(keys: Arc<Keys>, event_id: String, positive: bool) -> Result<Self> {
        let event_id = sha256::Hash::from_str(&event_id)?;

        Ok(Self {
            event: EventSdk::new_reaction(keys.deref(), event_id, positive)?,
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

    pub fn as_json(&self) -> Result<String> {
        self.event.as_json()
    }
}

pub enum Kind {
    Base { kind: KindBase },
    Custom { kind: u16 },
}

impl From<KindSdk> for Kind {
    fn from(kind: KindSdk) -> Self {
        match kind {
            KindSdk::Base(kind) => Self::Base { kind },
            KindSdk::Custom(kind) => Self::Custom { kind },
        }
    }
}

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
