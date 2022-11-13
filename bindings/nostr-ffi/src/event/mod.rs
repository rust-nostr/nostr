// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use bitcoin_hashes::sha256;
use nostr::{Contact as ContactSdk, Event as EventSdk, Tag};
use url::Url;

pub mod kind;

use self::kind::Kind;
use crate::contact::Contact;
use crate::key::Keys;
use crate::metadata::Metadata;

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
    pub fn new_generic(
        keys: Arc<Keys>,
        kind: Kind,
        content: String,
        tags: Vec<Vec<String>>,
    ) -> Result<Self> {
        let tags: Vec<Tag> = tags.into_iter().map(|tag| tag.into()).collect();
        Ok(Self {
            event: EventSdk::new_generic(keys.deref(), kind.into(), &content, &tags)?,
        })
    }

    pub fn set_metadata(keys: Arc<Keys>, metadata: Arc<Metadata>) -> Result<Self> {
        Ok(Self {
            event: EventSdk::set_metadata(keys.deref(), metadata.as_ref().deref().clone())?,
        })
    }

    pub fn add_recommended_relay(keys: Arc<Keys>, url: String) -> Result<Self> {
        let url = Url::from_str(&url)?;

        Ok(Self {
            event: EventSdk::add_recommended_relay(keys.deref(), &url)?,
        })
    }

    /// Create a new TextNote Event
    pub fn new_text_note(keys: Arc<Keys>, content: String, tags: Vec<Vec<String>>) -> Result<Self> {
        let tags: Vec<Tag> = tags.into_iter().map(|tag| tag.into()).collect();
        Ok(Self {
            event: EventSdk::new_text_note(keys.deref(), &content, &tags)?,
        })
    }

    /// Create a new POW TextNote Event
    pub fn new_pow_text_note(
        keys: Arc<Keys>,
        content: String,
        tags: Vec<Vec<String>>,
        difficulty: u8,
    ) -> Result<Self> {
        let tags: Vec<Tag> = tags.into_iter().map(|tag| tag.into()).collect();
        Ok(Self {
            event: EventSdk::new_pow_text_note(keys.deref(), &content, &tags, difficulty)?,
        })
    }

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
