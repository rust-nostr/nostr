// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Instant;

use anyhow::{anyhow, Result};
use bitcoin_hashes::{sha256, Hash};
use once_cell::sync::Lazy;
use regex::Regex;
use secp256k1::{KeyPair, Secp256k1, XOnlyPublicKey};
use serde_json::{json, Value};
use url::Url;

pub use super::kind::{Kind, KindBase};
pub use super::tag::{Marker, Tag, TagData, TagKind};
use super::Event;
use crate::metadata::Metadata;
use crate::util::nips::{nip04, nip05, nip13};
use crate::util::time::timestamp;
use crate::{Contact, Keys};

static REGEX_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^[a-zA-Z0-9][a-zA-Z_\-0-9]+[a-zA-Z0-9]$"#).expect("Invalid regex"));

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventBuilder {
    kind: Kind,
    tags: Vec<Tag>,
    content: String,
}

impl EventBuilder {
    pub fn new(kind: Kind, content: &str, tags: &[Tag]) -> Self {
        Self {
            kind,
            tags: tags.to_vec(),
            content: content.to_string(),
        }
    }

    pub fn gen_id(
        pubkey: &XOnlyPublicKey,
        created_at: u64,
        kind: &Kind,
        tags: &[Tag],
        content: &str,
    ) -> sha256::Hash {
        let json: Value = json!([0, pubkey, created_at, kind, tags, content]);
        let event_str: String = json.to_string();
        sha256::Hash::hash(event_str.as_bytes())
    }

    /// Build `Event`
    pub fn to_event(self, keys: &Keys) -> Result<Event> {
        let secp = Secp256k1::new();
        let keypair: &KeyPair = &keys.key_pair()?;
        let pubkey: XOnlyPublicKey = keys.public_key();
        let created_at: u64 = timestamp();

        let id: sha256::Hash =
            Self::gen_id(&pubkey, created_at, &self.kind, &self.tags, &self.content);
        let message = secp256k1::Message::from_slice(&id)?;

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
    pub fn to_pow_event(self, keys: &Keys, difficulty: u8) -> Result<Event> {
        let mut nonce: u128 = 0;
        let mut tags: Vec<Tag> = self.tags;

        let pubkey = keys.public_key();

        let now = Instant::now();

        loop {
            nonce += 1;

            tags.push(Tag::new(TagData::POW { nonce, difficulty }));

            let created_at: u64 = timestamp();
            let id: sha256::Hash =
                Self::gen_id(&pubkey, created_at, &self.kind, &tags, &self.content);

            if nip13::get_leading_zero_bits(id) >= difficulty {
                log::debug!(
                    "{} iterations in {} ms. Avg rate {} hashes/second",
                    nonce,
                    now.elapsed().as_millis(),
                    nonce * 1000 / std::cmp::max(1, now.elapsed().as_millis())
                );

                let secp = Secp256k1::new();
                let keypair: &KeyPair = &keys.key_pair()?;
                let message = secp256k1::Message::from_slice(&id)?;

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
    /// use std::str::FromStr;
    ///
    /// use nostr::key::{FromBech32, Keys};
    /// use nostr::metadata::Metadata;
    /// use nostr::EventBuilder;
    /// use url::Url;
    ///
    /// let my_keys = Keys::from_bech32("nsec1...").unwrap();
    ///
    /// let metadata = Metadata::new()
    ///     .name("username")
    ///     .display_name("My Username")
    ///     .about("Description")
    ///     .picture(Url::from_str("https://example.com/avatar.png").unwrap())
    ///     .nip05("username@example.com");
    ///
    /// let builder = EventBuilder::set_metadata(&my_keys, metadata).unwrap();
    /// ```
    pub fn set_metadata(keys: &Keys, metadata: Metadata) -> Result<Self> {
        let name = metadata.name;
        let display_name = metadata.display_name;
        let about = metadata.about;
        let picture = metadata.picture;
        let nip05_str = metadata.nip05;

        if let Some(name) = name.clone() {
            if !REGEX_NAME.is_match(&name) {
                return Err(anyhow!("Invalid name"));
            }
        }

        let mut metadata: Value = json!({
            "name": name.unwrap_or_else(|| "".into()),
            "display_name": display_name.unwrap_or_else(|| "".into()),
            "about": about.unwrap_or_else(|| "".into()),
            "picture": picture.unwrap_or_else(|| "".into()),
        });

        if let Some(nip05_str) = nip05_str {
            if !nip05::verify(keys.public_key(), &nip05_str)? {
                return Err(anyhow!("Impossible to verify NIP-05"));
            }
            metadata["nip05"] = json!(nip05_str);
        }

        Ok(Self::new(
            Kind::Base(KindBase::Metadata),
            &metadata.to_string(),
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
    pub fn new_text_note(content: &str, tags: &[Tag]) -> Self {
        Self::new(Kind::Base(KindBase::TextNote), content, tags)
    }

    /// Set contact list
    pub fn set_contact_list(list: Vec<Contact>) -> Self {
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

        Self::new(Kind::Base(KindBase::ContactList), "", &tags)
    }

    /// Create encrypted direct msg event
    pub fn new_encrypted_direct_msg(
        sender_keys: &Keys,
        receiver_keys: &Keys,
        content: &str,
    ) -> Result<Self> {
        let msg = nip04::encrypt(
            &sender_keys.secret_key()?,
            &receiver_keys.public_key(),
            content,
        )?;

        Ok(Self::new(
            Kind::Base(KindBase::EncryptedDirectMessage),
            &msg,
            &[Tag::new(TagData::PubKey(receiver_keys.public_key()))],
        ))
    }

    /// Create delete event
    pub fn delete(ids: Vec<sha256::Hash>, reason: Option<&str>) -> Self {
        let tags: Vec<Tag> = ids
            .iter()
            .map(|id| Tag::new(TagData::EventId(*id)))
            .collect();

        Self::new(
            Kind::Base(KindBase::EventDeletion),
            reason.unwrap_or(""),
            &tags,
        )
    }

    /// Add reaction (like/upvote, dislike/downvote) to an event
    pub fn new_reaction(event: &Event, positive: bool) -> Self {
        let tags: &[Tag] = &[
            Tag::new(TagData::EventId(event.id)),
            Tag::new(TagData::PubKey(event.pubkey)),
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
    ///
    pub fn new_channel(name: &str, about: Option<&str>, picture: Option<&str>) -> Result<Self> {
        if !REGEX_NAME.is_match(name) {
            return Err(anyhow!("Invalid name"));
        }

        let metadata: Value = json!({
            "name": name,
            "about": about.unwrap_or(""),
            "picture": picture.unwrap_or(""),
        });

        Ok(Self::new(
            Kind::Base(KindBase::ChannelCreation),
            &metadata.to_string(),
            &[],
        ))
    }

    /// Set channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    ///
    pub fn set_channel_metadata(
        channel_id: sha256::Hash, // event id of kind 40
        relay_url: Url,
        name: Option<&str>,
        about: Option<&str>,
        picture: Option<&str>,
    ) -> Result<Self> {
        if let Some(name) = name {
            if !REGEX_NAME.is_match(name) {
                return Err(anyhow!("Invalid name"));
            }
        }

        let metadata: Value = json!({
            "name": name.unwrap_or(""),
            "about": about.unwrap_or(""),
            "picture": picture.unwrap_or(""),
        });

        Ok(Self::new(
            Kind::Base(KindBase::ChannelMetadata),
            &metadata.to_string(),
            &[Tag::new(TagData::Nip10E(channel_id, relay_url, None))],
        ))
    }

    /// New channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    ///
    pub fn new_channel_msg(
        channel_id: sha256::Hash, // event id of kind 40
        relay_url: Url,
        content: &str,
    ) -> Self {
        Self::new(
            Kind::Base(KindBase::ChannelMessage),
            content,
            &[Tag::new(TagData::Nip10E(
                channel_id,
                relay_url,
                Some(Marker::Root),
            ))],
        )
    }

    /// Hide message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    ///
    pub fn hide_channel_msg(
        message_id: sha256::Hash, // event id of kind 42
        reason: &str,
    ) -> Self {
        let content: Value = json!({
            "reason": reason,
        });

        Self::new(
            Kind::Base(KindBase::ChannelHideMessage),
            &content.to_string(),
            &[Tag::new(TagData::EventId(message_id))],
        )
    }

    /// Hide message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    ///
    pub fn mute_channel_user(pubkey: XOnlyPublicKey, reason: &str) -> Self {
        let content: Value = json!({
            "reason": reason,
        });

        Self::new(
            Kind::Base(KindBase::ChannelMuteUser),
            &content.to_string(),
            &[Tag::new(TagData::PubKey(pubkey))],
        )
    }
}
