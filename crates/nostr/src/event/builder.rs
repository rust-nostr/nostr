// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Event builder

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

#[cfg(feature = "std")]
use bitcoin::secp256k1::rand;
use bitcoin::secp256k1::rand::{CryptoRng, Rng};
use bitcoin::secp256k1::{self, Secp256k1, Signing, Verification};
use serde_json::{json, Value};

use super::kind::{Kind, NIP90_JOB_REQUEST_RANGE, NIP90_JOB_RESULT_RANGE};
use super::tag::ImageDimensions;
use super::{Event, EventId, Marker, Tag, TagKind, TagStandard, UnsignedEvent};
use crate::key::{self, Keys, PublicKey};
use crate::nips::nip01::Coordinate;
#[cfg(feature = "nip04")]
use crate::nips::nip04;
use crate::nips::nip15::{ProductData, StallData};
#[cfg(all(feature = "std", feature = "nip44"))]
use crate::nips::nip44::{self, Version};
#[cfg(all(feature = "std", feature = "nip46"))]
use crate::nips::nip46::Message as NostrConnectMessage;
use crate::nips::nip51::{ArticlesCuration, Bookmarks, Emojis, Interests, MuteList};
use crate::nips::nip53::LiveEvent;
#[cfg(feature = "nip57")]
use crate::nips::nip57::ZapRequestData;
use crate::nips::nip58::Error as Nip58Error;
#[cfg(all(feature = "std", feature = "nip59"))]
use crate::nips::nip59;
use crate::nips::nip90::DataVendingMachineStatus;
use crate::nips::nip94::FileMetadata;
use crate::nips::nip98::HttpData;
use crate::nips::{nip13, nip58};
#[cfg(feature = "std")]
use crate::types::time::Instant;
use crate::types::time::TimeSupplier;
use crate::types::{Contact, Metadata, Timestamp};
use crate::util::{EventIdOrCoordinate, IntoPublicKey};
#[cfg(feature = "std")]
use crate::SECP256K1;
use crate::{Alphabet, JsonUtil, RelayMetadata, SingleLetterTag, UncheckedUrl, Url};

/// Wrong kind error
#[derive(Debug)]
pub enum WrongKindError {
    /// Singe kind
    Single(Kind),
    /// Range
    Range(Range<u16>),
}

impl fmt::Display for WrongKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(k) => write!(f, "{k}"),
            Self::Range(range) => write!(f, "'{} <= k <= {}'", range.start, range.end),
        }
    }
}

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
    /// OpenTimestamps error
    #[cfg(feature = "nip03")]
    OpenTimestamps(nostr_ots::Error),
    /// NIP04 error
    #[cfg(feature = "nip04")]
    NIP04(nip04::Error),
    /// NIP44 error
    #[cfg(all(feature = "std", feature = "nip44"))]
    NIP44(nip44::Error),
    /// NIP58 error
    NIP58(nip58::Error),
    /// Wrong kind
    WrongKind {
        /// The received wrong kind
        received: Kind,
        /// The expected kind (single or range)
        expected: WrongKindError,
    },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "Key: {e}"),
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::Unsigned(e) => write!(f, "Unsigned event: {e}"),
            #[cfg(feature = "nip03")]
            Self::OpenTimestamps(e) => write!(f, "NIP03: {e}"),
            #[cfg(feature = "nip04")]
            Self::NIP04(e) => write!(f, "NIP04: {e}"),
            #[cfg(all(feature = "std", feature = "nip44"))]
            Self::NIP44(e) => write!(f, "NIP44: {e}"),
            Self::NIP58(e) => write!(f, "NIP58: {e}"),
            Self::WrongKind { received, expected } => {
                write!(f, "Wrong kind: received={received}, expected={expected}")
            }
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

#[cfg(feature = "nip03")]
impl From<nostr_ots::Error> for Error {
    fn from(e: nostr_ots::Error) -> Self {
        Self::OpenTimestamps(e)
    }
}

#[cfg(feature = "nip04")]
impl From<nip04::Error> for Error {
    fn from(e: nip04::Error) -> Self {
        Self::NIP04(e)
    }
}

#[cfg(all(feature = "std", feature = "nip44"))]
impl From<nip44::Error> for Error {
    fn from(e: nip44::Error) -> Self {
        Self::NIP44(e)
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
    custom_created_at: Option<Timestamp>,
}

impl EventBuilder {
    /// New [`EventBuilder`]
    #[inline]
    pub fn new<S, I>(kind: Kind, content: S, tags: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = Tag>,
    {
        Self {
            kind,
            tags: tags.into_iter().collect(),
            content: content.into(),
            custom_created_at: None,
        }
    }

    /// Set a custom `created_at` UNIX timestamp
    #[inline]
    pub fn custom_created_at(mut self, created_at: Timestamp) -> Self {
        self.custom_created_at = Some(created_at);
        self
    }

    /// Build [`Event`]
    #[inline]
    pub fn to_event_with_ctx<C, R, T>(
        self,
        secp: &Secp256k1<C>,
        rng: &mut R,
        supplier: &T,
        keys: &Keys,
    ) -> Result<Event, Error>
    where
        C: Signing + Verification,
        R: Rng + CryptoRng,
        T: TimeSupplier,
    {
        let pubkey: PublicKey = keys.public_key().clone();
        Ok(self
            .to_unsigned_event_with_supplier(supplier, pubkey)
            .sign_with_ctx(secp, rng, keys)?)
    }

    /// Build [`UnsignedEvent`]
    #[inline]
    pub fn to_unsigned_event_with_supplier<P, T>(self, supplier: &T, public_key: P) -> UnsignedEvent
    where
        P: IntoPublicKey,
        T: TimeSupplier,
    {
        UnsignedEvent {
            id: None,
            pubkey: public_key.into_public_key(),
            created_at: self
                .custom_created_at
                .unwrap_or_else(|| Timestamp::now_with_supplier(supplier)),
            kind: self.kind,
            tags: self.tags,
            content: self.content,
        }
    }

    /// Build POW [`Event`]
    #[inline]
    pub fn to_pow_event_with_ctx<C, R, T>(
        self,
        secp: &Secp256k1<C>,
        rng: &mut R,
        supplier: &T,
        keys: &Keys,
        difficulty: u8,
    ) -> Result<Event, Error>
    where
        C: Signing + Verification,
        R: Rng + CryptoRng,
        T: TimeSupplier,
    {
        let pubkey: PublicKey = keys.public_key().clone();
        Ok(self
            .to_unsigned_pow_event_with_supplier(supplier, pubkey, difficulty)
            .sign_with_ctx(secp, rng, keys)?)
    }

    /// Build unsigned POW [`Event`]
    pub fn to_unsigned_pow_event_with_supplier<P, T>(
        self,
        supplier: &T,
        public_key: P,
        difficulty: u8,
    ) -> UnsignedEvent
    where
        P: IntoPublicKey,
        T: TimeSupplier,
    {
        let public_key: PublicKey = public_key.into_public_key();

        let mut nonce: u128 = 0;
        let mut tags: Vec<Tag> = self.tags;

        #[cfg(feature = "std")]
        let now = Instant::now();

        loop {
            nonce += 1;

            tags.push(Tag::pow(nonce, difficulty));

            let created_at: Timestamp = self
                .custom_created_at
                .unwrap_or_else(|| Timestamp::now_with_supplier(supplier));
            let id: EventId =
                EventId::new(&public_key, &created_at, &self.kind, &tags, &self.content);

            if nip13::get_leading_zero_bits(id.as_bytes()) >= difficulty {
                #[cfg(feature = "std")]
                tracing::debug!(
                    "{} iterations in {} ms. Avg rate {} hashes/second",
                    nonce,
                    now.elapsed().as_millis(),
                    nonce * 1000 / std::cmp::max(1, now.elapsed().as_millis())
                );

                return UnsignedEvent {
                    id: Some(id),
                    pubkey: public_key,
                    created_at,
                    kind: self.kind,
                    tags,
                    content: self.content,
                };
            }

            tags.pop();
        }
    }
}

impl EventBuilder {
    /// Build [`Event`]
    #[inline]
    #[cfg(feature = "std")]
    pub fn to_event(self, keys: &Keys) -> Result<Event, Error> {
        self.to_event_with_ctx(&SECP256K1, &mut rand::thread_rng(), &Instant::now(), keys)
    }

    /// Build [`UnsignedEvent`]
    #[inline]
    #[cfg(feature = "std")]
    pub fn to_unsigned_event<P>(self, public_key: P) -> UnsignedEvent
    where
        P: IntoPublicKey,
    {
        self.to_unsigned_event_with_supplier(&Instant::now(), public_key)
    }

    /// Build POW [`Event`]
    #[inline]
    #[cfg(feature = "std")]
    pub fn to_pow_event(self, keys: &Keys, difficulty: u8) -> Result<Event, Error> {
        self.to_pow_event_with_ctx(
            &SECP256K1,
            &mut rand::thread_rng(),
            &Instant::now(),
            keys,
            difficulty,
        )
    }

    /// Build unsigned POW [`Event`]
    #[inline]
    #[cfg(feature = "std")]
    pub fn to_unsigned_pow_event<P>(self, public_key: P, difficulty: u8) -> UnsignedEvent
    where
        P: IntoPublicKey,
    {
        self.to_unsigned_pow_event_with_supplier(&Instant::now(), public_key, difficulty)
    }
}

impl EventBuilder {
    /// Profile metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::{EventBuilder, Metadata, Url};
    ///
    /// let metadata = Metadata::new()
    ///     .name("username")
    ///     .display_name("My Username")
    ///     .about("Description")
    ///     .picture(Url::parse("https://example.com/avatar.png").unwrap())
    ///     .nip05("username@example.com")
    ///     .lud16("yuki@getalby.com");
    ///
    /// let builder = EventBuilder::metadata(&metadata);
    /// ```
    #[inline]
    pub fn metadata(metadata: &Metadata) -> Self {
        Self::new(Kind::Metadata, metadata.as_json(), [])
    }

    /// Relay list metadata (NIP65)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    pub fn relay_list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Url, Option<RelayMetadata>)>,
    {
        let tags = iter
            .into_iter()
            .map(|(url, metadata)| Tag::relay_metadata(url, metadata));
        Self::new(Kind::RelayList, "", tags)
    }

    /// Text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::EventBuilder;
    ///
    /// let builder = EventBuilder::text_note("My first text note from rust-nostr!", []);
    /// ```
    #[inline]
    pub fn text_note<S, I>(content: S, tags: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = Tag>,
    {
        Self::new(Kind::TextNote, content, tags)
    }

    /// Text note reply
    ///
    /// If no `root` is passed, the `rely_to` will be used for root `e` tag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/10.md>
    pub fn text_note_reply<S>(
        content: S,
        reply_to: &Event,
        root: Option<&Event>,
        relay_url: Option<UncheckedUrl>,
    ) -> Self
    where
        S: Into<String>,
    {
        let mut tags: Vec<Tag> = Vec::new();

        // Add `e` and `p` tag of **root** event
        match root {
            Some(root) => {
                // ID and author
                tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
                    event_id: root.id(),
                    relay_url: relay_url.clone(),
                    marker: Some(Marker::Root),
                }));
                tags.push(Tag::public_key(root.author().clone()));

                // Add others `p` tags
                tags.extend(
                    root.iter_tags()
                        .filter(|t| {
                            t.kind()
                                == TagKind::SingleLetter(SingleLetterTag {
                                    character: Alphabet::P,
                                    uppercase: false,
                                })
                        })
                        .cloned(),
                );
            }
            None => {
                // No root event is passed, use `reply_to` event ID for `root` marker
                tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
                    event_id: reply_to.id(),
                    relay_url: relay_url.clone(),
                    marker: Some(Marker::Root),
                }));
            }
        }

        // Add `e` and `p` tag of event author
        tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
            event_id: reply_to.id(),
            relay_url,
            marker: Some(Marker::Reply),
        }));
        tags.push(Tag::public_key(reply_to.author().clone()));

        // Add others `p` tags of reply_to event
        tags.extend(
            reply_to
                .iter_tags()
                .filter(|t| {
                    t.kind()
                        == TagKind::SingleLetter(SingleLetterTag {
                            character: Alphabet::P,
                            uppercase: false,
                        })
                })
                .cloned(),
        );

        // Compose event
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
    /// use nostr::prelude::*;
    ///
    /// let event_id = EventId::from_hex("b3e392b11f5d4f28321cedd09303a748acfd0487aea5a7450b3481c60b6e4f87").unwrap();
    /// let content: &str = "Lorem [ipsum][4] dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n\nRead more at #[3].";
    /// let tags = &[
    ///     Tag::identifier("lorem-ipsum".to_string()),
    ///     Tag::from_standardized(TagStandard::Title("Lorem Ipsum".to_string())),
    ///     Tag::from_standardized(TagStandard::PublishedAt(Timestamp::from(1296962229))),
    ///     Tag::hashtag("placeholder".to_string()),
    ///     Tag::event(event_id),
    /// ];
    /// let builder = EventBuilder::long_form_text_note("My first text note from rust-nostr!", []);
    /// ```
    #[inline]
    pub fn long_form_text_note<S, I>(content: S, tags: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = Tag>,
    {
        Self::new(Kind::LongFormTextNote, content, tags)
    }

    /// Contact list
    pub fn contact_list<I>(contacts: I) -> Self
    where
        I: IntoIterator<Item = Contact>,
    {
        let tags = contacts.into_iter().map(|contact| {
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: contact.public_key,
                relay_url: contact.relay_url,
                alias: contact.alias,
                uppercase: false,
            })
        });
        Self::new(Kind::ContactList, "", tags)
    }

    /// OpenTimestamps Attestations for Events
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/03.md>
    #[cfg(feature = "nip03")]
    pub fn opentimestamps(
        event_id: EventId,
        relay_url: Option<UncheckedUrl>,
    ) -> Result<Self, Error> {
        let ots: String = nostr_ots::timestamp_event(&event_id.to_hex())?;
        Ok(Self::new(
            Kind::OpenTimestamps,
            ots,
            [Tag::from_standardized_without_cell(TagStandard::Event {
                event_id,
                relay_url,
                marker: None,
            })],
        ))
    }

    /// Create encrypted direct msg event
    ///
    /// <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP-17!</div>
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/04.md>
    #[cfg(all(feature = "std", feature = "nip04"))]
    pub fn encrypted_direct_msg<S>(
        sender_keys: &Keys,
        receiver_pubkey: &PublicKey,
        content: S,
        reply_to: Option<EventId>,
    ) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let content: String =
            nip04::encrypt(sender_keys.secret_key()?, receiver_pubkey, content.into())?;
        let mut tags: Vec<Tag> = vec![Tag::public_key(receiver_pubkey.clone())];
        if let Some(reply_to) = reply_to {
            tags.push(Tag::event(reply_to));
        }
        Ok(Self::new(Kind::EncryptedDirectMessage, content, tags))
    }

    /// Repost
    pub fn repost(event: &Event, relay_url: Option<UncheckedUrl>) -> Self {
        if event.kind == Kind::TextNote {
            Self::new(
                Kind::Repost,
                event.as_json(),
                [
                    Tag::from_standardized_without_cell(TagStandard::Event {
                        event_id: event.id(),
                        relay_url,
                        marker: None,
                    }),
                    Tag::public_key(event.author().clone()),
                ],
            )
        } else {
            Self::new(
                Kind::GenericRepost,
                event.as_json(),
                [
                    Tag::from_standardized_without_cell(TagStandard::Event {
                        event_id: event.id(),
                        relay_url,
                        marker: None,
                    }),
                    Tag::public_key(event.author().clone()),
                    Tag::from_standardized_without_cell(TagStandard::Kind(event.kind())),
                ],
            )
        }
    }

    /// Create delete event
    #[inline]
    pub fn delete<I, T>(ids: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<EventIdOrCoordinate>,
    {
        Self::delete_with_reason(ids, "")
    }

    /// Create delete event with reason
    pub fn delete_with_reason<I, T, S>(ids: I, reason: S) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<EventIdOrCoordinate>,
        S: Into<String>,
    {
        let tags = ids.into_iter().map(|t| {
            let middle: EventIdOrCoordinate = t.into();
            middle.into()
        });
        Self::new(Kind::EventDeletion, reason.into(), tags)
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    #[inline]
    pub fn reaction<S>(event: &Event, reaction: S) -> Self
    where
        S: Into<String>,
    {
        Self::reaction_extended(event.id(), event.author().clone(), event.kind(), reaction)
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    pub fn reaction_extended<S>(
        event_id: EventId,
        public_key: PublicKey,
        kind: Kind,
        reaction: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::Reaction,
            reaction,
            [
                Tag::event(event_id),
                Tag::public_key(public_key),
                Tag::from_standardized_without_cell(TagStandard::Kind(kind)),
            ],
        )
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub fn channel(metadata: &Metadata) -> Self {
        Self::new(Kind::ChannelCreation, metadata.as_json(), [])
    }

    /// Channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub fn channel_metadata(
        channel_id: EventId,
        relay_url: Option<Url>,
        metadata: &Metadata,
    ) -> Self {
        Self::new(
            Kind::ChannelMetadata,
            metadata.as_json(),
            [Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: channel_id,
                relay_url: relay_url.map(|u| u.into()),
                marker: None,
            })],
        )
    }

    /// Channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub fn channel_msg<S>(channel_id: EventId, relay_url: Url, content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::ChannelMessage,
            content,
            [Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: channel_id,
                relay_url: Some(relay_url.into()),
                marker: Some(Marker::Root),
            })],
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
            [Tag::event(message_id)],
        )
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn mute_channel_user<S>(public_key: PublicKey, reason: Option<S>) -> Self
    where
        S: Into<String>,
    {
        let content: Value = json!({
            "reason": reason.map(|s| s.into()).unwrap_or_default(),
        });

        Self::new(
            Kind::ChannelMuteUser,
            content.to_string(),
            [Tag::public_key(public_key)],
        )
    }

    /// Create an auth event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn auth<S>(challenge: S, relay: Url) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::Authentication,
            "",
            [
                Tag::from_standardized_without_cell(TagStandard::Challenge(challenge.into())),
                Tag::from_standardized_without_cell(TagStandard::Relay(relay.into())),
            ],
        )
    }

    /// Nostr Connect
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/46.md>
    #[inline]
    #[cfg(all(feature = "std", feature = "nip04", feature = "nip46"))]
    pub fn nostr_connect(
        sender_keys: &Keys,
        receiver_pubkey: PublicKey,
        msg: NostrConnectMessage,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            Kind::NostrConnect,
            nip04::encrypt(sender_keys.secret_key()?, &receiver_pubkey, msg.as_json())?,
            [Tag::public_key(receiver_pubkey)],
        ))
    }

    /// Live Event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    #[inline]
    pub fn live_event(live_event: LiveEvent) -> Self {
        let tags: Vec<Tag> = live_event.into();
        Self::new(Kind::LiveEvent, "", tags)
    }

    /// Live Event Message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    pub fn live_event_msg<S>(
        live_event_id: S,
        live_event_host: PublicKey,
        content: S,
        relay_url: Option<Url>,
        mut tags: Vec<Tag>,
    ) -> Self
    where
        S: Into<String>,
    {
        tags.push(Tag::from_standardized_without_cell(
            TagStandard::Coordinate {
                coordinate: Coordinate::new(Kind::LiveEvent, live_event_host)
                    .identifier(live_event_id),
                relay_url: relay_url.map(|u| u.into()),
            },
        ));
        Self::new(Kind::LiveEventMessage, content, tags)
    }

    /// Create report event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    pub fn report<I, S>(tags: I, content: S) -> Self
    where
        I: IntoIterator<Item = Tag>,
        S: Into<String>,
    {
        Self::new(Kind::Reporting, content, tags)
    }

    /// Create **public** zap request event
    ///
    /// **This event MUST NOT be broadcasted to relays**, instead must be sent to a recipient's LNURL pay callback url.
    ///
    /// To build a **private** or **anonymous** zap request, use:
    ///
    /// ```rust,no_run
    /// use nostr::prelude::*;
    ///
    /// # #[cfg(all(feature = "std", feature = "nip57"))]
    /// # fn main() {
    /// # let keys = Keys::generate();
    /// # let public_key = PublicKey::from_bech32(
    /// # "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy",
    /// # ).unwrap();
    /// # let relays = [UncheckedUrl::from("wss://relay.damus.io")];
    /// let data = ZapRequestData::new(public_key, relays).message("Zap!");
    ///
    /// let anon_zap: Event = nip57::anonymous_zap_request(data.clone()).unwrap();
    /// println!("Anonymous zap request: {anon_zap:#?}");
    ///
    /// let private_zap: Event = nip57::private_zap_request(data, &keys).unwrap();
    /// println!("Private zap request: {private_zap:#?}");
    /// # }
    ///
    /// # #[cfg(not(all(feature = "std", feature = "nip57")))]
    /// # fn main() {}
    /// ```
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[cfg(feature = "nip57")]
    pub fn public_zap_request(data: ZapRequestData) -> Self {
        let message: String = data.message.clone();
        let tags: Vec<Tag> = data.into();
        Self::new(Kind::ZapRequest, message, tags)
    }

    /// Create zap receipt event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[cfg(feature = "nip57")]
    pub fn zap_receipt<S1, S2>(bolt11: S1, preimage: Option<S2>, zap_request: &Event) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let mut tags: Vec<Tag> = vec![
            Tag::from_standardized_without_cell(TagStandard::Bolt11(bolt11.into())),
            Tag::from_standardized_without_cell(TagStandard::Description(zap_request.as_json())),
        ];

        // add preimage tag if provided
        if let Some(pre_image_tag) = preimage {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Preimage(
                pre_image_tag.into(),
            )))
        }

        // add e tag
        if let Some(tag) = zap_request
            .iter_tags()
            .find(|t| {
                t.kind()
                    == TagKind::SingleLetter(SingleLetterTag {
                        character: Alphabet::E,
                        uppercase: false,
                    })
            })
            .cloned()
        {
            tags.push(tag);
        }

        // add p tag
        if let Some(tag) = zap_request
            .iter_tags()
            .find(|t| {
                t.kind()
                    == TagKind::SingleLetter(SingleLetterTag {
                        character: Alphabet::P,
                        uppercase: false,
                    })
            })
            .cloned()
        {
            tags.push(tag);
        }

        // add P tag
        tags.push(Tag::from_standardized_without_cell(
            TagStandard::PublicKey {
                public_key: zap_request.author().clone(),
                relay_url: None,
                alias: None,
                uppercase: true,
            },
        ));

        Self::new(Kind::ZapReceipt, "", tags)
    }

    /// Create a badge definition event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::{EventBuilder, ImageDimensions, UncheckedUrl};
    ///
    /// let badge_id = String::from("nostr-sdk-test-badge");
    /// let name = Some(String::from("rust-nostr test badge"));
    /// let description = Some(String::from("This is a test badge"));
    /// let image_url = Some(UncheckedUrl::from("https://nostr.build/someimage/1337"));
    /// let image_size = Some(ImageDimensions::new(1024, 1024));
    /// let thumbs = vec![(
    ///     UncheckedUrl::from("https://nostr.build/somethumbnail/1337"),
    ///     Some(ImageDimensions::new(256, 256)),
    /// )];
    ///
    /// let event_builder =
    ///     EventBuilder::define_badge(badge_id, name, description, image_url, image_size, thumbs);
    /// ```
    pub fn define_badge<S>(
        badge_id: S,
        name: Option<S>,
        description: Option<S>,
        image: Option<UncheckedUrl>,
        image_dimensions: Option<ImageDimensions>,
        thumbnails: Vec<(UncheckedUrl, Option<ImageDimensions>)>,
    ) -> Self
    where
        S: Into<String>,
    {
        let mut tags: Vec<Tag> = Vec::new();

        // Set identifier tag
        tags.push(Tag::identifier(badge_id.into()));

        // Set name tag
        if let Some(name) = name {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Name(
                name.into(),
            )));
        }

        // Set description tag
        if let Some(description) = description {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Description(description.into()),
            ));
        }

        // Set image tag
        if let Some(image) = image {
            let image_tag = if let Some(dimensions) = image_dimensions {
                Tag::from_standardized_without_cell(TagStandard::Image(image, Some(dimensions)))
            } else {
                Tag::from_standardized_without_cell(TagStandard::Image(image, None))
            };
            tags.push(image_tag);
        }

        // Set thumbnail tags
        for (thumb, dimensions) in thumbnails.into_iter() {
            let thumb_tag = if let Some(dimensions) = dimensions {
                Tag::from_standardized_without_cell(TagStandard::Thumb(thumb, Some(dimensions)))
            } else {
                Tag::from_standardized_without_cell(TagStandard::Thumb(thumb, None))
            };
            tags.push(thumb_tag);
        }

        Self::new(Kind::BadgeDefinition, "", tags)
    }

    /// Create a badge award event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    pub fn award_badge<I>(badge_definition: &Event, awarded_pubkeys: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = Tag>,
    {
        let badge_id = badge_definition
            .iter_tags()
            .find_map(|t| match t.as_standardized() {
                Some(TagStandard::Identifier(id)) => Some(id),
                _ => None,
            })
            .ok_or(Error::NIP58(nip58::Error::IdentifierTagNotFound))?;

        // At least 1 tag
        let mut tags = Vec::with_capacity(1);

        // Add identity tag
        tags.push(Tag::from_standardized_without_cell(
            TagStandard::Coordinate {
                coordinate: Coordinate::new(Kind::BadgeDefinition, badge_definition.author())
                    .identifier(badge_id),
                relay_url: None,
            },
        ));

        // Add awarded pubkeys
        let ptags = awarded_pubkeys
            .into_iter()
            .filter(|p| p.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::P)));
        tags.extend(ptags);

        // Build event
        Ok(Self::new(Kind::BadgeAward, "", tags))
    }

    /// Create a profile badges event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    pub fn profile_badges(
        badge_definitions: Vec<Event>,
        badge_awards: Vec<Event>,
        pubkey_awarded: &PublicKey,
    ) -> Result<Self, Error> {
        if badge_definitions.len() != badge_awards.len() {
            return Err(Error::NIP58(nip58::Error::InvalidLength));
        }

        let badge_awards: Vec<Event> = nip58::filter_for_kind(badge_awards, &Kind::BadgeAward);
        if badge_awards.is_empty() {
            return Err(Error::NIP58(Nip58Error::InvalidKind));
        }

        for award in badge_awards.iter() {
            if !award.iter_tags().any(|t| match t.as_standardized() {
                Some(TagStandard::PublicKey { public_key, .. }) => public_key == pubkey_awarded,
                _ => false,
            }) {
                return Err(Error::NIP58(Nip58Error::BadgeAwardsLackAwardedPublicKey));
            }
        }

        let badge_definitions: Vec<Event> =
            nip58::filter_for_kind(badge_definitions, &Kind::BadgeDefinition);
        if badge_definitions.is_empty() {
            return Err(Error::NIP58(Nip58Error::InvalidKind));
        }

        // Add identifier `d` tag
        let id_tag: Tag = Tag::identifier("profile_badges");
        let mut tags: Vec<Tag> = vec![id_tag];

        let badge_definitions_identifiers = badge_definitions.into_iter().filter_map(|event| {
            let id: String = event.identifier()?.to_string();
            Some((event, id))
        });

        let badge_awards_identifiers = badge_awards.into_iter().filter_map(|event| {
            let (_, relay_url) = nip58::extract_awarded_public_key(event.tags(), pubkey_awarded)?;
            let relay_url = relay_url.clone();
            let (id, a_tag) = event.iter_tags().find_map(|t| match t.as_standardized() {
                Some(TagStandard::Coordinate { coordinate, .. }) => {
                    Some((coordinate.identifier.clone(), t.clone()))
                }
                _ => None,
            })?;
            Some((event, id, a_tag, relay_url))
        });

        // This collection has been filtered for the needed tags
        let users_badges = core::iter::zip(badge_definitions_identifiers, badge_awards_identifiers);

        for (badge_definition, badge_award) in users_badges {
            match (badge_definition, badge_award) {
                ((_, identifier), (_, badge_id, ..)) if badge_id != identifier => {
                    return Err(Error::NIP58(Nip58Error::MismatchedBadgeDefinitionOrAward));
                }
                ((_, identifier), (badge_award_event, badge_id, a_tag, relay_url))
                    if badge_id == identifier =>
                {
                    let badge_award_event_tag: Tag =
                        Tag::from_standardized_without_cell(TagStandard::Event {
                            event_id: badge_award_event.id(),
                            relay_url: relay_url.clone(),
                            marker: None,
                        });
                    tags.extend_from_slice(&[a_tag, badge_award_event_tag]);
                }
                _ => {}
            }
        }

        Ok(EventBuilder::new(Kind::ProfileBadges, "", tags))
    }

    /// Data Vending Machine - Job Request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn job_request<I>(kind: Kind, tags: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = Tag>,
    {
        if kind.is_job_request() {
            Ok(Self::new(kind, "", tags))
        } else {
            Err(Error::WrongKind {
                received: kind,
                expected: WrongKindError::Range(NIP90_JOB_REQUEST_RANGE),
            })
        }
    }

    /// Data Vending Machine - Job Result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn job_result(
        job_request: Event,
        amount_millisats: u64,
        bolt11: Option<String>,
    ) -> Result<Self, Error> {
        let kind: Kind = job_request.kind() + 1000;
        if kind.is_job_result() {
            let mut tags: Vec<Tag> = job_request
                .iter_tags()
                .filter_map(|t| {
                    if t.kind()
                        == TagKind::SingleLetter(SingleLetterTag {
                            character: Alphabet::I,
                            uppercase: false,
                        })
                    {
                        Some(t.clone())
                    } else {
                        None
                    }
                })
                .collect();
            tags.extend_from_slice(&[
                Tag::event(job_request.id()),
                Tag::public_key(job_request.author().clone()),
                Tag::from_standardized_without_cell(TagStandard::Request(job_request)),
                Tag::from_standardized_without_cell(TagStandard::Amount {
                    millisats: amount_millisats,
                    bolt11,
                }),
            ]);
            Ok(Self::new(kind, "", tags))
        } else {
            Err(Error::WrongKind {
                received: kind,
                expected: WrongKindError::Range(NIP90_JOB_RESULT_RANGE),
            })
        }
    }

    /// Data Vending Machine - Job Feedback
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn job_feedback(
        job_request: &Event,
        status: DataVendingMachineStatus,
        extra_info: Option<String>,
        amount_millisats: u64,
        bolt11: Option<String>,
        payload: Option<String>,
    ) -> Self {
        let tags = [
            Tag::from_standardized_without_cell(TagStandard::DataVendingMachineStatus {
                status,
                extra_info,
            }),
            Tag::event(job_request.id()),
            Tag::public_key(job_request.author().clone()),
            Tag::from_standardized_without_cell(TagStandard::Amount {
                millisats: amount_millisats,
                bolt11,
            }),
        ];
        Self::new(Kind::JobFeedback, payload.unwrap_or_default(), tags)
    }

    /// File metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/94.md>
    #[inline]
    pub fn file_metadata<S>(description: S, metadata: FileMetadata) -> Self
    where
        S: Into<String>,
    {
        let tags: Vec<Tag> = metadata.into();
        Self::new(Kind::FileMetadata, description.into(), tags)
    }

    /// HTTP Auth
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/98.md>
    #[inline]
    pub fn http_auth(data: HttpData) -> Self {
        let tags: Vec<Tag> = data.into();
        Self::new(Kind::HttpAuth, "", tags)
    }

    /// Set stall data
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/15.md>
    #[inline]
    pub fn stall_data(data: StallData) -> Self {
        let content: String = data.as_json();
        let tags: Vec<Tag> = data.into();
        Self::new(Kind::SetStall, content, tags)
    }

    /// Set product data
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/15.md>
    #[inline]
    pub fn product_data(data: ProductData) -> Self {
        let content: String = data.as_json();
        let tags: Vec<Tag> = data.into();
        Self::new(Kind::SetProduct, content, tags)
    }

    /// Seal
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(all(feature = "std", feature = "nip59"))]
    pub fn seal(
        sender_keys: &Keys,
        receiver_pubkey: &PublicKey,
        rumor: UnsignedEvent,
    ) -> Result<Self, Error> {
        let content = nip44::encrypt(
            sender_keys.secret_key()?,
            receiver_pubkey,
            rumor.as_json(),
            Version::default(),
        )?;
        Ok(Self::new(Kind::Seal, content, [])
            .custom_created_at(Timestamp::tweaked(nip59::RANGE_RANDOM_TIMESTAMP_TWEAK)))
    }

    /// Gift Wrap from seal
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[cfg(all(feature = "std", feature = "nip59"))]
    pub fn gift_wrap_from_seal(
        receiver: &PublicKey,
        seal: &Event,
        expiration: Option<Timestamp>,
    ) -> Result<Event, Error> {
        if seal.kind != Kind::Seal {
            return Err(Error::WrongKind {
                received: seal.kind,
                expected: WrongKindError::Single(Kind::Seal),
            });
        }

        let keys: Keys = Keys::generate();
        let content: String = nip44::encrypt(
            keys.secret_key()?,
            receiver,
            seal.as_json(),
            Version::default(),
        )?;

        let mut tags: Vec<Tag> = Vec::with_capacity(1 + usize::from(expiration.is_some()));
        tags.push(Tag::public_key(receiver.clone()));

        if let Some(timestamp) = expiration {
            tags.push(Tag::expiration(timestamp));
        }

        Self::new(Kind::GiftWrap, content, tags)
            .custom_created_at(Timestamp::tweaked(nip59::RANGE_RANDOM_TIMESTAMP_TWEAK))
            .to_event(&keys)
    }

    /// Gift Wrap
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(all(feature = "std", feature = "nip59"))]
    pub fn gift_wrap(
        sender_keys: &Keys,
        receiver: &PublicKey,
        rumor: UnsignedEvent,
        expiration: Option<Timestamp>,
    ) -> Result<Event, Error> {
        let seal: Event = Self::seal(sender_keys, receiver, rumor)?.to_event(sender_keys)?;
        Self::gift_wrap_from_seal(receiver, &seal, expiration)
    }

    /// GiftWrapped Sealed Direct message
    #[inline]
    #[cfg(feature = "nip59")]
    #[deprecated(since = "0.31.0", note = "Use `private_msg_rumor` instead.")]
    pub fn sealed_direct<S>(receiver: PublicKey, message: S) -> Self
    where
        S: Into<String>,
    {
        Self::private_msg_rumor(receiver, message)
    }

    /// Private Direct message rumor
    ///
    /// <div class="warning">
    /// This constructor compose ONLY the rumor for the private direct message!
    /// NOT USE THIS IF YOU DON'T KNOW WHAT YOU ARE DOING!
    /// </div>
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub fn private_msg_rumor<S>(receiver: PublicKey, message: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(
            Kind::PrivateDirectMessage,
            message,
            [Tag::public_key(receiver)],
        )
    }

    /// Mute list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn mute_list(list: MuteList) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::MuteList, "", tags)
    }

    /// Pinned notes
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn pinned_notes<I>(ids: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        Self::new(Kind::PinList, "", ids.into_iter().map(Tag::event))
    }

    /// Bookmarks
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn bookmarks(list: Bookmarks) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::Bookmarks, "", tags)
    }

    /// Communities
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn communities<I>(communities: I) -> Self
    where
        I: IntoIterator<Item = Coordinate>,
    {
        Self::new(
            Kind::Communities,
            "",
            communities.into_iter().map(Tag::from),
        )
    }

    /// Public chats
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn public_chats<I>(chat: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        Self::new(Kind::PublicChats, "", chat.into_iter().map(Tag::event))
    }

    /// Blocked relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn blocked_relays<I>(relay: I) -> Self
    where
        I: IntoIterator<Item = UncheckedUrl>,
    {
        Self::new(
            Kind::BlockedRelays,
            "",
            relay
                .into_iter()
                .map(|r| Tag::from_standardized_without_cell(TagStandard::Relay(r))),
        )
    }

    /// Search relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn search_relays<I>(relay: I) -> Self
    where
        I: IntoIterator<Item = UncheckedUrl>,
    {
        Self::new(
            Kind::SearchRelays,
            "",
            relay
                .into_iter()
                .map(|r| Tag::from_standardized_without_cell(TagStandard::Relay(r))),
        )
    }

    /// Interests
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn interests(list: Interests) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::Interests, "", tags)
    }

    /// Emojis
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn emojis(list: Emojis) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::Emojis, "", tags)
    }

    /// Follow sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn follow_sets<I>(public_keys: I) -> Self
    where
        I: IntoIterator<Item = PublicKey>,
    {
        Self::new(
            Kind::FollowSets,
            "",
            public_keys.into_iter().map(Tag::public_key),
        )
    }

    /// Relay sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn relay_sets<I>(relay: I) -> Self
    where
        I: IntoIterator<Item = UncheckedUrl>,
    {
        Self::new(
            Kind::RelaySets,
            "",
            relay
                .into_iter()
                .map(|r| Tag::from_standardized_without_cell(TagStandard::Relay(r))),
        )
    }

    /// Bookmark sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn bookmarks_sets(list: Bookmarks) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::BookmarkSets, "", tags)
    }

    /// Article Curation sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn articles_curation_sets(list: ArticlesCuration) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::ArticlesCurationSets, "", tags)
    }

    /// Videos Curation sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn videos_curation_sets<I>(video: I) -> Self
    where
        I: IntoIterator<Item = Coordinate>,
    {
        Self::new(
            Kind::VideosCurationSets,
            "",
            video.into_iter().map(Tag::from),
        )
    }

    /// Emoji sets
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    pub fn emoji_sets<I>(emoji: I) -> Self
    where
        I: IntoIterator<Item = (String, UncheckedUrl)>,
    {
        let tags = emoji.into_iter().map(|(s, url)| {
            Tag::from_standardized_without_cell(TagStandard::Emoji { shortcode: s, url })
        });
        Self::new(Kind::EmojiSets, "", tags)
    }

    /// Label
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    pub fn label<S, I>(namespace: S, labels: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = String>,
    {
        let namespace: String = namespace.into();
        let labels: Vec<String> = labels.into_iter().chain([namespace.clone()]).collect();
        Self::new(
            Kind::Label,
            "",
            [
                Tag::from_standardized_without_cell(TagStandard::LabelNamespace(namespace)),
                Tag::from_standardized_without_cell(TagStandard::Label(labels)),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    use core::str::FromStr;

    use super::*;
    #[cfg(feature = "std")]
    use crate::SecretKey;

    #[test]
    #[cfg(feature = "std")]
    fn round_trip() {
        let keys = Keys::new(
            SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap(),
        );

        let event = EventBuilder::text_note("hello", [])
            .to_event(&keys)
            .unwrap();

        let serialized = event.as_json();
        let deserialized = Event::from_json(serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    #[cfg(all(feature = "std", feature = "nip04"))]
    fn test_encrypted_direct_msg() {
        let sender_keys = Keys::new(
            SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap(),
        );
        let receiver_keys = Keys::new(
            SecretKey::from_str("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap(),
        );

        let content = "Mercury, the Winged Messenger";
        let event = EventBuilder::encrypted_direct_msg(
            &sender_keys,
            receiver_keys.public_key(),
            content,
            None,
        )
        .unwrap()
        .to_event(&sender_keys)
        .unwrap();

        event.verify().unwrap();
    }

    #[test]
    #[cfg(feature = "nip57")]
    fn test_zap_event_builder() {
        let bolt11 = "lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0";
        let preimage = Some("5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f");
        let zap_request_json = String::from("{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}");
        let zap_request_event: Event = Event::from_json(zap_request_json).unwrap();
        let event_builder = EventBuilder::zap_receipt(bolt11, preimage, &zap_request_event);

        assert_eq!(6, event_builder.tags.len());

        let has_preimage_tag = event_builder
            .tags
            .clone()
            .iter()
            .find(|t| t.kind() == TagKind::Preimage)
            .is_some();

        assert_eq!(true, has_preimage_tag);
    }

    #[test]
    #[cfg(feature = "nip57")]
    fn test_zap_event_builder_without_preimage() {
        let bolt11 = "lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0";
        let preimage: Option<&str> = None;
        let zap_request_json = String::from("{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}");
        let zap_request_event = Event::from_json(zap_request_json).unwrap();
        let event_builder = EventBuilder::zap_receipt(bolt11, preimage, &zap_request_event);

        assert_eq!(5, event_builder.tags.len());
        let has_preimage_tag = event_builder
            .tags
            .clone()
            .iter()
            .find(|t| t.kind() == TagKind::Preimage)
            .is_some();

        assert_eq!(false, has_preimage_tag);
    }

    #[test]
    fn test_badge_definition_event_builder_badge_id_only() {
        let badge_id = String::from("bravery");
        let event_builder =
            EventBuilder::define_badge(badge_id, None, None, None, None, Vec::new());

        let has_id = event_builder
            .tags
            .clone()
            .iter()
            .find(|t| t.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::D)))
            .is_some();
        assert_eq!(true, has_id);

        assert_eq!(Kind::BadgeDefinition, event_builder.kind);
    }

    #[test]
    fn test_badge_definition_event_builder_full() {
        let badge_id = String::from("bravery");
        let name = Some(String::from("Bravery"));
        let description = Some(String::from("Brave pubkey"));
        let image_url = Some(UncheckedUrl::from("https://nostr.build/someimage/1337"));
        let image_size = Some(ImageDimensions::new(1024, 1024));
        let thumbs = vec![(
            UncheckedUrl::from("https://nostr.build/somethumbnail/1337"),
            Some(ImageDimensions::new(256, 256)),
        )];

        let event_builder =
            EventBuilder::define_badge(badge_id, name, description, image_url, image_size, thumbs);

        let has_id = event_builder
            .tags
            .clone()
            .iter()
            .find(|t| t.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::D)))
            .is_some();
        assert_eq!(true, has_id);

        assert_eq!(Kind::BadgeDefinition, event_builder.kind);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_badge_award_event_builder() {
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
            Tag::from_standardized(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from_str("wss://nostr.oxtr.dev").unwrap()),
                alias: None,
                uppercase: false,
            }),
            Tag::from_standardized(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df",
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from_str("wss://nostr.oxtr.dev").unwrap()),
                alias: None,
                uppercase: false,
            }),
        ];
        let event_builder: Event =
            EventBuilder::award_badge(&badge_definition_event, awarded_pubkeys)
                .unwrap()
                .to_event(&keys)
                .unwrap();

        assert_eq!(event_builder.kind(), Kind::BadgeAward);
        assert_eq!(event_builder.content(), "");
        assert_eq!(event_builder.tags(), example_event.tags());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_profile_badges() {
        // The pubkey used for profile badges event
        let keys = Keys::generate();
        let pub_key = keys.public_key();

        // Create badge 1
        let badge_one_keys = Keys::generate();
        let badge_one_pubkey = badge_one_keys.public_key();
        let relay_url = UncheckedUrl::from_str("wss://nostr.oxtr.dev").unwrap();

        let awarded_pubkeys = vec![
            Tag::from_standardized(TagStandard::PublicKey {
                public_key: pub_key.clone(),
                relay_url: Some(relay_url.clone()),
                alias: None,
                uppercase: false,
            }),
            Tag::from_standardized(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df",
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from_str("wss://nostr.oxtr.dev").unwrap()),
                alias: None,
                uppercase: false,
            }),
        ];
        let bravery_badge_event =
            self::EventBuilder::define_badge("bravery", None, None, None, None, Vec::new())
                .to_event(&badge_one_keys)
                .unwrap();
        let bravery_badge_award =
            self::EventBuilder::award_badge(&bravery_badge_event, awarded_pubkeys.clone())
                .unwrap()
                .to_event(&badge_one_keys)
                .unwrap();

        // Badge 2
        let badge_two_keys = Keys::generate();
        let badge_two_pubkey = badge_two_keys.public_key();

        let honor_badge_event =
            self::EventBuilder::define_badge("honor", None, None, None, None, Vec::new())
                .to_event(&badge_two_keys)
                .unwrap();
        let honor_badge_award =
            self::EventBuilder::award_badge(&honor_badge_event, awarded_pubkeys.clone())
                .unwrap()
                .to_event(&badge_two_keys)
                .unwrap();

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
            bravery_badge_award.id().to_string(),
            badge_two_pubkey.to_string(),
            honor_badge_award.id().to_string(),
        );
        let example_event: Event = serde_json::from_str(&example_event_json).unwrap();

        let badge_definitions = vec![bravery_badge_event, honor_badge_event];
        let badge_awards = vec![bravery_badge_award, honor_badge_award];
        let profile_badges =
            EventBuilder::profile_badges(badge_definitions, badge_awards, &pub_key)
                .unwrap()
                .to_event(&keys)
                .unwrap();

        assert_eq!(profile_badges.kind(), Kind::ProfileBadges);
        assert_eq!(profile_badges.tags(), example_event.tags());
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn builder_to_event(bh: &mut Bencher) {
        let keys = Keys::generate();
        bh.iter(|| {
            black_box(EventBuilder::text_note("hello", []).to_event(&keys)).unwrap();
        });
    }
}
