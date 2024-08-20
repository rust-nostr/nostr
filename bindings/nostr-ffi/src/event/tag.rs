// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::event::tag;
use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::nips::nip10;
use nostr::nips::nip26::Conditions;
use nostr::secp256k1::schnorr::Signature;
use nostr::{UncheckedUrl, Url};
use uniffi::{Enum, Object};

use super::kind::KindEnum;
use crate::error::{NostrError, Result};
use crate::nips::nip01::Coordinate;
use crate::nips::nip10::Marker;
use crate::nips::nip39::Identity;
use crate::nips::nip48::Protocol;
use crate::nips::nip53::LiveEventMarker;
use crate::nips::nip56::Report;
use crate::nips::nip65::RelayMetadata;
use crate::nips::nip90::DataVendingMachineStatus;
use crate::nips::nip98::HttpMethod;
use crate::types::filter::SingleLetterTag;
use crate::{Event, EventId, ImageDimensions, LiveEventStatus, PublicKey, Timestamp};

#[derive(Enum)]
pub enum TagKind {
    SingleLetter {
        single_letter: Arc<SingleLetterTag>,
    },
    /// Protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    Protected,
    /// Human-readable plaintext summary of what that event is about
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    Alt,
    /// Relay
    RelayUrl,
    /// Nonce
    Nonce,
    /// Delegation
    Delegation,
    /// Content warning
    ContentWarning,
    /// Expiration
    Expiration,
    /// Subject
    Subject,
    /// Auth challenge
    Challenge,
    /// Title (NIP23)
    Title,
    /// Image (NIP23)
    Image,
    /// Thumbnail
    Thumb,
    /// Summary (NIP23)
    Summary,
    /// PublishedAt (NIP23)
    PublishedAt,
    /// Description (NIP57)
    Description,
    /// Bolt11 Invoice (NIP57)
    Bolt11,
    /// Preimage (NIP57)
    Preimage,
    /// Relays (NIP57)
    Relays,
    /// Amount (NIP57)
    Amount,
    /// Lnurl (NIP57)
    Lnurl,
    /// Name tag
    Name,
    /// Url
    Url,
    /// AES 256 GCM
    Aes256Gcm,
    /// Size of file in bytes
    Size,
    /// Size of file in pixels
    Dim,
    /// Magnet
    Magnet,
    /// Blurhash
    Blurhash,
    /// Streaming
    Streaming,
    /// Recording
    Recording,
    /// Starts
    Starts,
    /// Ends
    Ends,
    /// Status
    Status,
    /// Current participants
    CurrentParticipants,
    /// Total participants
    TotalParticipants,
    /// HTTP Method Request
    Method,
    /// Payload HASH
    Payload,
    Anon,
    Proxy,
    Emoji,
    /// Encrypted
    Encrypted,
    Request,
    Word,
    Unknown {
        unknown: String,
    },
}

impl<'a> From<tag::TagKind<'a>> for TagKind {
    fn from(value: tag::TagKind) -> Self {
        match value {
            tag::TagKind::SingleLetter(single_letter) => Self::SingleLetter {
                single_letter: Arc::new(single_letter.into()),
            },
            tag::TagKind::Protected => Self::Protected,
            tag::TagKind::Alt => Self::Alt,
            tag::TagKind::Relay => Self::RelayUrl,
            tag::TagKind::Nonce => Self::Nonce,
            tag::TagKind::Delegation => Self::Delegation,
            tag::TagKind::ContentWarning => Self::ContentWarning,
            tag::TagKind::Expiration => Self::Expiration,
            tag::TagKind::Subject => Self::Subject,
            tag::TagKind::Challenge => Self::Challenge,
            tag::TagKind::Title => Self::Title,
            tag::TagKind::Image => Self::Image,
            tag::TagKind::Thumb => Self::Thumb,
            tag::TagKind::Summary => Self::Summary,
            tag::TagKind::PublishedAt => Self::PublishedAt,
            tag::TagKind::Description => Self::Description,
            tag::TagKind::Bolt11 => Self::Bolt11,
            tag::TagKind::Preimage => Self::Preimage,
            tag::TagKind::Relays => Self::Relays,
            tag::TagKind::Amount => Self::Amount,
            tag::TagKind::Lnurl => Self::Lnurl,
            tag::TagKind::Name => Self::Name,
            tag::TagKind::Url => Self::Url,
            tag::TagKind::Aes256Gcm => Self::Aes256Gcm,
            tag::TagKind::Size => Self::Size,
            tag::TagKind::Dim => Self::Dim,
            tag::TagKind::Magnet => Self::Magnet,
            tag::TagKind::Blurhash => Self::Blurhash,
            tag::TagKind::Streaming => Self::Streaming,
            tag::TagKind::Recording => Self::Recording,
            tag::TagKind::Starts => Self::Starts,
            tag::TagKind::Ends => Self::Ends,
            tag::TagKind::Status => Self::Status,
            tag::TagKind::CurrentParticipants => Self::CurrentParticipants,
            tag::TagKind::TotalParticipants => Self::TotalParticipants,
            tag::TagKind::Method => Self::Method,
            tag::TagKind::Payload => Self::Payload,
            tag::TagKind::Anon => Self::Anon,
            tag::TagKind::Proxy => Self::Proxy,
            tag::TagKind::Emoji => Self::Emoji,
            tag::TagKind::Encrypted => Self::Encrypted,
            tag::TagKind::Request => Self::Request,
            tag::TagKind::Word => Self::Word,
            tag::TagKind::Custom(unknown) => Self::Unknown {
                unknown: unknown.to_string(),
            },
        }
    }
}

impl<'a> From<TagKind> for tag::TagKind<'a> {
    fn from(value: TagKind) -> Self {
        match value {
            TagKind::SingleLetter { single_letter } => Self::SingleLetter(**single_letter),
            TagKind::Protected => Self::Protected,
            TagKind::Alt => Self::Alt,
            TagKind::RelayUrl => Self::Relay,
            TagKind::Nonce => Self::Nonce,
            TagKind::Delegation => Self::Delegation,
            TagKind::ContentWarning => Self::ContentWarning,
            TagKind::Expiration => Self::Expiration,
            TagKind::Subject => Self::Subject,
            TagKind::Challenge => Self::Challenge,
            TagKind::Title => Self::Title,
            TagKind::Image => Self::Image,
            TagKind::Thumb => Self::Thumb,
            TagKind::Summary => Self::Summary,
            TagKind::PublishedAt => Self::PublishedAt,
            TagKind::Description => Self::Description,
            TagKind::Bolt11 => Self::Bolt11,
            TagKind::Preimage => Self::Preimage,
            TagKind::Relays => Self::Relays,
            TagKind::Amount => Self::Amount,
            TagKind::Lnurl => Self::Lnurl,
            TagKind::Name => Self::Name,
            TagKind::Url => Self::Url,
            TagKind::Aes256Gcm => Self::Aes256Gcm,
            TagKind::Size => Self::Size,
            TagKind::Dim => Self::Dim,
            TagKind::Magnet => Self::Magnet,
            TagKind::Blurhash => Self::Blurhash,
            TagKind::Streaming => Self::Streaming,
            TagKind::Recording => Self::Recording,
            TagKind::Starts => Self::Starts,
            TagKind::Ends => Self::Ends,
            TagKind::Status => Self::Status,
            TagKind::CurrentParticipants => Self::CurrentParticipants,
            TagKind::TotalParticipants => Self::TotalParticipants,
            TagKind::Method => Self::Method,
            TagKind::Payload => Self::Payload,
            TagKind::Anon => Self::Anon,
            TagKind::Proxy => Self::Proxy,
            TagKind::Emoji => Self::Emoji,
            TagKind::Encrypted => Self::Encrypted,
            TagKind::Request => Self::Request,
            TagKind::Word => Self::Word,
            TagKind::Unknown { unknown } => Self::Custom(Cow::Owned(unknown)),
        }
    }
}

/// Tag
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Tag {
    inner: tag::Tag,
}

impl Deref for Tag {
    type Target = tag::Tag;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<tag::Tag> for Tag {
    fn from(inner: tag::Tag) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Tag {
    /// Parse tag
    ///
    /// Return error if the tag is empty!
    #[inline]
    #[uniffi::constructor]
    pub fn parse(data: &[String]) -> Result<Self> {
        Ok(Self {
            inner: tag::Tag::parse(data)?,
        })
    }

    /// Construct from standardized tag
    #[inline]
    #[uniffi::constructor]
    pub fn from_standardized(standardized: TagStandard) -> Result<Self> {
        let standardized: tag::TagStandard = tag::TagStandard::try_from(standardized)?;
        Ok(Self {
            inner: tag::Tag::from_standardized(standardized),
        })
    }

    /// Get tag kind
    #[inline]
    pub fn kind(&self) -> TagKind {
        self.inner.kind().into()
    }

    /// Return the **first** tag value (index `1`), if exists.
    #[inline]
    pub fn content(&self) -> Option<String> {
        self.inner.content().map(|c| c.to_string())
    }

    /// Get `SingleLetterTag`
    #[inline]
    pub fn single_letter_tag(&self) -> Option<Arc<SingleLetterTag>> {
        self.inner.single_letter_tag().map(|s| Arc::new(s.into()))
    }

    /// Get standardized tag
    pub fn as_standardized(&self) -> Option<TagStandard> {
        self.inner.as_standardized().cloned().map(|t| t.into())
    }

    /// Get array of strings
    pub fn as_vec(&self) -> Vec<String> {
        self.inner.as_slice().to_vec()
    }

    /// Compose `["e", "<event-id">]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[uniffi::constructor]
    pub fn event(event_id: &EventId) -> Self {
        Self {
            inner: tag::Tag::event(**event_id),
        }
    }

    /// Compose `["p", "<public-key>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[uniffi::constructor]
    pub fn public_key(public_key: &PublicKey) -> Self {
        Self {
            inner: tag::Tag::public_key(**public_key),
        }
    }

    /// Compose `["d", "<identifier>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[uniffi::constructor]
    pub fn identifier(identifier: &str) -> Self {
        Self {
            inner: tag::Tag::identifier(identifier),
        }
    }

    /// Compose `["a", "<coordinate>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[uniffi::constructor]
    pub fn coordinate(coordinate: &Coordinate) -> Self {
        Self {
            inner: tag::Tag::coordinate(coordinate.deref().clone()),
        }
    }

    /// Compose `["nonce", "<nonce>", "<difficulty>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    #[inline]
    #[uniffi::constructor]
    pub fn pow(nonce: u64, difficulty: u8) -> Self {
        Self {
            inner: tag::Tag::pow(nonce as u128, difficulty),
        }
    }

    /// Compose `["expiration", "<timestamp>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[inline]
    #[uniffi::constructor]
    pub fn expiration(timestamp: &Timestamp) -> Self {
        Self {
            inner: tag::Tag::expiration(**timestamp),
        }
    }

    /// Compose `["e", "<event-id>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    #[uniffi::constructor]
    pub fn event_report(event_id: &EventId, report: Report) -> Self {
        Self {
            inner: tag::Tag::event_report(**event_id, report.into()),
        }
    }

    /// Compose `["p", "<public-key>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    #[uniffi::constructor]
    pub fn public_key_report(public_key: &PublicKey, report: Report) -> Self {
        Self {
            inner: tag::Tag::public_key_report(**public_key, report.into()),
        }
    }

    /// Compose `["r", "<relay-url>", "<metadata>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[inline]
    #[uniffi::constructor]
    pub fn relay_metadata(relay_url: &str, metadata: Option<RelayMetadata>) -> Result<Self> {
        let relay_url: Url = Url::from_str(relay_url)?;
        Ok(Self {
            inner: tag::Tag::relay_metadata(relay_url, metadata.map(|m| m.into())),
        })
    }

    /// Compose `["t", "<hashtag>"]` tag
    #[inline]
    #[uniffi::constructor]
    pub fn hashtag(hashtag: &str) -> Self {
        Self {
            inner: tag::Tag::hashtag(hashtag),
        }
    }

    /// Compose `["title", "<title>"]` tag
    #[inline]
    #[uniffi::constructor]
    pub fn title(title: &str) -> Self {
        Self {
            inner: tag::Tag::title(title),
        }
    }

    /// Compose image tag
    #[inline]
    #[uniffi::constructor(default(dimensions = None))]
    pub fn image(url: &str, dimensions: Option<Arc<ImageDimensions>>) -> Self {
        Self {
            inner: tag::Tag::image(UncheckedUrl::from(url), dimensions.map(|d| **d)),
        }
    }

    /// Compose `["description", "<description>"]` tag
    #[inline]
    #[uniffi::constructor]
    pub fn description(description: &str) -> Self {
        Self {
            inner: tag::Tag::description(description),
        }
    }

    /// Protected event
    ///
    /// JSON: `["-"]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    #[inline]
    #[uniffi::constructor]
    pub fn protected() -> Self {
        Self {
            inner: tag::Tag::protected(),
        }
    }

    /// A short human-readable plaintext summary of what that event is about
    ///
    /// JSON: `["alt", "<summary>"]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    #[inline]
    #[uniffi::constructor]
    pub fn alt(summary: &str) -> Self {
        Self {
            inner: tag::Tag::alt(summary),
        }
    }

    /// Compose custom tag
    ///
    /// JSON: `["<kind>", "<value-1>", "<value-2>", ...]`
    #[inline]
    #[uniffi::constructor]
    pub fn custom(kind: TagKind, values: &[String]) -> Self {
        Self {
            inner: tag::Tag::custom(kind.into(), values),
        }
    }

    /// Check if is a standard event tag with `root` marker
    pub fn is_root(&self) -> bool {
        self.inner.is_root()
    }

    /// Check if is a standard event tag with `reply` marker
    pub fn is_reply(&self) -> bool {
        self.inner.is_reply()
    }

    /// Check if it's a protected event tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    pub fn is_protected(&self) -> bool {
        self.inner.is_protected()
    }
}

/// Standardized tag
#[derive(Enum)]
pub enum TagStandard {
    EventTag {
        event_id: Arc<EventId>,
        relay_url: Option<String>,
        marker: Option<Marker>,
        /// Should be the public key of the author of the referenced event
        public_key: Option<Arc<PublicKey>>,
    },
    PublicKeyTag {
        public_key: Arc<PublicKey>,
        relay_url: Option<String>,
        alias: Option<String>,
        /// Whether the p tag is an uppercase P or not
        uppercase: bool,
    },
    EventReport {
        event_id: Arc<EventId>,
        report: Report,
    },
    PubKeyReport {
        public_key: Arc<PublicKey>,
        report: Report,
    },
    PublicKeyLiveEvent {
        public_key: Arc<PublicKey>,
        relay_url: Option<String>,
        marker: LiveEventMarker,
        proof: Option<String>,
    },
    Reference {
        reference: String,
    },
    RelayMetadataTag {
        relay_url: String,
        rw: Option<RelayMetadata>,
    },
    Hashtag {
        hashtag: String,
    },
    Geohash {
        geohash: String,
    },
    Identifier {
        identifier: String,
    },
    ExternalIdentity {
        identity: Identity,
    },
    CoordinateTag {
        coordinate: Arc<Coordinate>,
        relay_url: Option<String>,
    },
    Kind {
        kind: KindEnum,
    },
    RelayUrl {
        relay_url: String,
    },
    POW {
        nonce: String,
        difficulty: u8,
    },
    Delegation {
        delegator: Arc<PublicKey>,
        conditions: String,
        sig: String,
    },
    ContentWarning {
        reason: Option<String>,
    },
    Expiration {
        timestamp: Arc<Timestamp>,
    },
    Subject {
        subject: String,
    },
    Challenge {
        challenge: String,
    },
    Title {
        title: String,
    },
    Image {
        url: String,
        dimensions: Option<Arc<ImageDimensions>>,
    },
    Thumb {
        url: String,
        dimensions: Option<Arc<ImageDimensions>>,
    },
    Summary {
        summary: String,
    },
    Description {
        desc: String,
    },
    Bolt11 {
        bolt11: String,
    },
    Preimage {
        preimage: String,
    },
    Relays {
        urls: Vec<String>,
    },
    Amount {
        millisats: u64,
        bolt11: Option<String>,
    },
    Lnurl {
        lnurl: String,
    },
    Name {
        name: String,
    },
    PublishedAt {
        timestamp: Arc<Timestamp>,
    },
    UrlTag {
        url: String,
    },
    MimeType {
        mime: String,
    },
    Aes256Gcm {
        key: String,
        iv: String,
    },
    Sha256 {
        hash: String,
    },
    Size {
        size: u64,
    },
    /// Size of file in pixels
    Dim {
        dimensions: Arc<ImageDimensions>,
    },
    Magnet {
        uri: String,
    },
    Blurhash {
        blurhash: String,
    },
    Streaming {
        url: String,
    },
    Recording {
        url: String,
    },
    Starts {
        timestamp: Arc<Timestamp>,
    },
    Ends {
        timestamp: Arc<Timestamp>,
    },
    LiveEventStatusTag {
        status: LiveEventStatus,
    },
    CurrentParticipants {
        num: u64,
    },
    TotalParticipants {
        num: u64,
    },
    AbsoluteURL {
        url: String,
    },
    Method {
        method: HttpMethod,
    },
    Payload {
        hash: String,
    },
    Anon {
        msg: Option<String>,
    },
    Proxy {
        id: String,
        protocol: Protocol,
    },
    Emoji {
        shortcode: String,
        url: String,
    },
    Encrypted,
    Request {
        event: Arc<Event>,
    },
    DataVendingMachineStatusTag {
        status: DataVendingMachineStatus,
        extra_info: Option<String>,
    },
    Word {
        word: String,
    },
    LabelNamespace {
        namespace: String,
    },
    Label {
        label: Vec<String>,
    },
    /// Protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    Protected,
    /// A short human-readable plaintext summary of what that event is about
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    Alt {
        summary: String,
    },
}

impl From<tag::TagStandard> for TagStandard {
    fn from(value: tag::TagStandard) -> Self {
        match value {
            tag::TagStandard::Event {
                event_id,
                relay_url,
                marker,
                public_key,
            } => Self::EventTag {
                event_id: Arc::new(event_id.into()),
                relay_url: relay_url.map(|u| u.to_string()),
                marker: marker.map(|m| m.into()),
                public_key: public_key.map(|p| Arc::new(p.into())),
            },
            tag::TagStandard::PublicKey {
                public_key,
                relay_url,
                alias,
                uppercase,
            } => Self::PublicKeyTag {
                public_key: Arc::new(public_key.into()),
                relay_url: relay_url.map(|u| u.to_string()),
                alias,
                uppercase,
            },
            tag::TagStandard::EventReport(id, report) => Self::EventReport {
                event_id: Arc::new(id.into()),
                report: report.into(),
            },
            tag::TagStandard::PublicKeyReport(pk, report) => Self::PubKeyReport {
                public_key: Arc::new(pk.into()),
                report: report.into(),
            },
            tag::TagStandard::PublicKeyLiveEvent {
                public_key,
                relay_url,
                marker,
                proof,
            } => Self::PublicKeyLiveEvent {
                public_key: Arc::new(public_key.into()),
                relay_url: relay_url.map(|u| u.to_string()),
                marker: marker.into(),
                proof: proof.map(|p| p.to_string()),
            },
            tag::TagStandard::Reference(r) => Self::Reference { reference: r },
            tag::TagStandard::RelayMetadata {
                relay_url,
                metadata,
            } => Self::RelayMetadataTag {
                relay_url: relay_url.to_string(),
                rw: metadata.map(|rw| rw.into()),
            },
            tag::TagStandard::Hashtag(t) => Self::Hashtag { hashtag: t },
            tag::TagStandard::Geohash(g) => Self::Geohash { geohash: g },
            tag::TagStandard::Identifier(d) => Self::Identifier { identifier: d },
            tag::TagStandard::Coordinate {
                coordinate,
                relay_url,
            } => Self::CoordinateTag {
                coordinate: Arc::new(coordinate.into()),
                relay_url: relay_url.map(|u| u.to_string()),
            },
            tag::TagStandard::ExternalIdentity(identity) => Self::ExternalIdentity {
                identity: identity.into(),
            },
            tag::TagStandard::Kind(kind) => Self::Kind { kind: kind.into() },
            tag::TagStandard::Relay(url) => Self::RelayUrl {
                relay_url: url.to_string(),
            },
            tag::TagStandard::POW { nonce, difficulty } => Self::POW {
                nonce: nonce.to_string(),
                difficulty,
            },
            tag::TagStandard::Delegation {
                delegator,
                conditions,
                sig,
            } => Self::Delegation {
                delegator: Arc::new(delegator.into()),
                conditions: conditions.to_string(),
                sig: sig.to_string(),
            },
            tag::TagStandard::ContentWarning { reason } => Self::ContentWarning { reason },
            tag::TagStandard::Expiration(timestamp) => Self::Expiration {
                timestamp: Arc::new(timestamp.into()),
            },
            tag::TagStandard::Subject(sub) => Self::Subject { subject: sub },
            tag::TagStandard::Challenge(challenge) => Self::Challenge { challenge },
            tag::TagStandard::Title(title) => Self::Title { title },
            tag::TagStandard::Image(image, dimensions) => Self::Image {
                url: image.to_string(),
                dimensions: dimensions.map(|d| Arc::new(d.into())),
            },
            tag::TagStandard::Thumb(thumb, dimensions) => Self::Thumb {
                url: thumb.to_string(),
                dimensions: dimensions.map(|d| Arc::new(d.into())),
            },
            tag::TagStandard::Summary(summary) => Self::Summary { summary },
            tag::TagStandard::PublishedAt(timestamp) => Self::PublishedAt {
                timestamp: Arc::new(timestamp.into()),
            },
            tag::TagStandard::Description(description) => Self::Description { desc: description },
            tag::TagStandard::Bolt11(bolt11) => Self::Bolt11 { bolt11 },
            tag::TagStandard::Preimage(preimage) => Self::Preimage { preimage },
            tag::TagStandard::Relays(relays) => Self::Relays {
                urls: relays.into_iter().map(|r| r.to_string()).collect(),
            },
            tag::TagStandard::Amount { millisats, bolt11 } => Self::Amount { millisats, bolt11 },
            tag::TagStandard::Name(name) => Self::Name { name },
            tag::TagStandard::Lnurl(lnurl) => Self::Lnurl { lnurl },
            tag::TagStandard::Url(url) => Self::UrlTag {
                url: url.to_string(),
            },
            tag::TagStandard::MimeType(mime) => Self::MimeType { mime },
            tag::TagStandard::Aes256Gcm { key, iv } => Self::Aes256Gcm { key, iv },
            tag::TagStandard::Sha256(hash) => Self::Sha256 {
                hash: hash.to_string(),
            },
            tag::TagStandard::Size(bytes) => Self::Size { size: bytes as u64 },
            tag::TagStandard::Dim(dim) => Self::Dim {
                dimensions: Arc::new(dim.into()),
            },
            tag::TagStandard::Magnet(uri) => Self::Magnet { uri },
            tag::TagStandard::Blurhash(data) => Self::Blurhash { blurhash: data },
            tag::TagStandard::Streaming(url) => Self::Streaming {
                url: url.to_string(),
            },
            tag::TagStandard::Recording(url) => Self::Recording {
                url: url.to_string(),
            },
            tag::TagStandard::Starts(timestamp) => Self::Starts {
                timestamp: Arc::new(timestamp.into()),
            },
            tag::TagStandard::Ends(timestamp) => Self::Ends {
                timestamp: Arc::new(timestamp.into()),
            },
            tag::TagStandard::LiveEventStatus(s) => Self::LiveEventStatusTag { status: s.into() },
            tag::TagStandard::CurrentParticipants(num) => Self::CurrentParticipants { num },
            tag::TagStandard::TotalParticipants(num) => Self::TotalParticipants { num },
            tag::TagStandard::AbsoluteURL(url) => Self::AbsoluteURL {
                url: url.to_string(),
            },
            tag::TagStandard::Method(method) => Self::Method {
                method: method.into(),
            },
            tag::TagStandard::Payload(p) => Self::Payload {
                hash: p.to_string(),
            },
            tag::TagStandard::Anon { msg } => Self::Anon { msg },
            tag::TagStandard::Proxy { id, protocol } => Self::Proxy {
                id,
                protocol: protocol.into(),
            },
            tag::TagStandard::Emoji { shortcode, url } => Self::Emoji {
                shortcode,
                url: url.to_string(),
            },
            tag::TagStandard::Encrypted => Self::Encrypted,
            tag::TagStandard::Request(event) => Self::Request {
                event: Arc::new(event.into()),
            },
            tag::TagStandard::DataVendingMachineStatus { status, extra_info } => {
                Self::DataVendingMachineStatusTag {
                    status: status.into(),
                    extra_info,
                }
            }
            tag::TagStandard::Word(word) => Self::Word { word },
            tag::TagStandard::LabelNamespace(label) => Self::LabelNamespace { namespace: label },
            tag::TagStandard::Label(labels) => Self::Label { label: labels },
            tag::TagStandard::Protected => Self::Protected,
            tag::TagStandard::Alt(summary) => Self::Alt { summary },
        }
    }
}

impl TryFrom<TagStandard> for tag::TagStandard {
    type Error = NostrError;

    fn try_from(value: TagStandard) -> Result<Self, Self::Error> {
        match value {
            TagStandard::EventTag {
                event_id,
                relay_url,
                marker,
                public_key,
            } => Ok(Self::Event {
                event_id: **event_id,
                relay_url: relay_url.map(UncheckedUrl::from),
                marker: marker.map(nip10::Marker::from),
                public_key: public_key.map(|p| **p),
            }),
            TagStandard::PublicKeyTag {
                public_key,
                relay_url,
                alias,
                uppercase,
            } => Ok(Self::PublicKey {
                public_key: **public_key,
                relay_url: relay_url.map(UncheckedUrl::from),
                alias,
                uppercase,
            }),
            TagStandard::EventReport { event_id, report } => {
                Ok(Self::EventReport(**event_id, report.into()))
            }
            TagStandard::PubKeyReport { public_key, report } => {
                Ok(Self::PublicKeyReport(**public_key, report.into()))
            }
            TagStandard::PublicKeyLiveEvent {
                public_key,
                relay_url,
                marker,
                proof,
            } => Ok(Self::PublicKeyLiveEvent {
                public_key: **public_key,
                relay_url: relay_url.map(UncheckedUrl::from),
                marker: marker.into(),
                proof: match proof {
                    Some(proof) => Some(Signature::from_str(&proof)?),
                    None => None,
                },
            }),
            TagStandard::Reference { reference } => Ok(Self::Reference(reference)),
            TagStandard::RelayMetadataTag { relay_url, rw } => Ok(Self::RelayMetadata {
                relay_url: Url::from_str(&relay_url)?,
                metadata: rw.map(|rw| rw.into()),
            }),
            TagStandard::Hashtag { hashtag } => Ok(Self::Hashtag(hashtag)),
            TagStandard::Geohash { geohash } => Ok(Self::Geohash(geohash)),
            TagStandard::Identifier { identifier } => Ok(Self::Identifier(identifier)),
            TagStandard::ExternalIdentity { identity } => {
                Ok(Self::ExternalIdentity(identity.into()))
            }
            TagStandard::CoordinateTag {
                coordinate,
                relay_url,
            } => Ok(Self::Coordinate {
                coordinate: coordinate.as_ref().deref().clone(),
                relay_url: relay_url.map(UncheckedUrl::from),
            }),
            TagStandard::Kind { kind } => Ok(Self::Kind(kind.into())),
            TagStandard::RelayUrl { relay_url } => Ok(Self::Relay(UncheckedUrl::from(relay_url))),
            TagStandard::POW { nonce, difficulty } => Ok(Self::POW {
                nonce: nonce.parse()?,
                difficulty,
            }),
            TagStandard::Delegation {
                delegator,
                conditions,
                sig,
            } => Ok(Self::Delegation {
                delegator: **delegator,
                conditions: Conditions::from_str(&conditions)?,
                sig: Signature::from_str(&sig)?,
            }),
            TagStandard::ContentWarning { reason } => Ok(Self::ContentWarning { reason }),
            TagStandard::Expiration { timestamp } => Ok(Self::Expiration(**timestamp)),
            TagStandard::Subject { subject } => Ok(Self::Subject(subject)),
            TagStandard::Challenge { challenge } => Ok(Self::Challenge(challenge)),
            TagStandard::Title { title } => Ok(Self::Title(title)),
            TagStandard::Image { url, dimensions } => Ok(Self::Image(
                UncheckedUrl::from(url),
                dimensions.map(|d| **d),
            )),
            TagStandard::Thumb { url, dimensions } => Ok(Self::Thumb(
                UncheckedUrl::from(url),
                dimensions.map(|d| **d),
            )),
            TagStandard::Summary { summary } => Ok(Self::Summary(summary)),
            TagStandard::Description { desc } => Ok(Self::Description(desc)),
            TagStandard::Bolt11 { bolt11 } => Ok(Self::Bolt11(bolt11)),
            TagStandard::Preimage { preimage } => Ok(Self::Preimage(preimage)),
            TagStandard::Relays { urls } => Ok(Self::Relays(
                urls.into_iter().map(UncheckedUrl::from).collect(),
            )),
            TagStandard::Amount { millisats, bolt11 } => Ok(Self::Amount { millisats, bolt11 }),
            TagStandard::Lnurl { lnurl } => Ok(Self::Lnurl(lnurl)),
            TagStandard::Name { name } => Ok(Self::Name(name)),
            TagStandard::PublishedAt { timestamp } => Ok(Self::PublishedAt(**timestamp)),
            TagStandard::UrlTag { url } => Ok(Self::Url(Url::parse(&url)?)),
            TagStandard::MimeType { mime } => Ok(Self::MimeType(mime)),
            TagStandard::Aes256Gcm { key, iv } => Ok(Self::Aes256Gcm { key, iv }),
            TagStandard::Sha256 { hash } => Ok(Self::Sha256(Sha256Hash::from_str(&hash)?)),
            TagStandard::Size { size } => Ok(Self::Size(size as usize)),
            TagStandard::Dim { dimensions } => Ok(Self::Dim(**dimensions)),
            TagStandard::Magnet { uri } => Ok(Self::Magnet(uri)),
            TagStandard::Blurhash { blurhash } => Ok(Self::Blurhash(blurhash)),
            TagStandard::Streaming { url } => Ok(Self::Streaming(UncheckedUrl::from(url))),
            TagStandard::Recording { url } => Ok(Self::Recording(UncheckedUrl::from(url))),
            TagStandard::Starts { timestamp } => Ok(Self::Starts(**timestamp)),
            TagStandard::Ends { timestamp } => Ok(Self::Ends(**timestamp)),
            TagStandard::LiveEventStatusTag { status } => Ok(Self::LiveEventStatus(status.into())),
            TagStandard::CurrentParticipants { num } => Ok(Self::CurrentParticipants(num)),
            TagStandard::TotalParticipants { num } => Ok(Self::CurrentParticipants(num)),
            TagStandard::AbsoluteURL { url } => Ok(Self::AbsoluteURL(UncheckedUrl::from(url))),
            TagStandard::Method { method } => Ok(Self::Method(method.into())),
            TagStandard::Payload { hash } => Ok(Self::Payload(Sha256Hash::from_str(&hash)?)),
            TagStandard::Anon { msg } => Ok(Self::Anon { msg }),
            TagStandard::Proxy { id, protocol } => Ok(Self::Proxy {
                id,
                protocol: protocol.into(),
            }),
            TagStandard::Emoji { shortcode, url } => Ok(Self::Emoji {
                shortcode,
                url: UncheckedUrl::from(url),
            }),
            TagStandard::Encrypted => Ok(Self::Encrypted),
            TagStandard::Request { event } => Ok(Self::Request(event.as_ref().deref().clone())),
            TagStandard::DataVendingMachineStatusTag { status, extra_info } => {
                Ok(Self::DataVendingMachineStatus {
                    status: status.into(),
                    extra_info,
                })
            }
            TagStandard::Word { word } => Ok(Self::Word(word)),
            TagStandard::LabelNamespace { namespace } => Ok(Self::LabelNamespace(namespace)),
            TagStandard::Label { label } => Ok(Self::Label(label)),
            TagStandard::Protected => Ok(Self::Protected),
            TagStandard::Alt { summary } => Ok(Self::Alt(summary)),
        }
    }
}
