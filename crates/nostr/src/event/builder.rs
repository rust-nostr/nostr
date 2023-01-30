// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event builder

use bitcoin::secp256k1::{KeyPair, Message, Secp256k1, XOnlyPublicKey};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use url::Url;

pub use super::kind::Kind;
pub use super::tag::{Marker, Tag, TagKind};
use super::{Event, EventId};
use crate::key::{self, Keys};
#[cfg(feature = "nip04")]
use crate::nips::nip04;
#[cfg(feature = "nip13")]
use crate::nips::nip13;
use crate::types::{Contact, Metadata, Timestamp};

static REGEX_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^[a-zA-Z0-9][a-zA-Z_\-0-9]+[a-zA-Z0-9]$"#).expect("Invalid regex"));

/// [`EventBuilder`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Key error
    #[error(transparent)]
    Key(#[from] key::Error),
    #[error(transparent)]
    /// Secp256k1 error
    Secp256k1(#[from] bitcoin::secp256k1::Error),
    /// JSON error
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Invalid metadata name
    #[error("invalid name")]
    InvalidName,
    /// NIP04 error
    #[cfg(feature = "nip04")]
    #[error(transparent)]
    NIP04(#[from] nip04::Error),
}

/// [`Event`] builder
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventBuilder {
    kind: Kind,
    tags: Vec<Tag>,
    content: String,
}

impl EventBuilder {
    /// New [`EventBuilder`]
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

    /// Build [`Event`]
    pub fn to_event(self, keys: &Keys) -> Result<Event, Error> {
        let secp = Secp256k1::new();
        let keypair: &KeyPair = &keys.key_pair()?;
        let pubkey: XOnlyPublicKey = keys.public_key();
        let created_at: Timestamp = Timestamp::now();

        let id = EventId::new(&pubkey, created_at, &self.kind, &self.tags, &self.content);
        let message = Message::from_slice(id.as_bytes())?;

        Ok(Event {
            id,
            pubkey,
            created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig: secp.sign_schnorr(&message, keypair),
            ots: None,
        })
    }

    /// Build POW [`Event`]
    #[cfg(feature = "nip13")]
    pub fn to_pow_event(self, keys: &Keys, difficulty: u8) -> Result<Event, Error> {
        #[cfg(target_arch = "wasm32")]
        use instant::Instant;
        #[cfg(not(target_arch = "wasm32"))]
        use std::time::Instant;

        let mut nonce: u128 = 0;
        let mut tags: Vec<Tag> = self.tags;

        let pubkey = keys.public_key();

        let now = Instant::now();

        loop {
            nonce += 1;

            tags.push(Tag::POW { nonce, difficulty });

            let created_at: Timestamp = Timestamp::now();
            let id = EventId::new(&pubkey, created_at, &self.kind, &tags, &self.content);

            if nip13::get_leading_zero_bits(id.inner()) >= difficulty {
                log::debug!(
                    "{} iterations in {} ms. Avg rate {} hashes/second",
                    nonce,
                    now.elapsed().as_millis(),
                    nonce * 1000 / std::cmp::max(1, now.elapsed().as_millis())
                );

                let secp = Secp256k1::new();
                let keypair: &KeyPair = &keys.key_pair()?;
                let message = Message::from_slice(id.as_bytes())?;

                return Ok(Event {
                    id,
                    pubkey,
                    created_at,
                    kind: self.kind,
                    tags,
                    content: self.content,
                    sig: secp.sign_schnorr(&message, keypair),
                    ots: None,
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
    /// use nostr::url::Url;
    /// use nostr::{EventBuilder, Metadata};
    ///
    /// let metadata = Metadata::new()
    ///     .name("username")
    ///     .display_name("My Username")
    ///     .about("Description")
    ///     .picture(Url::parse("https://example.com/avatar.png").unwrap())
    ///     .nip05("username@example.com")
    ///     .lud16("yuki@stacker.news");
    ///
    /// let builder = EventBuilder::set_metadata(metadata).unwrap();
    /// ```
    pub fn set_metadata(metadata: Metadata) -> Result<Self, Error> {
        if let Some(name) = metadata.name.clone() {
            if !REGEX_NAME.is_match(&name) {
                return Err(Error::InvalidName);
            }
        }

        Ok(Self::new(
            Kind::Metadata,
            serde_json::to_string(&metadata)?,
            &[],
        ))
    }

    /// Add recommended relay
    pub fn add_recommended_relay(url: &Url) -> Self {
        Self::new(Kind::RecommendRelay, url.as_ref(), &[])
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
        Self::new(Kind::TextNote, content, tags)
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

        Self::new(Kind::ContactList, "", &tags)
    }

    /// Create encrypted direct msg event
    #[cfg(feature = "nip04")]
    pub fn new_encrypted_direct_msg<S>(
        sender_keys: &Keys,
        receiver_pubkey: XOnlyPublicKey,
        content: S,
    ) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let msg = nip04::encrypt(&sender_keys.secret_key()?, &receiver_pubkey, content.into())?;

        Ok(Self::new(
            Kind::EncryptedDirectMessage,
            msg,
            &[Tag::PubKey(receiver_pubkey, None)],
        ))
    }

    /// Repost event
    pub fn repost(event_id: EventId, public_key: XOnlyPublicKey) -> Self {
        Self::new(
            Kind::Repost,
            String::new(),
            &[
                Tag::Event(event_id, None, None),
                Tag::PubKey(public_key, None),
            ],
        )
    }

    /// Create delete event
    pub fn delete<S>(ids: Vec<EventId>, reason: Option<S>) -> Self
    where
        S: Into<String>,
    {
        let tags: Vec<Tag> = ids.iter().map(|id| Tag::Event(*id, None, None)).collect();

        Self::new(
            Kind::EventDeletion,
            reason.map(|s| s.into()).unwrap_or_default(),
            &tags,
        )
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    pub fn new_reaction<S>(event_id: EventId, public_key: XOnlyPublicKey, content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::Reaction,
            content,
            &[
                Tag::Event(event_id, None, None),
                Tag::PubKey(public_key, None),
            ],
        )
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

        Ok(Self::new(Kind::ChannelCreation, metadata.to_string(), &[]))
    }

    /// Set channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn set_channel_metadata(
        channel_id: EventId, // event id of kind 40
        relay_url: Option<Url>,
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
            Kind::ChannelMetadata,
            metadata.to_string(),
            &[Tag::Event(
                channel_id,
                relay_url.map(|u| u.to_string()),
                None,
            )],
        ))
    }

    /// New channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn new_channel_msg<S>(
        channel_id: EventId, // event id of kind 40
        relay_url: Option<Url>,
        content: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::ChannelMessage,
            content,
            &[Tag::Event(
                channel_id,
                relay_url.map(|u| u.to_string()),
                Some(Marker::Root),
            )],
        )
    }

    /// Hide message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn hide_channel_msg<S>(
        message_id: EventId, // event id of kind 42
        reason: Option<S>,
    ) -> Self
    where
        S: Into<String>,
    {
        let content: Value = json!({
            "reason": reason.map(|s| s.into()).unwrap_or_default(),
        });

        Self::new(
            Kind::ChannelHideMessage,
            content.to_string(),
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
            Kind::ChannelMuteUser,
            content.to_string(),
            &[Tag::PubKey(pubkey, None)],
        )
    }

    /// Create an auth event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    pub fn auth<S>(challenge: S, relay: Url) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::Authentication,
            "",
            &[Tag::Challenge(challenge.into()), Tag::Relay(relay)],
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::secp256k1::SecretKey;

    use crate::{Event, EventBuilder, Keys, Result};

    #[test]
    fn round_trip() -> Result<()> {
        let keys = Keys::new(SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);

        let event = EventBuilder::new_text_note("hello", &vec![]).to_event(&keys)?;

        let serialized = event.as_json().unwrap();
        let deserialized = Event::from_json(serialized)?;

        assert_eq!(event, deserialized);

        Ok(())
    }

    #[test]
    #[cfg(feature = "nip04")]
    fn test_encrypted_direct_msg() -> Result<()> {
        let sender_keys = Keys::new(SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);
        let receiver_keys = Keys::new(SecretKey::from_str(
            "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);

        let content = "Mercury, the Winged Messenger";
        let event = EventBuilder::new_encrypted_direct_msg(
            &sender_keys,
            receiver_keys.public_key(),
            content,
        )?
        .to_event(&sender_keys)?;

        Ok(event.verify()?)
    }
}
