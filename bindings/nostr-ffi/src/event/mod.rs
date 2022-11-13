// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use bitcoin_hashes::sha256;
use nostr::{Contact as ContactSdk, Event as EventSdk};
use url::Url;

pub mod kind;

use self::kind::Kind;
use crate::contact::Contact;
use crate::key::Keys;

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
        username: Option<String>,
        display_name: Option<String>,
        about: Option<String>,
        picture: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            event: EventSdk::set_metadata(
                keys.deref(),
                username.as_deref(),
                display_name.as_deref(),
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
    pub fn new_text_note(content: String, keys: Arc<Keys>, tags: Vec<Tag>) -> Result<Self> {
        Ok(Self {
            event: EventSdk::new_text_note(&content, keys.deref(), &tags)?,
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
    pub fn new_reaction(keys: Arc<Keys>, event: Arc<Event>, positive: bool) -> Result<Self> {
        Ok(Self {
            event: EventSdk::new_reaction(keys.deref(), event.deref(), positive)?,
        })
    }

    pub fn verify(&self) -> bool {
        self.event.verify().is_ok()
    }

    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            event: EventSdk::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> Result<String> {
        self.event.as_json()
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
