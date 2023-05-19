// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event builder

use core::fmt;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use bitcoin_hashes::sha256::Hash;
#[cfg(target_arch = "wasm32")]
use instant::Instant;
use secp256k1::XOnlyPublicKey;
use serde_json::{json, Value};
use url::Url;

pub use super::kind::Kind;
pub use super::tag::{Marker, Tag, TagKind};
use super::{Event, EventId, UnsignedEvent};
use crate::key::{self, Keys};
#[cfg(feature = "nip04")]
use crate::nips::nip04;
use crate::nips::nip13;
#[cfg(feature = "nip46")]
use crate::nips::nip46::Message as NostrConnectMessage;
use crate::types::{ChannelId, Contact, Metadata, Timestamp};

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
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use secp256k1::SecretKey;

    use crate::{Event, EventBuilder, Keys, Result, Tag};

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
}
