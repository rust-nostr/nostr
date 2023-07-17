// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event builder

#[cfg(feature = "mmr")]
use bitcoin_hashes::sha256::Hash;
use core::fmt;
#[cfg(target_arch = "wasm32")]
use instant::Instant;
use secp256k1::XOnlyPublicKey;
use serde_json::{json, Value};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use url::Url;

pub use super::kind::Kind;
pub use super::tag::{Marker, Tag, TagKind};
use super::{Event, EventId, UnsignedEvent};
use crate::key::{self, Keys};
#[cfg(feature = "nip04")]
use crate::nips::nip04;
#[cfg(feature = "nip46")]
use crate::nips::nip46::Message as NostrConnectMessage;
use crate::nips::nip58::Error as Nip58Error;
use crate::nips::{nip13, nip58};
use crate::types::{ChannelId, Contact, Metadata, Timestamp};
use crate::UncheckedUrl;

/// [`EventBuilder`] error
#[derive(Debug)]
pub enum Error {
    /// Key error
    Key(key::Error),
    /// JSON error
    Json(serde_json::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Unsigned event error
    Unsigned(super::unsigned::Error),
    /// NIP04 error
    #[cfg(feature = "nip04")]
    NIP04(nip04::Error),
    /// NIP58 error
    NIP58(nip58::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::Unsigned(e) => write!(f, "{e}"),
            #[cfg(feature = "nip04")]
            Self::NIP04(e) => write!(f, "{e}"),
            Self::NIP58(e) => write!(f, "{e}"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<super::unsigned::Error> for Error {
    fn from(e: super::unsigned::Error) -> Self {
        Self::Unsigned(e)
    }
}

#[cfg(feature = "nip04")]
impl From<nip04::Error> for Error {
    fn from(e: nip04::Error) -> Self {
        Self::NIP04(e)
    }
}

impl From<nip58::Error> for Error {
    fn from(e: nip58::Error) -> Self {
        Self::NIP58(e)
    }
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
        let pubkey: XOnlyPublicKey = keys.public_key();
        Ok(self.to_unsigned_event(pubkey).sign(keys)?)
    }

    /// Build [`UnsignedEvent`]
    pub fn to_unsigned_event(self, pubkey: XOnlyPublicKey) -> UnsignedEvent {
        let created_at: Timestamp = Timestamp::now();
        let id = EventId::new(&pubkey, created_at, &self.kind, &self.tags, &self.content);
        UnsignedEvent {
            id,
            pubkey,
            created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
        }
    }

    /// Build POW [`Event`]
    pub fn to_pow_event(self, keys: &Keys, difficulty: u8) -> Result<Event, Error> {
        let pubkey: XOnlyPublicKey = keys.public_key();
        Ok(self.to_unsigned_pow_event(pubkey, difficulty).sign(keys)?)
    }

    /// Build unsigned POW [`Event`]
    pub fn to_unsigned_pow_event(self, pubkey: XOnlyPublicKey, difficulty: u8) -> UnsignedEvent {
        let mut nonce: u128 = 0;
        let mut tags: Vec<Tag> = self.tags;

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

                return UnsignedEvent {
                    id,
                    pubkey,
                    created_at,
                    kind: self.kind,
                    tags,
                    content: self.content,
                };
            }

            tags.pop();
        }
    }

    /// Build MMR [`Event`]
    #[cfg(feature = "mmr")]
    pub fn to_mmr_event(
        self,
        keys: &Keys,
        prev_event_hash: Hash,
        prev_mmr_root: Hash,
        prev_event_pos: i64,
    ) -> Result<Event, Error> {
        let pubkey: XOnlyPublicKey = keys.public_key();
        Ok(self
            .to_unsigned_mmr_event(pubkey, prev_event_hash, prev_mmr_root, prev_event_pos)
            .sign(keys)?)
    }

    /// Build unsigned MMR [`Event`]
    #[cfg(feature = "mmr")]
    pub fn to_unsigned_mmr_event(
        self,
        pubkey: XOnlyPublicKey,
        prev_event_id: Hash,
        prev_mmr_root: Hash,
        prev_event_pos: i64,
    ) -> UnsignedEvent {
        let mut tags: Vec<Tag> = self.tags;
        tags.push(Tag::Mmr {
            prev_event_id,
            prev_mmr_root,
            prev_event_pos,
        });
        let created_at: Timestamp = Timestamp::now();
        let id = EventId::new(&pubkey, created_at, &self.kind, &tags, &self.content);
        // TODO verify if valid MMR append operation for vector commitment
        UnsignedEvent {
            id,
            pubkey,
            created_at,
            kind: self.kind,
            tags,
            content: self.content,
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
    ///     .lud16("yuki@getalby.com");
    ///
    /// let builder = EventBuilder::set_metadata(metadata);
    /// ```
    pub fn set_metadata(metadata: Metadata) -> Self {
        Self::new(Kind::Metadata, metadata.as_json(), &[])
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

    /// Long-form text note (generally referred to as "articles" or "blog posts").
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/23.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::str::FromStr;
    ///
    /// use nostr::{EventBuilder, Tag, Timestamp, EventId, UncheckedUrl};
    ///
    /// let event_id = EventId::from_hex("b3e392b11f5d4f28321cedd09303a748acfd0487aea5a7450b3481c60b6e4f87").unwrap();
    /// let content: &str = "Lorem [ipsum][4] dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n\nRead more at #[3].";
    /// let tags = &[
    ///     Tag::Identifier("lorem-ipsum".to_string()),
    ///     Tag::Title("Lorem Ipsum".to_string()),
    ///     Tag::PublishedAt(Timestamp::from(1296962229)),
    ///     Tag::Hashtag("placeholder".to_string()),
    ///     Tag::Event(event_id, Some(UncheckedUrl::from("wss://relay.example.com")), None),
    /// ];
    /// let builder = EventBuilder::long_form_text_note("My first text note from Nostr SDK!", &[]);
    /// ```
    pub fn long_form_text_note<S>(content: S, tags: &[Tag]) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::LongFormTextNote, content, tags)
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
        Ok(Self::new(
            Kind::EncryptedDirectMessage,
            nip04::encrypt(&sender_keys.secret_key()?, &receiver_pubkey, content.into())?,
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
    pub fn new_channel(metadata: Metadata) -> Self {
        Self::new(Kind::ChannelCreation, metadata.as_json(), &[])
    }

    /// Set channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn set_channel_metadata(
        channel_id: ChannelId,
        relay_url: Option<Url>,
        metadata: Metadata,
    ) -> Self {
        Self::new(
            Kind::ChannelMetadata,
            metadata.as_json(),
            &[Tag::Event(
                channel_id.into(),
                relay_url.map(|u| u.into()),
                None,
            )],
        )
    }

    /// New channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn new_channel_msg<S>(channel_id: ChannelId, relay_url: Url, content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::ChannelMessage,
            content,
            &[Tag::Event(
                channel_id.into(),
                Some(relay_url.into()),
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
            &[Tag::Challenge(challenge.into()), Tag::Relay(relay.into())],
        )
    }

    /// Nostr Connect
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/46.md>
    #[cfg(all(feature = "nip04", feature = "nip46"))]
    pub fn nostr_connect(
        sender_keys: &Keys,
        receiver_pubkey: XOnlyPublicKey,
        msg: NostrConnectMessage,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            Kind::NostrConnect,
            nip04::encrypt(&sender_keys.secret_key()?, &receiver_pubkey, msg.as_json())?,
            &[Tag::PubKey(receiver_pubkey, None)],
        ))
    }

    /// Create report event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    pub fn report<S>(tags: &[Tag], content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::Reporting, content, tags)
    }

    /// Create zap request event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    pub fn new_zap_request<S>(
        pubkey: XOnlyPublicKey,
        event_id: Option<EventId>,
        amount: Option<u64>,
        lnurl: Option<S>,
    ) -> Self
    where
        S: Into<String>,
    {
        let mut tags = vec![Tag::PubKey(pubkey, None)];

        if let Some(event_id) = event_id {
            tags.push(Tag::Event(event_id, None, None));
        }

        if let Some(amount) = amount {
            tags.push(Tag::Amount(amount));
        }

        if let Some(lnurl) = lnurl {
            tags.push(Tag::Lnurl(lnurl.into()));
        }

        Self::new(Kind::ZapRequest, "", &tags)
    }

    /// Create zap event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    pub fn new_zap<S>(bolt11: S, preimage: Option<S>, zap_request: Event) -> Self
    where
        S: Into<String>,
    {
        let mut tags = vec![
            Tag::Bolt11(bolt11.into()),
            Tag::Description(zap_request.as_json()),
        ];

        // add preimage tag if provided
        if let Some(pre_image_tag) = preimage {
            tags.push(Tag::Preimage(pre_image_tag.into()))
        }

        // add e tag
        if let Some(tag) = zap_request
            .tags
            .clone()
            .into_iter()
            .find(|t| t.kind() == TagKind::E)
        {
            tags.push(tag);
        }

        // add p tag
        if let Some(tag) = zap_request
            .tags
            .into_iter()
            .find(|t| t.kind() == TagKind::P)
        {
            tags.push(tag);
        }

        Self::new(Kind::Zap, "", &tags)
    }

    /// Create a badge definition event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::nips::nip58::ImageDimensions;
    /// use nostr::EventBuilder;
    ///
    /// let badge_id = String::from("nostr-sdk-test-badge");
    /// let name = Some(String::from("Nostr SDK test badge"));
    /// let description = Some(String::from("This is a test badge"));
    /// let image_url = Some(String::from("https://nostr.build/someimage/1337"));
    /// let image_size = Some(ImageDimensions(1024, 1024));
    /// let thumbs = Some(vec![(
    ///     String::from("https://nostr.build/somethumbnail/1337"),
    ///     Some(ImageDimensions(256, 256)),
    /// )]);
    ///
    /// let event_builder =
    ///     EventBuilder::define_badge(badge_id, name, description, image_url, image_size, thumbs);
    /// ```
    pub fn define_badge<S>(
        badge_id: S,
        name: Option<S>,
        description: Option<S>,
        image: Option<S>,
        image_dimensions: Option<nip58::ImageDimensions>,
        thumbnails: Option<Vec<(S, Option<nip58::ImageDimensions>)>>,
    ) -> Self
    where
        S: Into<String>,
    {
        let mut tags: Vec<Tag> = Vec::new();

        // Set identifier tag
        tags.push(Tag::Identifier(badge_id.into()));

        // Set name tag
        if let Some(name) = name {
            tags.push(Tag::Name(name.into()));
        }

        // Set description tag
        if let Some(description) = description {
            tags.push(Tag::Description(description.into()));
        }

        // Set image tag
        if let Some(image) = image {
            let image_tag = if let Some(dimensions) = image_dimensions {
                let nip58::ImageDimensions(width, height) = dimensions;
                Tag::Image(image.into(), Some((width, height)))
            } else {
                Tag::Image(image.into(), None)
            };
            tags.push(image_tag);
        }

        // Set thumbnail tags
        if let Some(thumbs) = thumbnails {
            for thumb in thumbs {
                let thumb_url = thumb.0.into();
                let thumb_tag = if let Some(width_height) = thumb.1 {
                    let nip58::ImageDimensions(width, height) = width_height;
                    Tag::Thumb(thumb_url, Some((width, height)))
                } else {
                    Tag::Thumb(thumb_url, None)
                };
                tags.push(thumb_tag);
            }
        }

        Self::new(Kind::BadgeDefinition, "", &tags)
    }

    /// Create a badge award event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::event::tag::{Tag, TagKind};
    /// use nostr::key::Keys;
    /// use nostr::EventBuilder;
    /// use nostr::UncheckedUrl;
    /// use secp256k1::XOnlyPublicKey;
    /// use std::str::FromStr;
    ///
    /// let keys = Keys::generate();
    ///
    /// // Create a new badge event
    /// let badge_id = String::from("bravery");
    /// let badge_definition_event = EventBuilder::define_badge(badge_id, None, None, None, None, None)
    ///     .to_event(&keys)
    ///     .unwrap();
    ///
    /// let awarded_pubkeys = vec![
    ///     Tag::PubKey(
    ///         XOnlyPublicKey::from_str(
    ///             "232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df",
    ///         )
    ///         .unwrap(),
    ///         None,
    ///     ),
    ///     Tag::PubKey(
    ///         XOnlyPublicKey::from_str(
    ///             "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
    ///         )
    ///         .unwrap(),
    ///         None,
    ///     ),
    /// ];
    /// let event_builder = EventBuilder::award_badge(&badge_definition_event, awarded_pubkeys.clone());
    /// ```
    pub fn award_badge(badge_definition: &Event, awarded_pubkeys: Vec<Tag>) -> Result<Self, Error> {
        let mut tags = Vec::new();

        let badge_id = badge_definition
            .tags
            .iter()
            .find_map(|t| match t {
                Tag::Identifier(id) => Some(id),
                _ => None,
            })
            .ok_or(Error::NIP58(nip58::Error::IdentifierTagNotFound))?;

        // Add identity tag
        tags.push(Tag::A {
            kind: Kind::BadgeDefinition,
            public_key: badge_definition.pubkey,
            identifier: badge_id.clone(),
            relay_url: None,
        });

        // Add awarded pubkeys
        let ptags: Vec<Tag> = awarded_pubkeys
            .into_iter()
            .filter(|p| matches!(p, Tag::PubKey(..)))
            .collect();

        tags.extend(ptags);

        // Build event
        Ok(Self::new(Kind::BadgeAward, "", &tags))
    }

    /// Create a profile badges event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::event::tag::{Tag, TagKind};
    /// use nostr::key::Keys;
    /// use nostr::EventBuilder;
    /// use nostr::UncheckedUrl;
    /// use secp256k1::XOnlyPublicKey;
    /// use std::str::FromStr;
    ///
    /// // Keys used for defining a badge and awarding it
    /// let new_badge_keys = Keys::generate();
    ///
    /// // Keys that will create the profile badges event
    /// let profile_badges_keys = Keys::generate();
    ///
    /// // Create a new badge event
    /// let badge_id = String::from("bravery");
    /// let badge_definition_event = EventBuilder::define_badge(badge_id, None, None, None, None, None)
    ///     .to_event(&new_badge_keys)
    ///     .unwrap();
    ///
    /// let awarded_pubkeys = vec![Tag::PubKey(profile_badges_keys.public_key(), None)];
    /// let award_badge_event =
    ///     EventBuilder::award_badge(&badge_definition_event, awarded_pubkeys.clone())
    ///         .unwrap()
    ///         .to_event(&new_badge_keys)
    ///         .unwrap();
    ///
    /// let profile_badges_event = EventBuilder::profile_badges(
    ///     vec![badge_definition_event],
    ///     vec![award_badge_event],
    ///     &profile_badges_keys.public_key(),
    /// )
    /// .unwrap()
    /// .to_event(&profile_badges_keys)
    /// .unwrap();
    /// ```
    pub fn profile_badges(
        badge_definitions: Vec<Event>,
        badge_awards: Vec<Event>,
        pubkey_awarded: &XOnlyPublicKey,
    ) -> Result<Self, Error> {
        if badge_definitions.len() != badge_awards.len() {
            return Err(Error::NIP58(nip58::Error::InvalidLength));
        }

        let mut badge_awards = nip58::filter_for_kind(badge_awards, &Kind::BadgeAward);
        if badge_awards.is_empty() {
            return Err(Error::NIP58(Nip58Error::InvalidKind));
        }

        for award in &badge_awards {
            if !award.tags.iter().any(|t| match t {
                Tag::PubKey(pub_key, _) => pub_key == pubkey_awarded,
                _ => false,
            }) {
                return Err(Error::NIP58(Nip58Error::BadgeAwardsLackAwardedPublicKey));
            }
        }

        let mut badge_definitions =
            nip58::filter_for_kind(badge_definitions, &Kind::BadgeDefinition);
        if badge_definitions.is_empty() {
            return Err(Error::NIP58(Nip58Error::InvalidKind));
        }

        // Add identifier `d` tag
        let id_tag = Tag::Identifier("profile_badges".to_owned());
        let mut tags: Vec<Tag> = vec![id_tag];

        let badge_definitions_identifiers = badge_definitions
            .iter_mut()
            .map(|event| {
                let tags = core::mem::take(&mut event.tags);
                let id =
                    nip58::extract_identifier(tags).ok_or(Nip58Error::IdentifierTagNotFound)?;

                Ok((event.clone(), id))
            })
            .collect::<Result<Vec<(Event, Tag)>, Nip58Error>>();
        let badge_definitions_identifiers =
            badge_definitions_identifiers.map_err(|_| nip58::Error::IdentifierTagNotFound)?;

        let badge_awards_identifiers = badge_awards
            .iter_mut()
            .map(|event| {
                let tags = core::mem::take(&mut event.tags);
                let (_, relay_url) = nip58::extract_awarded_public_key(&tags, pubkey_awarded)
                    .ok_or(Nip58Error::BadgeAwardsLackAwardedPublicKey)?;
                let (id, a_tag) = tags
                    .iter()
                    .find_map(|t| match t {
                        Tag::A { identifier, .. } => Some((identifier.clone(), t.clone())),
                        _ => None,
                    })
                    .ok_or(Nip58Error::BadgeAwardMissingATag)?;
                Ok((event.clone(), id, a_tag, relay_url))
            })
            .collect::<Result<Vec<(Event, String, Tag, Option<UncheckedUrl>)>, Nip58Error>>();
        let badge_awards_identifiers = badge_awards_identifiers?;

        // This collection has been filtered for the needed tags
        let users_badges: Vec<(_, _)> =
            core::iter::zip(badge_definitions_identifiers, badge_awards_identifiers).collect();

        for (badge_definition, badge_award) in users_badges {
            match (&badge_definition, &badge_award) {
                ((_, Tag::Identifier(identifier)), (_, badge_id, ..)) if badge_id != identifier => {
                    return Err(Error::NIP58(Nip58Error::MismatchedBadgeDefinitionOrAward));
                }
                (
                    (_, Tag::Identifier(identifier)),
                    (badge_award_event, badge_id, a_tag, relay_url),
                ) if badge_id == identifier => {
                    let badge_definition_event_tag = a_tag.clone().to_owned();
                    let badge_award_event_tag =
                        Tag::Event(badge_award_event.clone().id, relay_url.clone(), None);
                    tags.extend_from_slice(&[badge_definition_event_tag, badge_award_event_tag]);
                }
                _ => {}
            }
        }

        let event_builder = EventBuilder::new(Kind::ProfileBadges, String::new(), &tags);

        Ok(event_builder)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use secp256k1::{SecretKey, XOnlyPublicKey};

    use crate::{
        nips::nip58::ImageDimensions, Event, EventBuilder, Keys, Kind, Result, Tag, UncheckedUrl,
    };

    #[test]
    fn round_trip() -> Result<()> {
        let keys = Keys::new(SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);

        let event = EventBuilder::new_text_note("hello", &vec![]).to_event(&keys)?;

        let serialized = event.as_json();
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

    #[test]
    fn test_zap_event_builder() {
        let bolt11 = String::from("lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0");
        let preimage = Some(String::from(
            "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f",
        ));
        let zap_request_json = String::from("{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}");
        let zap_request_event: Event = Event::from_json(zap_request_json).unwrap();
        let event_builder = EventBuilder::new_zap(bolt11, preimage, zap_request_event);

        assert_eq!(5, event_builder.tags.len());

        let has_preimage_tag = event_builder
            .tags
            .clone()
            .iter()
            .find(|t| matches!(t, Tag::Preimage(_)))
            .is_some();

        assert_eq!(true, has_preimage_tag);
    }

    #[test]
    fn test_zap_event_builder_without_preimage() {
        let bolt11 = String::from("lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0");
        let preimage = None;
        let zap_request_json = String::from("{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}");
        let zap_request_event = Event::from_json(zap_request_json).unwrap();
        let event_builder = EventBuilder::new_zap(bolt11, preimage, zap_request_event);

        assert_eq!(4, event_builder.tags.len());
        let has_preimage_tag = event_builder
            .tags
            .clone()
            .iter()
            .find(|t| matches!(t, Tag::Preimage(_)))
            .is_some();

        assert_eq!(false, has_preimage_tag);
    }

    #[test]
    fn test_badge_definition_event_builder_badge_id_only() {
        let badge_id = String::from("bravery");
        let event_builder = EventBuilder::define_badge(badge_id, None, None, None, None, None);

        let has_id = event_builder
            .tags
            .clone()
            .iter()
            .find(|t| matches!(t, Tag::Identifier(_)))
            .is_some();
        assert_eq!(true, has_id);

        assert_eq!(Kind::BadgeDefinition, event_builder.kind);
    }

    #[test]
    fn test_badge_definition_event_builder_full() {
        let badge_id = String::from("bravery");
        let name = Some(String::from("Bravery"));
        let description = Some(String::from("Brave pubkey"));
        let image_url = Some(String::from("https://nostr.build/someimage/1337"));
        let image_size = Some(ImageDimensions(1024, 1024));
        let thumbs = Some(vec![(
            String::from("https://nostr.build/somethumbnail/1337"),
            Some(ImageDimensions(256, 256)),
        )]);

        let event_builder =
            EventBuilder::define_badge(badge_id, name, description, image_url, image_size, thumbs);

        let has_id = event_builder
            .tags
            .clone()
            .iter()
            .find(|t| matches!(t, Tag::Identifier(_)))
            .is_some();
        assert_eq!(true, has_id);

        assert_eq!(Kind::BadgeDefinition, event_builder.kind);
    }

    #[test]
    fn test_badge_award_event_builder() -> Result<()> {
        let keys = Keys::generate();
        let pub_key = keys.public_key();

        // Set up badge definition
        let badge_definition_event_json = format!(
            r#"{{
                "id": "4d16822726cefcb45768988c6451b6de5a20b504b8df85efe0808caf346e167c",
                "pubkey": "{}",
                "created_at": 1677921759,
                "kind": 30009,
                "tags": [
                  ["d", "bravery"],
                  ["name", "Bravery"],
                  ["description", "A brave soul"]
                ],
                "content": "",
                "sig": "cf154350a615f0355d165b52c7ecccce563d9a935801181e9016d077f38d31a1dc992a757ef8d652a416885f33d836cf408c79f5d983d6f1f03c966ace946d59"
              }}"#,
            pub_key.to_string()
        );
        let badge_definition_event: Event =
            serde_json::from_str(&badge_definition_event_json).unwrap();

        // Set up goal event
        let example_event_json = format!(
            r#"{{
            "content": "",
            "id": "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "kind": 8,
            "pubkey": "{}",
            "sig": "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            "created_at": 1671739153,
            "tags": [
                ["a", "30009:{}:bravery"],
                ["p", "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245", "wss://nostr.oxtr.dev"],
                ["p", "232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df", "wss://nostr.oxtr.dev"]
            ]
            }}"#,
            pub_key.to_string(),
            pub_key.to_string()
        );
        let example_event: Event = serde_json::from_str(&example_event_json).unwrap();

        // Create new event with the event builder
        let awarded_pubkeys = vec![
            Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
                )?,
                Some(UncheckedUrl::from_str("wss://nostr.oxtr.dev")?),
            ),
            Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df",
                )?,
                Some(UncheckedUrl::from_str("wss://nostr.oxtr.dev")?),
            ),
        ];
        let event_builder: Event =
            EventBuilder::award_badge(&badge_definition_event, awarded_pubkeys)?
                .to_event(&keys)
                .unwrap();

        assert_eq!(event_builder.kind, Kind::BadgeAward);
        assert_eq!(event_builder.content, "");
        assert_eq!(event_builder.tags, example_event.tags);

        Ok(())
    }

    #[test]
    fn test_profile_badges() -> Result<()> {
        // The pubkey used for profile badges event
        let keys = Keys::generate();
        let pub_key = keys.public_key();

        // Create badge 1
        let badge_one_keys = Keys::generate();
        let badge_one_pubkey = badge_one_keys.public_key();
        let relay_url = UncheckedUrl::from_str("wss://nostr.oxtr.dev").unwrap();

        let awarded_pubkeys = vec![
            Tag::PubKey(pub_key.clone(), Some(relay_url.clone())),
            Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df",
                )?,
                Some(UncheckedUrl::from_str("wss://nostr.oxtr.dev")?),
            ),
        ];
        let bravery_badge_event =
            self::EventBuilder::define_badge("bravery", None, None, None, None, None)
                .to_event(&badge_one_keys)?;
        let bravery_badge_award =
            self::EventBuilder::award_badge(&bravery_badge_event, awarded_pubkeys.clone())?
                .to_event(&badge_one_keys)?;

        //Badge 2
        let badge_two_keys = Keys::generate();
        let badge_two_pubkey = badge_two_keys.public_key();

        let honor_badge_event =
            self::EventBuilder::define_badge("honor", None, None, None, None, None)
                .to_event(&badge_two_keys)?;
        let honor_badge_award =
            self::EventBuilder::award_badge(&honor_badge_event, awarded_pubkeys.clone())?
                .to_event(&badge_two_keys)?;

        let example_event_json = format!(
            r#"{{
            "content":"",
            "id": "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "kind": 30008,
            "pubkey": "{}",
            "sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            "created_at":1671739153,
            "tags":[
                ["d", "profile_badges"],
                ["a", "30009:{}:bravery"],
                ["e", "{}", "wss://nostr.oxtr.dev"],
                ["a", "30009:{}:honor"],
                ["e", "{}", "wss://nostr.oxtr.dev"]
            ]
            }}"#,
            pub_key.to_string(),
            badge_one_pubkey.to_string(),
            bravery_badge_award.id.to_string(),
            badge_two_pubkey.to_string(),
            honor_badge_award.id.to_string(),
        );
        let example_event: Event = serde_json::from_str(&example_event_json).unwrap();

        let badge_definitions = vec![bravery_badge_event, honor_badge_event];
        let badge_awards = vec![bravery_badge_award, honor_badge_award];
        let profile_badges =
            EventBuilder::profile_badges(badge_definitions, badge_awards, &pub_key)?
                .to_event(&keys)?;

        assert_eq!(profile_badges.kind, Kind::ProfileBadges);
        assert_eq!(profile_badges.tags, example_event.tags);
        Ok(())
    }
}
