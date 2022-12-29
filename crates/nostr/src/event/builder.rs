// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Instant;

use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{KeyPair, Message, Secp256k1, XOnlyPublicKey};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use url::Url;

pub use super::kind::{Kind, KindBase};
pub use super::tag::{Marker, Tag, TagKind};
use super::Event;
use crate::key::{self, Keys};
use crate::metadata::Metadata;
use crate::util::nips;
use crate::util::time::timestamp;
use crate::{Contact, Sha256Hash};

static REGEX_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^[a-zA-Z0-9][a-zA-Z_\-0-9]+[a-zA-Z0-9]$"#).expect("Invalid regex"));

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Key error
    #[error("key error: {0}")]
    Key(#[from] key::Error),
    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] bitcoin::secp256k1::Error),
    /// Invalid metadata name
    #[error("invalid name")]
    InvalidName,
    /// NIP04 error
    #[cfg(feature = "nip04")]
    #[error("nip04 error: {0}")]
    NIP04(#[from] nips::nip04::Error),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventBuilder {
    kind: Kind,
    tags: Vec<Tag>,
    content: String,
}

impl EventBuilder {
    pub fn new<S>(kind: Kind, content: S, tags: &[Tag]) -> Self
    where
        S: Into<String>,
    {
        Self {
            kind,
            tags: tags.to_vec(),
            content: content.into(),
        }
    }

    pub fn gen_id(
        pubkey: &XOnlyPublicKey,
        created_at: u64,
        kind: &Kind,
        tags: &[Tag],
        content: &str,
    ) -> Sha256Hash {
        let json: Value = json!([0, pubkey, created_at, kind, tags, content]);
        let event_str: String = json.to_string();
        Sha256Hash::hash(event_str.as_bytes())
    }

    /// Build `Event`
    pub fn to_event(self, keys: &Keys) -> Result<Event, Error> {
        let secp = Secp256k1::new();
        let keypair: &KeyPair = &keys.key_pair()?;
        let pubkey: XOnlyPublicKey = keys.public_key();
        let created_at: u64 = timestamp();

        let id: Sha256Hash =
            Self::gen_id(&pubkey, created_at, &self.kind, &self.tags, &self.content);
        let message = Message::from_slice(&id)?;

        Ok(Event {
            id,
            pubkey,
            created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig: secp.sign_schnorr(&message, keypair),
        })
    }

    /// Build POW `Event`
    pub fn to_pow_event(self, keys: &Keys, difficulty: u8) -> Result<Event, Error> {
        let mut nonce: u128 = 0;
        let mut tags: Vec<Tag> = self.tags;

        let pubkey = keys.public_key();

        let now = Instant::now();

        loop {
            nonce += 1;

            tags.push(Tag::POW { nonce, difficulty });

            let created_at: u64 = timestamp();
            let id: Sha256Hash =
                Self::gen_id(&pubkey, created_at, &self.kind, &tags, &self.content);

            if nips::nip13::get_leading_zero_bits(id) >= difficulty {
                log::debug!(
                    "{} iterations in {} ms. Avg rate {} hashes/second",
                    nonce,
                    now.elapsed().as_millis(),
                    nonce * 1000 / std::cmp::max(1, now.elapsed().as_millis())
                );

                let secp = Secp256k1::new();
                let keypair: &KeyPair = &keys.key_pair()?;
                let message = Message::from_slice(&id)?;

                return Ok(Event {
                    id,
                    pubkey,
                    created_at,
                    kind: self.kind,
                    tags,
                    content: self.content,
                    sig: secp.sign_schnorr(&message, keypair),
                });
            }

            tags.pop();
        }
    }
}

impl EventBuilder {
    /// Set metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::metadata::Metadata;
    /// use nostr::url::Url;
    /// use nostr::EventBuilder;
    ///
    /// let metadata = Metadata::new()
    ///     .name("username")
    ///     .display_name("My Username")
    ///     .about("Description")
    ///     .picture(Url::parse("https://example.com/avatar.png").unwrap())
    ///     .nip05("username@example.com");
    ///
    /// let builder = EventBuilder::set_metadata(metadata).unwrap();
    /// ```
    pub fn set_metadata(metadata: Metadata) -> Result<Self, Error> {
        let name = metadata.name;
        let display_name = metadata.display_name;
        let about = metadata.about;
        let picture = metadata.picture;
        let nip05_str = metadata.nip05;

        if let Some(name) = name.clone() {
            if !REGEX_NAME.is_match(&name) {
                return Err(Error::InvalidName);
            }
        }

        let mut metadata: Value = json!({
            "name": name.unwrap_or_default(),
            "display_name": display_name.unwrap_or_default(),
            "about": about.unwrap_or_default(),
        });

        if let Some(picture) = picture {
            metadata["picture"] = json!(picture);
        }

        if let Some(nip05_str) = nip05_str {
            metadata["nip05"] = json!(nip05_str);
        }

        Ok(Self::new(
            Kind::Base(KindBase::Metadata),
            metadata.to_string(),
            &[],
        ))
    }

    /// Add recommended relay
    pub fn add_recommended_relay(url: &Url) -> Self {
        Self::new(Kind::Base(KindBase::RecommendRelay), url.as_ref(), &[])
    }

    /// Text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::EventBuilder;
    ///
    /// let builder = EventBuilder::new_text_note("My first text note from Nostr SDK!", &[]);
    /// ```
    pub fn new_text_note<S>(content: S, tags: &[Tag]) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::Base(KindBase::TextNote), content, tags)
    }

    /// Set contact list
    pub fn set_contact_list(list: Vec<Contact>) -> Self {
        let tags: Vec<Tag> = list
            .iter()
            .map(|contact| Tag::ContactList {
                pk: contact.pk,
                relay_url: contact.relay_url.clone(),
                alias: contact.alias.clone(),
            })
            .collect();

        Self::new(Kind::Base(KindBase::ContactList), "", &tags)
    }

    /// Create encrypted direct msg event
    #[cfg(feature = "nip04")]
    pub fn new_encrypted_direct_msg<S>(
        sender_keys: &Keys,
        receiver_keys: &Keys,
        content: S,
    ) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let msg = nips::nip04::encrypt(
            &sender_keys.secret_key()?,
            &receiver_keys.public_key(),
            content.into(),
        )?;

        Ok(Self::new(
            Kind::Base(KindBase::EncryptedDirectMessage),
            &msg,
            &[Tag::PubKey(receiver_keys.public_key(), None)],
        ))
    }

    /// Create delete event
    pub fn delete<S>(ids: Vec<Sha256Hash>, reason: Option<S>) -> Self
    where
        S: Into<String>,
    {
        let tags: Vec<Tag> = ids.iter().map(|id| Tag::Event(*id, None, None)).collect();

        Self::new(
            Kind::Base(KindBase::EventDeletion),
            reason.map(|s| s.into()).unwrap_or_default(),
            &tags,
        )
    }

    /// Add reaction (like/upvote, dislike/downvote) to an event
    pub fn new_reaction(event: &Event, positive: bool) -> Self {
        let tags: &[Tag] = &[
            Tag::Event(event.id, None, None),
            Tag::PubKey(event.pubkey, None),
        ];

        let content: &str = match positive {
            true => "+",
            false => "-",
        };

        Self::new(Kind::Base(KindBase::Reaction), content, tags)
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn new_channel(metadata: Metadata) -> Result<Self, Error> {
        let name = metadata.name;
        let display_name = metadata.display_name;
        let about = metadata.about;
        let picture = metadata.picture;

        if let Some(name) = name.as_ref() {
            if !REGEX_NAME.is_match(name) {
                return Err(Error::InvalidName);
            }
        }

        let mut metadata: Value = json!({
            "name": name.unwrap_or_default(),
            "display_name": display_name.unwrap_or_default(),
            "about": about.unwrap_or_default(),
        });

        if let Some(picture) = picture {
            metadata["picture"] = json!(picture);
        }

        Ok(Self::new(
            Kind::Base(KindBase::ChannelCreation),
            metadata.to_string(),
            &[],
        ))
    }

    /// Set channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn set_channel_metadata(
        channel_id: Sha256Hash, // event id of kind 40
        relay_url: Url,
        metadata: Metadata,
    ) -> Result<Self, Error> {
        let name = metadata.name;
        let display_name = metadata.display_name;
        let about = metadata.about;
        let picture = metadata.picture;

        if let Some(name) = name.as_ref() {
            if !REGEX_NAME.is_match(name) {
                return Err(Error::InvalidName);
            }
        }

        let mut metadata: Value = json!({
            "name": name.unwrap_or_default(),
            "display_name": display_name.unwrap_or_default(),
            "about": about.unwrap_or_default(),
        });

        if let Some(picture) = picture {
            metadata["picture"] = json!(picture);
        }

        Ok(Self::new(
            Kind::Base(KindBase::ChannelMetadata),
            metadata.to_string(),
            &[Tag::Event(channel_id, Some(relay_url), None)],
        ))
    }

    /// New channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn new_channel_msg<S>(
        channel_id: Sha256Hash, // event id of kind 40
        relay_url: Url,
        content: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::Base(KindBase::ChannelMessage),
            content,
            &[Tag::Event(channel_id, Some(relay_url), Some(Marker::Root))],
        )
    }

    /// Hide message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn hide_channel_msg<S>(
        message_id: Sha256Hash, // event id of kind 42
        reason: Option<S>,
    ) -> Self
    where
        S: Into<String>,
    {
        let content: Value = json!({
            "reason": reason.map(|s| s.into()).unwrap_or_default(),
        });

        Self::new(
            Kind::Base(KindBase::ChannelHideMessage),
            &content.to_string(),
            &[Tag::Event(message_id, None, None)],
        )
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn mute_channel_user<S>(pubkey: XOnlyPublicKey, reason: Option<S>) -> Self
    where
        S: Into<String>,
    {
        let content: Value = json!({
            "reason": reason.map(|s| s.into()).unwrap_or_default(),
        });

        Self::new(
            Kind::Base(KindBase::ChannelMuteUser),
            &content.to_string(),
            &[Tag::PubKey(pubkey, None)],
        )
    }
}
