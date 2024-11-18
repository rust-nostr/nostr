// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Event builder

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::{CryptoRng, Rng};
use bitcoin::secp256k1::{Secp256k1, Signing, Verification};
use serde_json::{json, Value};

#[cfg(all(feature = "std", feature = "nip04", feature = "nip46"))]
use crate::nips::nip46::Message as NostrConnectMessage;
use crate::prelude::*;

/// Wrong kind error
#[derive(Debug)]
pub enum WrongKindError {
    /// Single kind
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
    /// Signer error
    Signer(SignerError),
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
    /// NIP59 error
    #[cfg(all(feature = "std", feature = "nip59"))]
    NIP59(nip59::Error),
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
            Self::Signer(e) => write!(f, "{e}"),
            Self::Unsigned(e) => write!(f, "{e}"),
            #[cfg(feature = "nip03")]
            Self::OpenTimestamps(e) => write!(f, "{e}"),
            #[cfg(feature = "nip04")]
            Self::NIP04(e) => write!(f, "{e}"),
            #[cfg(all(feature = "std", feature = "nip44"))]
            Self::NIP44(e) => write!(f, "{e}"),
            Self::NIP58(e) => write!(f, "{e}"),
            #[cfg(all(feature = "std", feature = "nip59"))]
            Self::NIP59(e) => write!(f, "{e}"),
            Self::WrongKind { received, expected } => {
                write!(f, "Wrong kind: received={received}, expected={expected}")
            }
        }
    }
}

impl From<SignerError> for Error {
    fn from(e: SignerError) -> Self {
        Self::Signer(e)
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

#[cfg(all(feature = "std", feature = "nip59"))]
impl From<nip59::Error> for Error {
    fn from(e: nip59::Error) -> Self {
        Self::NIP59(e)
    }
}

/// Event builder
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventBuilder {
    kind: Kind,
    tags: Vec<Tag>,
    content: String,
    custom_created_at: Option<Timestamp>,
    /// POW difficulty
    pow: Option<u8>,
}

impl EventBuilder {
    /// New event builder
    #[inline]
    pub fn new<S>(kind: Kind, content: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            kind,
            tags: Vec::new(),
            content: content.into(),
            custom_created_at: None,
            pow: None,
        }
    }

    /// Add tags
    #[deprecated(since = "0.37.0", note = "Use `tags` instead")]
    pub fn add_tags<I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = Tag>,
    {
        self.tags.extend(tags);
        self
    }

    /// Add tag
    #[inline]
    pub fn tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add tags
    ///
    /// This method extend the current tags (if any).
    #[inline]
    pub fn tags<I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = Tag>,
    {
        self.tags.extend(tags);
        self
    }

    /// Set a custom `created_at` UNIX timestamp
    #[inline]
    pub fn custom_created_at(mut self, created_at: Timestamp) -> Self {
        self.custom_created_at = Some(created_at);
        self
    }

    /// Set POW difficulty
    ///
    /// Only values `> 0` are accepted!
    #[inline]
    pub fn pow(mut self, difficulty: u8) -> Self {
        if difficulty > 0 {
            self.pow = Some(difficulty);
        }
        self
    }

    /// Build unsigned event
    pub fn build_with_ctx<T>(self, supplier: &T, pubkey: PublicKey) -> UnsignedEvent
    where
        T: TimeSupplier,
    {
        // Check if should be POW
        match self.pow {
            Some(difficulty) if difficulty > 0 => {
                let mut nonce: u128 = 0;
                let mut tags: Vec<Tag> = self.tags;

                tags.reserve_exact(1);

                loop {
                    nonce += 1;

                    tags.push(Tag::pow(nonce, difficulty));

                    let created_at: Timestamp = self
                        .custom_created_at
                        .unwrap_or_else(|| Timestamp::now_with_supplier(supplier));
                    let id: EventId =
                        EventId::new(&pubkey, &created_at, &self.kind, &tags, &self.content);

                    if id.check_pow(difficulty) {
                        return UnsignedEvent {
                            id: Some(id),
                            pubkey,
                            created_at,
                            kind: self.kind,
                            tags: Tags::new(tags),
                            content: self.content,
                        };
                    }

                    tags.pop();
                }
            }
            // No POW difficulty set OR difficulty == 0
            _ => {
                let mut unsigned: UnsignedEvent = UnsignedEvent {
                    id: None,
                    pubkey,
                    created_at: self
                        .custom_created_at
                        .unwrap_or_else(|| Timestamp::now_with_supplier(supplier)),
                    kind: self.kind,
                    tags: Tags::new(self.tags),
                    content: self.content,
                };
                unsigned.ensure_id();
                unsigned
            }
        }
    }

    /// Build unsigned event
    #[inline]
    #[cfg(feature = "std")]
    pub fn build(self, pubkey: PublicKey) -> UnsignedEvent {
        self.build_with_ctx(&Instant::now(), pubkey)
    }

    /// Build, sign and return [`Event`]
    ///
    /// Shortcut for `builder.build(public_key).sign(signer)`.
    #[inline]
    #[cfg(feature = "std")]
    pub async fn sign<T>(self, signer: &T) -> Result<Event, Error>
    where
        T: NostrSigner,
    {
        let public_key: PublicKey = signer.get_public_key().await?;
        Ok(self.build(public_key).sign(signer).await?)
    }

    /// Build, sign and return [`Event`] using [`Keys`] signer
    #[inline]
    #[cfg(feature = "std")]
    pub fn sign_with_keys(self, keys: &Keys) -> Result<Event, Error> {
        self.sign_with_ctx(&SECP256K1, &mut OsRng, &Instant::now(), keys)
    }

    /// Build, sign and return [`Event`] using [`Keys`] signer
    pub fn sign_with_ctx<C, R, T>(
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
        let pubkey: PublicKey = keys.public_key();
        Ok(self
            .build_with_ctx(supplier, pubkey)
            .sign_with_ctx(secp, rng, keys)?)
    }

    /// Profile metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::prelude::*;
    ///
    /// let metadata = Metadata::new()
    ///     .name("username")
    ///     .display_name("My Username")
    ///     .about("Description")
    ///     .picture(Url::parse("https://example.com/avatar.png").unwrap())
    ///     .nip05("username@example.com")
    ///     .lud16("pay@yukikishimoto.com");
    ///
    /// let builder = EventBuilder::metadata(&metadata);
    /// ```
    #[inline]
    pub fn metadata(metadata: &Metadata) -> Self {
        Self::new(Kind::Metadata, metadata.as_json())
    }

    /// Relay list metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    pub fn relay_list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Url, Option<RelayMetadata>)>,
    {
        let tags = iter
            .into_iter()
            .map(|(url, metadata)| Tag::relay_metadata(url, metadata));
        Self::new(Kind::RelayList, "").tags(tags)
    }

    /// Text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::EventBuilder;
    ///
    /// let builder = EventBuilder::text_note("My first text note from rust-nostr!");
    /// ```
    #[inline]
    pub fn text_note<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::TextNote, content)
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
                    event_id: root.id,
                    relay_url: relay_url.clone(),
                    marker: Some(Marker::Root),
                    public_key: Some(root.pubkey),
                    uppercase: false,
                }));
                tags.push(Tag::public_key(root.pubkey));

                // Add others `p` tags
                tags.extend(
                    root.tags
                        .iter()
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
                    event_id: reply_to.id,
                    relay_url: relay_url.clone(),
                    marker: Some(Marker::Root),
                    public_key: Some(reply_to.pubkey),
                    uppercase: false,
                }));
            }
        }

        // Add `e` and `p` tag of event author
        tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
            event_id: reply_to.id,
            relay_url,
            marker: Some(Marker::Reply),
            public_key: Some(reply_to.pubkey),
            uppercase: false,
        }));
        tags.push(Tag::public_key(reply_to.pubkey));

        // Add others `p` tags of reply_to event
        tags.extend(
            reply_to
                .tags
                .iter()
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
        Self::new(Kind::TextNote, content).tags(tags)
    }

    /// Comment
    ///
    /// If no `root` is passed, the `comment_to` will be used for root `e` tag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/22.md>
    pub fn comment<S>(
        content: S,
        comment_to: &Event,
        root: Option<&Event>,
        relay_url: Option<UncheckedUrl>,
    ) -> Self
    where
        S: Into<String>,
    {
        // The added tags will be at least 4
        let mut tags: Vec<Tag> = Vec::with_capacity(4);

        // Add `A`, `E` and `K` tag of **root** event
        if let Some(root) = root {
            // If event has coordinate, add it to tags otherwise push the event ID
            match root.coordinate() {
                Some(coordinate) => {
                    tags.push(Tag::from_standardized_without_cell(
                        TagStandard::Coordinate {
                            coordinate,
                            relay_url: relay_url.clone(),
                            uppercase: true,
                        },
                    ));
                }
                None => {
                    // ID and author
                    tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
                        event_id: root.id,
                        relay_url: relay_url.clone(),
                        marker: None,
                        public_key: Some(root.pubkey),
                        uppercase: true,
                    }));
                }
            }

            // Kind
            tags.push(Tag::from_standardized_without_cell(TagStandard::Kind {
                kind: root.kind,
                uppercase: true,
            }));

            // Add others `p` tags
            tags.extend(
                root.tags
                    .iter()
                    .filter(|t| {
                        t.kind()
                            == TagKind::SingleLetter(SingleLetterTag {
                                character: Alphabet::P,
                                uppercase: false,
                            })
                    })
                    .cloned(),
            );
        } else {
            match comment_to.coordinate() {
                Some(coordinate) => {
                    tags.push(Tag::from_standardized_without_cell(
                        TagStandard::Coordinate {
                            coordinate,
                            relay_url: relay_url.clone(),
                            uppercase: true,
                        },
                    ));
                }
                None => {
                    // ID and author
                    tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
                        event_id: comment_to.id,
                        relay_url: relay_url.clone(),
                        marker: None,
                        public_key: Some(comment_to.pubkey),
                        uppercase: true,
                    }));
                }
            }

            // Kind
            tags.push(Tag::from_standardized_without_cell(TagStandard::Kind {
                kind: comment_to.kind,
                uppercase: true,
            }));
        }

        // Add `a` tag (if event has it)
        if let Some(coordinate) = comment_to.coordinate() {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Coordinate {
                    coordinate,
                    relay_url: relay_url.clone(),
                    uppercase: false, // <--- Same as root event but lowercase
                },
            ));
        }

        // Add `e` tag of event author
        tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
            event_id: comment_to.id,
            relay_url,
            marker: None,
            public_key: Some(comment_to.pubkey),
            uppercase: false,
        }));

        // Add `k` tag of event kind
        tags.push(Tag::from_standardized_without_cell(TagStandard::Kind {
            kind: comment_to.kind,
            uppercase: false,
        }));

        // Add others `p` tags of comment_to event
        // TODO: avoid `p` tag duplicates (are added also before from root event)
        tags.extend(
            comment_to
                .tags
                .iter()
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
        Self::new(Kind::Comment, content).tags(tags)
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
    /// let builder = EventBuilder::long_form_text_note("My first text note from rust-nostr!");
    /// ```
    #[inline]
    pub fn long_form_text_note<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::LongFormTextNote, content)
    }

    /// Contact/Follow list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
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
        Self::new(Kind::ContactList, "").tags(tags)
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
        Ok(
            Self::new(Kind::OpenTimestamps, ots).tags([Tag::from_standardized_without_cell(
                TagStandard::Event {
                    event_id,
                    relay_url,
                    marker: None,
                    public_key: None,
                    uppercase: false,
                },
            )]),
        )
    }

    /// Repost
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/18.md>
    pub fn repost(event: &Event, relay_url: Option<UncheckedUrl>) -> Self {
        if event.kind == Kind::TextNote {
            Self::new(Kind::Repost, event.as_json()).tags([
                Tag::from_standardized_without_cell(TagStandard::Event {
                    event_id: event.id,
                    relay_url,
                    marker: None,
                    // NOTE: not add public key since it's already included as `p` tag
                    public_key: None,
                    uppercase: false,
                }),
                Tag::public_key(event.pubkey),
            ])
        } else {
            Self::new(Kind::GenericRepost, event.as_json()).tags([
                Tag::from_standardized_without_cell(TagStandard::Event {
                    event_id: event.id,
                    relay_url,
                    marker: None,
                    // NOTE: not add public key since it's already included as `p` tag
                    public_key: None,
                    uppercase: false,
                }),
                Tag::public_key(event.pubkey),
                Tag::from_standardized_without_cell(TagStandard::Kind {
                    kind: event.kind,
                    uppercase: false,
                }),
            ])
        }
    }

    /// Event deletion
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    #[inline]
    pub fn delete<I, T>(ids: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<EventIdOrCoordinate>,
    {
        Self::delete_with_reason(ids, "")
    }

    /// Event deletion with reason
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
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
        Self::new(Kind::EventDeletion, reason.into()).tags(tags)
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    #[inline]
    pub fn reaction<S>(event: &Event, reaction: S) -> Self
    where
        S: Into<String>,
    {
        Self::reaction_extended(event.id, event.pubkey, Some(event.kind), reaction)
    }

    /// Add reaction (like/upvote, dislike/downvote or emoji) to an event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub fn reaction_extended<S>(
        event_id: EventId,
        public_key: PublicKey,
        kind: Option<Kind>,
        reaction: S,
    ) -> Self
    where
        S: Into<String>,
    {
        let mut tags: Vec<Tag> = Vec::with_capacity(2 + usize::from(kind.is_some()));

        tags.push(Tag::event(event_id));
        tags.push(Tag::public_key(public_key));

        if let Some(kind) = kind {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Kind {
                kind,
                uppercase: false,
            }));
        }

        Self::new(Kind::Reaction, reaction).tags(tags)
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub fn channel(metadata: &Metadata) -> Self {
        Self::new(Kind::ChannelCreation, metadata.as_json())
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
        Self::new(Kind::ChannelMetadata, metadata.as_json()).tags([
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: channel_id,
                relay_url: relay_url.map(|u| u.into()),
                marker: None,
                public_key: None,
                uppercase: false,
            }),
        ])
    }

    /// Channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub fn channel_msg<S>(channel_id: EventId, relay_url: Url, content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::ChannelMessage, content).tags([Tag::from_standardized_without_cell(
            TagStandard::Event {
                event_id: channel_id,
                relay_url: Some(relay_url.into()),
                marker: Some(Marker::Root),
                public_key: None,
                uppercase: false,
            },
        )])
    }

    /// Hide message
    ///
    /// The `message_id` must be the [`EventId`] of the kind `42`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub fn hide_channel_msg<S>(message_id: EventId, reason: Option<S>) -> Self
    where
        S: Into<String>,
    {
        let content: Value = json!({
            "reason": reason.map(|s| s.into()).unwrap_or_default(),
        });

        Self::new(Kind::ChannelHideMessage, content.to_string()).tag(Tag::event(message_id))
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

        Self::new(Kind::ChannelMuteUser, content.to_string()).tag(Tag::public_key(public_key))
    }

    /// Authentication of clients to relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn auth<S>(challenge: S, relay: Url) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::Authentication, "").tags([
            Tag::from_standardized_without_cell(TagStandard::Challenge(challenge.into())),
            Tag::from_standardized_without_cell(TagStandard::Relay(relay.into())),
        ])
    }

    /// Nostr Connect / Nostr Remote Signing
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
            nip04::encrypt(sender_keys.secret_key(), &receiver_pubkey, msg.as_json())?,
        )
        .tag(Tag::public_key(receiver_pubkey)))
    }

    /// Live Event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    #[inline]
    pub fn live_event(live_event: LiveEvent) -> Self {
        let tags: Vec<Tag> = live_event.into();
        Self::new(Kind::LiveEvent, "").tags(tags)
    }

    /// Live Event Message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/53.md>
    pub fn live_event_msg<S>(
        live_event_id: S,
        live_event_host: PublicKey,
        content: S,
        relay_url: Option<Url>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::LiveEventMessage, content).tag(Tag::from_standardized_without_cell(
            TagStandard::Coordinate {
                coordinate: Coordinate::new(Kind::LiveEvent, live_event_host)
                    .identifier(live_event_id),
                relay_url: relay_url.map(|u| u.into()),
                uppercase: false,
            },
        ))
    }

    /// Reporting
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    pub fn report<I, S>(tags: I, content: S) -> Self
    where
        I: IntoIterator<Item = Tag>,
        S: Into<String>,
    {
        Self::new(Kind::Reporting, content).tags(tags)
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
    /// # let relays = [Url::parse("wss://relay.damus.io").unwrap()];
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
        Self::new(Kind::ZapRequest, message).tags(tags)
    }

    /// Zap Receipt
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
            .tags
            .iter()
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

        // add a tag
        if let Some(tag) = zap_request
            .tags
            .iter()
            .find(|t| {
                t.kind()
                    == TagKind::SingleLetter(SingleLetterTag {
                        character: Alphabet::A,
                        uppercase: false,
                    })
            })
            .cloned()
        {
            tags.push(tag);
        }

        // add p tag
        if let Some(tag) = zap_request
            .tags
            .iter()
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
                public_key: zap_request.pubkey,
                relay_url: None,
                alias: None,
                uppercase: true,
            },
        ));

        Self::new(Kind::ZapReceipt, "").tags(tags)
    }

    /// Badge definition
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr::prelude::*;
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

        Self::new(Kind::BadgeDefinition, "").tags(tags)
    }

    /// Badge award
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/58.md>
    pub fn award_badge<I>(badge_definition: &Event, awarded_public_keys: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let badge_id = badge_definition
            .tags
            .iter()
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
                coordinate: Coordinate::new(Kind::BadgeDefinition, badge_definition.pubkey)
                    .identifier(badge_id),
                relay_url: None,
                uppercase: false,
            },
        ));

        // Add awarded public keys
        tags.extend(awarded_public_keys.into_iter().map(Tag::public_key));

        // Build event
        Ok(Self::new(Kind::BadgeAward, "").tags(tags))
    }

    /// Profile badges
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
            return Err(Error::NIP58(nip58::Error::InvalidKind));
        }

        for award in badge_awards.iter() {
            if !award.tags.iter().any(|t| match t.as_standardized() {
                Some(TagStandard::PublicKey { public_key, .. }) => public_key == pubkey_awarded,
                _ => false,
            }) {
                return Err(Error::NIP58(nip58::Error::BadgeAwardsLackAwardedPublicKey));
            }
        }

        let badge_definitions: Vec<Event> =
            nip58::filter_for_kind(badge_definitions, &Kind::BadgeDefinition);
        if badge_definitions.is_empty() {
            return Err(Error::NIP58(nip58::Error::InvalidKind));
        }

        // Add identifier `d` tag
        let id_tag: Tag = Tag::identifier("profile_badges");
        let mut tags: Vec<Tag> = vec![id_tag];

        let badge_definitions_identifiers = badge_definitions.iter().filter_map(|event| {
            let id: &str = event.tags.identifier()?;
            Some((event, id))
        });

        let badge_awards_identifiers = badge_awards.iter().filter_map(|event| {
            let (_, relay_url) =
                nip58::extract_awarded_public_key(event.tags.as_slice(), pubkey_awarded)?;
            let (id, a_tag) = event.tags.iter().find_map(|t| match t.as_standardized() {
                Some(TagStandard::Coordinate { coordinate, .. }) => {
                    Some((&coordinate.identifier, t))
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
                    return Err(Error::NIP58(nip58::Error::MismatchedBadgeDefinitionOrAward));
                }
                ((_, identifier), (badge_award_event, badge_id, a_tag, relay_url))
                    if badge_id == identifier =>
                {
                    let badge_award_event_tag: Tag =
                        Tag::from_standardized_without_cell(TagStandard::Event {
                            event_id: badge_award_event.id,
                            relay_url: relay_url.clone(),
                            marker: None,
                            public_key: None,
                            uppercase: false,
                        });
                    tags.extend_from_slice(&[a_tag.clone(), badge_award_event_tag]);
                }
                _ => {}
            }
        }

        Ok(EventBuilder::new(Kind::ProfileBadges, "").tags(tags))
    }

    /// Data Vending Machine (DVM) - Job Request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn job_request(kind: Kind) -> Result<Self, Error> {
        if !kind.is_job_request() {
            return Err(Error::WrongKind {
                received: kind,
                expected: WrongKindError::Range(NIP90_JOB_REQUEST_RANGE),
            });
        }

        Ok(Self::new(kind, ""))
    }

    /// Data Vending Machine (DVM) - Job Result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn job_result<S>(
        job_request: Event,
        payload: S,
        millisats: u64,
        bolt11: Option<String>,
    ) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let kind: Kind = job_request.kind + 1000;

        // Check if Job Result kind
        if !kind.is_job_result() {
            return Err(Error::WrongKind {
                received: kind,
                expected: WrongKindError::Range(NIP90_JOB_RESULT_RANGE),
            });
        }

        let mut tags: Vec<Tag> = job_request
            .tags
            .iter()
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
            Tag::event(job_request.id),
            Tag::public_key(job_request.pubkey),
            Tag::from_standardized_without_cell(TagStandard::Request(job_request)),
            Tag::from_standardized_without_cell(TagStandard::Amount { millisats, bolt11 }),
        ]);

        Ok(Self::new(kind, payload).tags(tags))
    }

    /// Data Vending Machine (DVM) - Job Feedback
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn job_feedback(data: JobFeedbackData) -> Self {
        let mut tags: Vec<Tag> = Vec::with_capacity(3);

        tags.push(Tag::event(data.job_request_id));
        tags.push(Tag::public_key(data.customer_public_key));
        tags.push(Tag::from_standardized_without_cell(
            TagStandard::DataVendingMachineStatus {
                status: data.status,
                extra_info: data.extra_info,
            },
        ));

        if let Some(millisats) = data.amount_msat {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Amount {
                millisats,
                bolt11: data.bolt11,
            }));
        }

        Self::new(Kind::JobFeedback, data.payload.unwrap_or_default()).tags(tags)
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
        Self::new(Kind::FileMetadata, description.into()).tags(tags)
    }

    /// HTTP Auth
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/98.md>
    #[inline]
    pub fn http_auth(data: HttpData) -> Self {
        let tags: Vec<Tag> = data.into();
        Self::new(Kind::HttpAuth, "").tags(tags)
    }

    /// Set stall data
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/15.md>
    #[inline]
    pub fn stall_data(data: StallData) -> Self {
        let content: String = data.as_json();
        let tags: Vec<Tag> = data.into();
        Self::new(Kind::SetStall, content).tags(tags)
    }

    /// Set product data
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/15.md>
    #[inline]
    pub fn product_data(data: ProductData) -> Self {
        let content: String = data.as_json();
        let tags: Vec<Tag> = data.into();
        Self::new(Kind::SetProduct, content).tags(tags)
    }

    /// Seal
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(all(feature = "std", feature = "nip59"))]
    pub async fn seal<T>(
        signer: &T,
        receiver_pubkey: &PublicKey,
        rumor: EventBuilder,
    ) -> Result<Self, Error>
    where
        T: NostrSigner,
    {
        Ok(nip59::make_seal(signer, receiver_pubkey, rumor).await?)
    }

    /// Gift Wrap from seal
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[cfg(all(feature = "std", feature = "nip59"))]
    pub fn gift_wrap_from_seal<I>(
        receiver: &PublicKey,
        seal: &Event,
        extra_tags: I,
    ) -> Result<Event, Error>
    where
        I: IntoIterator<Item = Tag>,
    {
        if seal.kind != Kind::Seal {
            return Err(Error::WrongKind {
                received: seal.kind,
                expected: WrongKindError::Single(Kind::Seal),
            });
        }

        let keys: Keys = Keys::generate();
        let content: String = nip44::encrypt(
            keys.secret_key(),
            receiver,
            seal.as_json(),
            nip44::Version::default(),
        )?;

        // Collect extra tags
        let mut tags: Vec<Tag> = extra_tags.into_iter().collect();

        // Push received public key
        tags.push(Tag::public_key(*receiver));

        Self::new(Kind::GiftWrap, content)
            .tags(tags)
            .custom_created_at(Timestamp::tweaked(nip59::RANGE_RANDOM_TIMESTAMP_TWEAK))
            .sign_with_keys(&keys)
    }

    /// Gift Wrap
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(all(feature = "std", feature = "nip59"))]
    pub async fn gift_wrap<T, I>(
        signer: &T,
        receiver: &PublicKey,
        rumor: EventBuilder,
        extra_tags: I,
    ) -> Result<Event, Error>
    where
        T: NostrSigner,
        I: IntoIterator<Item = Tag>,
    {
        let seal: Event = Self::seal(signer, receiver, rumor)
            .await?
            .sign(signer)
            .await?;
        Self::gift_wrap_from_seal(receiver, &seal, extra_tags)
    }

    /// Private Direct message rumor
    ///
    /// You probably are looking for [`EventBuilder::private_msg`] method.
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
        Self::new(Kind::PrivateDirectMessage, message).tags([Tag::public_key(receiver)])
    }

    /// Private Direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[inline]
    #[cfg(all(feature = "std", feature = "nip59"))]
    pub async fn private_msg<T, S, I>(
        signer: &T,
        receiver: PublicKey,
        message: S,
        rumor_extra_tags: I,
    ) -> Result<Event, Error>
    where
        T: NostrSigner,
        S: Into<String>,
        I: IntoIterator<Item = Tag>,
    {
        let rumor: Self = Self::private_msg_rumor(receiver, message).tags(rumor_extra_tags);
        Self::gift_wrap(signer, &receiver, rumor, []).await
    }

    /// Mute list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn mute_list(list: MuteList) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::MuteList, "").tags(tags)
    }

    /// Pinned notes
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn pinned_notes<I>(ids: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        Self::new(Kind::PinList, "").tags(ids.into_iter().map(Tag::event))
    }

    /// Bookmarks
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn bookmarks(list: Bookmarks) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::Bookmarks, "").tags(tags)
    }

    /// Communities
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn communities<I>(communities: I) -> Self
    where
        I: IntoIterator<Item = Coordinate>,
    {
        Self::new(Kind::Communities, "").tags(communities.into_iter().map(Tag::from))
    }

    /// Public chats
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn public_chats<I>(chat: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        Self::new(Kind::PublicChats, "").tags(chat.into_iter().map(Tag::event))
    }

    /// Blocked relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn blocked_relays<I>(relay: I) -> Self
    where
        I: IntoIterator<Item = UncheckedUrl>,
    {
        Self::new(Kind::BlockedRelays, "").tags(
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
        Self::new(Kind::SearchRelays, "").tags(
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
        Self::new(Kind::Interests, "").tags(tags)
    }

    /// Emojis
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    #[inline]
    pub fn emojis(list: Emojis) -> Self {
        let tags: Vec<Tag> = list.into();
        Self::new(Kind::Emojis, "").tags(tags)
    }

    /// Follow set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    pub fn follow_set<ID, I>(identifier: ID, public_keys: I) -> Self
    where
        ID: Into<String>,
        I: IntoIterator<Item = PublicKey>,
    {
        let tags: Vec<Tag> = vec![Tag::identifier(identifier)];
        Self::new(Kind::FollowSet, "").tags(
            tags.into_iter()
                .chain(public_keys.into_iter().map(Tag::public_key)),
        )
    }

    /// Relay set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    pub fn relay_set<ID, I>(identifier: ID, relays: I) -> Self
    where
        ID: Into<String>,
        I: IntoIterator<Item = UncheckedUrl>,
    {
        let tags: Vec<Tag> = vec![Tag::identifier(identifier)];
        Self::new(Kind::RelaySet, "").tags(
            tags.into_iter().chain(
                relays
                    .into_iter()
                    .map(|r| Tag::from_standardized_without_cell(TagStandard::Relay(r))),
            ),
        )
    }

    /// Bookmark set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    pub fn bookmarks_set<ID>(identifier: ID, list: Bookmarks) -> Self
    where
        ID: Into<String>,
    {
        let mut tags: Vec<Tag> = list.into();
        tags.push(Tag::identifier(identifier));
        Self::new(Kind::BookmarkSet, "").tags(tags)
    }

    /// Article Curation set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    pub fn articles_curation_set<ID>(identifier: ID, list: ArticlesCuration) -> Self
    where
        ID: Into<String>,
    {
        let mut tags: Vec<Tag> = list.into();
        tags.push(Tag::identifier(identifier));
        Self::new(Kind::ArticlesCurationSet, "").tags(tags)
    }

    /// Videos Curation set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    pub fn videos_curation_set<ID, I>(identifier: ID, video: I) -> Self
    where
        ID: Into<String>,
        I: IntoIterator<Item = Coordinate>,
    {
        let tags: Vec<Tag> = vec![Tag::identifier(identifier)];
        Self::new(Kind::VideosCurationSet, "").tags(
            tags.into_iter()
                .chain(video.into_iter().map(Tag::coordinate)),
        )
    }

    /// Interest set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    pub fn interest_set<ID, I, S>(identifier: ID, hashtags: I) -> Self
    where
        ID: Into<String>,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let tags: Vec<Tag> = vec![Tag::identifier(identifier)];
        Self::new(Kind::InterestSet, "").tags(
            tags.into_iter()
                .chain(hashtags.into_iter().map(Tag::hashtag)),
        )
    }

    /// Emoji set
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/51.md>
    pub fn emoji_set<ID, I>(identifier: ID, emojis: I) -> Self
    where
        ID: Into<String>,
        I: IntoIterator<Item = (String, UncheckedUrl)>,
    {
        let tags: Vec<Tag> = vec![Tag::identifier(identifier)];
        Self::new(Kind::EmojiSet, "").tags(tags.into_iter().chain(emojis.into_iter().map(
            |(s, url)| {
                Tag::from_standardized_without_cell(TagStandard::Emoji { shortcode: s, url })
            },
        )))
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
        Self::new(Kind::Label, "").tags([
            Tag::from_standardized_without_cell(TagStandard::LabelNamespace(namespace)),
            Tag::from_standardized_without_cell(TagStandard::Label(labels)),
        ])
    }

    /// Git Repository Announcement
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    #[inline]
    pub fn git_repository_announcement(announcement: GitRepositoryAnnouncement) -> Self {
        announcement.to_event_builder()
    }

    /// Git Issue
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    #[inline]
    pub fn git_issue(issue: GitIssue) -> Self {
        issue.to_event_builder()
    }

    /// Git Patch
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    #[inline]
    pub fn git_patch(patch: GitPatch) -> Self {
        patch.to_event_builder()
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

        let event = EventBuilder::text_note("hello")
            .sign_with_keys(&keys)
            .unwrap();

        let serialized = event.as_json();
        let deserialized = Event::from_json(serialized).unwrap();

        assert_eq!(event, deserialized);
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
            .any(|t| t.kind() == TagKind::Preimage);

        assert!(has_preimage_tag);
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
            .any(|t| t.kind() == TagKind::Preimage);

        assert!(!has_preimage_tag);
    }

    #[test]
    fn test_badge_definition_event_builder_badge_id_only() {
        let badge_id = String::from("bravery");
        let event_builder =
            EventBuilder::define_badge(badge_id, None, None, None, None, Vec::new());

        let has_id =
            event_builder.tags.clone().iter().any(|t| {
                t.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::D))
            });
        assert!(has_id);

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

        let has_id =
            event_builder.tags.clone().iter().any(|t| {
                t.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::D))
            });
        assert!(has_id);

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
            pub_key
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
                ["p", "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245"],
                ["p", "232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df"]
            ]
            }}"#,
            pub_key, pub_key
        );
        let example_event: Event = serde_json::from_str(&example_event_json).unwrap();

        // Create new event with the event builder
        let awarded_pubkeys = vec![
            PublicKey::from_str("32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245")
                .unwrap(),
            PublicKey::from_str("232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df")
                .unwrap(),
        ];
        let event_builder: Event =
            EventBuilder::award_badge(&badge_definition_event, awarded_pubkeys)
                .unwrap()
                .sign_with_keys(&keys)
                .unwrap();

        assert_eq!(event_builder.kind, Kind::BadgeAward);
        assert_eq!(event_builder.content, "");
        assert_eq!(event_builder.tags, example_event.tags);
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

        let awarded_pubkeys = vec![
            pub_key,
            PublicKey::from_str("232a4ba3df82ccc252a35abee7d87d1af8fc3cc749e4002c3691434da692b1df")
                .unwrap(),
        ];
        let bravery_badge_event =
            EventBuilder::define_badge("bravery", None, None, None, None, Vec::new())
                .sign_with_keys(&badge_one_keys)
                .unwrap();
        let bravery_badge_award =
            EventBuilder::award_badge(&bravery_badge_event, awarded_pubkeys.clone())
                .unwrap()
                .sign_with_keys(&badge_one_keys)
                .unwrap();

        // Badge 2
        let badge_two_keys = Keys::generate();
        let badge_two_pubkey = badge_two_keys.public_key();

        let honor_badge_event =
            EventBuilder::define_badge("honor", None, None, None, None, Vec::new())
                .sign_with_keys(&badge_two_keys)
                .unwrap();
        let honor_badge_award =
            EventBuilder::award_badge(&honor_badge_event, awarded_pubkeys.clone())
                .unwrap()
                .sign_with_keys(&badge_two_keys)
                .unwrap();

        let example_event_json = format!(
            r#"{{
            "content":"",
            "id": "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "kind": 30008,
            "pubkey": "{pub_key}",
            "sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            "created_at":1671739153,
            "tags":[
                ["d", "profile_badges"],
                ["a", "30009:{badge_one_pubkey}:bravery"],
                ["e", "{}"],
                ["a", "30009:{badge_two_pubkey}:honor"],
                ["e", "{}"]
            ]
            }}"#,
            bravery_badge_award.id, honor_badge_award.id,
        );
        let example_event: Event = serde_json::from_str(&example_event_json).unwrap();

        let badge_definitions = vec![bravery_badge_event, honor_badge_event];
        let badge_awards = vec![bravery_badge_award, honor_badge_award];
        let profile_badges =
            EventBuilder::profile_badges(badge_definitions, badge_awards, &pub_key)
                .unwrap()
                .sign_with_keys(&keys)
                .unwrap();

        assert_eq!(profile_badges.kind, Kind::ProfileBadges);
        assert_eq!(profile_badges.tags, example_event.tags);
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
            black_box(EventBuilder::text_note("hello", []).sign_with_keys(&keys)).unwrap();
        });
    }
}
