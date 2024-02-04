// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(non_camel_case_types)]

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::event::tag;
use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::nips::nip26::Conditions;
use nostr::secp256k1::schnorr::Signature;
use nostr::{Kind, UncheckedUrl, Url};
use uniffi::{Enum, Object, Record};

use crate::error::{NostrError, Result};
use crate::nips::nip48::Protocol;
use crate::nips::nip53::LiveEventMarker;
use crate::nips::nip90::DataVendingMachineStatus;
use crate::{Event, EventId, ImageDimensions, LiveEventStatus, PublicKey, Timestamp};

/// Marker
#[derive(Enum)]
pub enum Marker {
    /// Root
    Root,
    /// Reply
    Reply,
    /// Custom
    Custom { custom: String },
}

impl From<Marker> for tag::Marker {
    fn from(value: Marker) -> Self {
        match value {
            Marker::Root => Self::Root,
            Marker::Reply => Self::Reply,
            Marker::Custom { custom } => Self::Custom(custom),
        }
    }
}

impl From<tag::Marker> for Marker {
    fn from(value: tag::Marker) -> Self {
        match value {
            tag::Marker::Root => Self::Root,
            tag::Marker::Reply => Self::Reply,
            tag::Marker::Custom(custom) => Self::Custom { custom },
        }
    }
}

/// Report
#[derive(Enum)]
pub enum Report {
    /// Depictions of nudity, porn, etc
    Nudity,
    /// Profanity, hateful speech, etc.
    Profanity,
    /// Something which may be illegal in some jurisdiction
    ///
    /// Remember: there is what is right and there is the law.
    Illegal,
    /// Spam
    Spam,
    /// Someone pretending to be someone else
    Impersonation,
}

impl From<Report> for tag::Report {
    fn from(value: Report) -> Self {
        match value {
            Report::Nudity => Self::Nudity,
            Report::Profanity => Self::Profanity,
            Report::Illegal => Self::Illegal,
            Report::Spam => Self::Spam,
            Report::Impersonation => Self::Impersonation,
        }
    }
}

impl From<tag::Report> for Report {
    fn from(value: tag::Report) -> Self {
        match value {
            tag::Report::Nudity => Self::Nudity,
            tag::Report::Profanity => Self::Profanity,
            tag::Report::Illegal => Self::Illegal,
            tag::Report::Spam => Self::Spam,
            tag::Report::Impersonation => Self::Impersonation,
        }
    }
}

#[derive(Enum)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
}

impl From<HttpMethod> for tag::HttpMethod {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => Self::GET,
            HttpMethod::Post => Self::POST,
            HttpMethod::Put => Self::PUT,
            HttpMethod::Patch => Self::PATCH,
        }
    }
}

impl From<tag::HttpMethod> for HttpMethod {
    fn from(value: tag::HttpMethod) -> Self {
        match value {
            tag::HttpMethod::GET => Self::Get,
            tag::HttpMethod::POST => Self::Post,
            tag::HttpMethod::PUT => Self::Put,
            tag::HttpMethod::PATCH => Self::Patch,
        }
    }
}

#[derive(Enum)]
pub enum RelayMetadata {
    /// Read
    Read,
    /// Write
    Write,
}

impl From<RelayMetadata> for nostr::RelayMetadata {
    fn from(value: RelayMetadata) -> Self {
        match value {
            RelayMetadata::Read => Self::Read,
            RelayMetadata::Write => Self::Write,
        }
    }
}

impl From<nostr::RelayMetadata> for RelayMetadata {
    fn from(value: nostr::RelayMetadata) -> Self {
        match value {
            nostr::RelayMetadata::Read => Self::Read,
            nostr::RelayMetadata::Write => Self::Write,
        }
    }
}

#[derive(Enum)]
pub enum TagKind {
    /// Public key
    P,
    /// Public key
    UpperP,
    /// Event id
    E,
    /// Reference (URL, etc.)
    R,
    /// Hashtag
    T,
    /// Geohash
    G,
    /// Identifier
    D,
    /// Referencing and tagging
    A,
    /// External Identities
    I,
    /// MIME type
    M,
    /// Absolute URL
    U,
    /// SHA256
    X,
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
    Unknown {
        unknown: String,
    },
}

impl From<tag::TagKind> for TagKind {
    fn from(value: tag::TagKind) -> Self {
        match value {
            tag::TagKind::P => Self::P,
            tag::TagKind::UpperP => Self::UpperP,
            tag::TagKind::E => Self::E,
            tag::TagKind::R => Self::R,
            tag::TagKind::T => Self::T,
            tag::TagKind::G => Self::G,
            tag::TagKind::D => Self::D,
            tag::TagKind::A => Self::A,
            tag::TagKind::I => Self::I,
            tag::TagKind::M => Self::M,
            tag::TagKind::U => Self::U,
            tag::TagKind::X => Self::X,
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
            tag::TagKind::Custom(unknown) => Self::Unknown { unknown },
        }
    }
}

impl From<TagKind> for tag::TagKind {
    fn from(value: TagKind) -> Self {
        match value {
            TagKind::P => Self::P,
            TagKind::UpperP => Self::UpperP,
            TagKind::E => Self::E,
            TagKind::R => Self::R,
            TagKind::T => Self::T,
            TagKind::G => Self::G,
            TagKind::D => Self::D,
            TagKind::A => Self::A,
            TagKind::I => Self::I,
            TagKind::M => Self::M,
            TagKind::U => Self::U,
            TagKind::X => Self::X,
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
            TagKind::Unknown { unknown } => Self::Custom(unknown),
        }
    }
}

#[derive(Enum)]
pub enum TagEnum {
    Unknown {
        kind: TagKind,
        data: Vec<String>,
    },
    EventTag {
        event_id: Arc<EventId>,
        relay_url: Option<String>,
        marker: Option<Marker>,
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
    PubKeyLiveEvent {
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
    ExternalIdentityTag {
        identity: Identity,
    },
    A {
        kind: u64,
        public_key: Arc<PublicKey>,
        identifier: String,
        relay_url: Option<String>,
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
}

impl From<tag::Tag> for TagEnum {
    fn from(value: tag::Tag) -> Self {
        match value {
            tag::Tag::Generic(kind, data) => Self::Unknown {
                kind: kind.into(),
                data,
            },
            tag::Tag::Event {
                event_id,
                relay_url,
                marker,
            } => Self::EventTag {
                event_id: Arc::new(event_id.into()),
                relay_url: relay_url.map(|u| u.to_string()),
                marker: marker.map(|m| m.into()),
            },
            tag::Tag::PublicKey {
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
            tag::Tag::EventReport(id, report) => Self::EventReport {
                event_id: Arc::new(id.into()),
                report: report.into(),
            },
            tag::Tag::PubKeyReport(pk, report) => Self::PubKeyReport {
                public_key: Arc::new(pk.into()),
                report: report.into(),
            },
            tag::Tag::PubKeyLiveEvent {
                public_key,
                relay_url,
                marker,
                proof,
            } => Self::PubKeyLiveEvent {
                public_key: Arc::new(public_key.into()),
                relay_url: relay_url.map(|u| u.to_string()),
                marker: marker.into(),
                proof: proof.map(|p| p.to_string()),
            },
            tag::Tag::Reference(r) => Self::Reference { reference: r },
            tag::Tag::RelayMetadata(url, rw) => Self::RelayMetadataTag {
                relay_url: url.to_string(),
                rw: rw.map(|rw| rw.into()),
            },
            tag::Tag::Hashtag(t) => Self::Hashtag { hashtag: t },
            tag::Tag::Geohash(g) => Self::Geohash { geohash: g },
            tag::Tag::Identifier(d) => Self::Identifier { identifier: d },
            tag::Tag::A {
                kind,
                public_key,
                identifier,
                relay_url,
            } => Self::A {
                kind: kind.as_u64(),
                public_key: Arc::new(public_key.into()),
                identifier,
                relay_url: relay_url.map(|u| u.to_string()),
            },
            tag::Tag::ExternalIdentity(identity) => Self::ExternalIdentityTag {
                identity: identity.into(),
            },
            tag::Tag::Relay(url) => Self::RelayUrl {
                relay_url: url.to_string(),
            },
            tag::Tag::POW { nonce, difficulty } => Self::POW {
                nonce: nonce.to_string(),
                difficulty,
            },
            tag::Tag::Delegation {
                delegator,
                conditions,
                sig,
            } => Self::Delegation {
                delegator: Arc::new(delegator.into()),
                conditions: conditions.to_string(),
                sig: sig.to_string(),
            },
            tag::Tag::ContentWarning { reason } => Self::ContentWarning { reason },
            tag::Tag::Expiration(timestamp) => Self::Expiration {
                timestamp: Arc::new(timestamp.into()),
            },
            tag::Tag::Subject(sub) => Self::Subject { subject: sub },
            tag::Tag::Challenge(challenge) => Self::Challenge { challenge },
            tag::Tag::Title(title) => Self::Title { title },
            tag::Tag::Image(image, dimensions) => Self::Image {
                url: image.to_string(),
                dimensions: dimensions.map(|d| Arc::new(d.into())),
            },
            tag::Tag::Thumb(thumb, dimensions) => Self::Thumb {
                url: thumb.to_string(),
                dimensions: dimensions.map(|d| Arc::new(d.into())),
            },
            tag::Tag::Summary(summary) => Self::Summary { summary },
            tag::Tag::PublishedAt(timestamp) => Self::PublishedAt {
                timestamp: Arc::new(timestamp.into()),
            },
            tag::Tag::Description(description) => Self::Description { desc: description },
            tag::Tag::Bolt11(bolt11) => Self::Bolt11 { bolt11 },
            tag::Tag::Preimage(preimage) => Self::Preimage { preimage },
            tag::Tag::Relays(relays) => Self::Relays {
                urls: relays.into_iter().map(|r| r.to_string()).collect(),
            },
            tag::Tag::Amount { millisats, bolt11 } => Self::Amount { millisats, bolt11 },
            tag::Tag::Name(name) => Self::Name { name },
            tag::Tag::Lnurl(lnurl) => Self::Lnurl { lnurl },
            tag::Tag::Url(url) => Self::UrlTag {
                url: url.to_string(),
            },
            tag::Tag::MimeType(mime) => Self::MimeType { mime },
            tag::Tag::Aes256Gcm { key, iv } => Self::Aes256Gcm { key, iv },
            tag::Tag::Sha256(hash) => Self::Sha256 {
                hash: hash.to_string(),
            },
            tag::Tag::Size(bytes) => Self::Size { size: bytes as u64 },
            tag::Tag::Dim(dim) => Self::Dim {
                dimensions: Arc::new(dim.into()),
            },
            tag::Tag::Magnet(uri) => Self::Magnet { uri },
            tag::Tag::Blurhash(data) => Self::Blurhash { blurhash: data },
            tag::Tag::Streaming(url) => Self::Streaming {
                url: url.to_string(),
            },
            tag::Tag::Recording(url) => Self::Recording {
                url: url.to_string(),
            },
            tag::Tag::Starts(timestamp) => Self::Starts {
                timestamp: Arc::new(timestamp.into()),
            },
            tag::Tag::Ends(timestamp) => Self::Ends {
                timestamp: Arc::new(timestamp.into()),
            },
            tag::Tag::LiveEventStatus(s) => Self::LiveEventStatusTag { status: s.into() },
            tag::Tag::CurrentParticipants(num) => Self::CurrentParticipants { num },
            tag::Tag::TotalParticipants(num) => Self::TotalParticipants { num },
            tag::Tag::AbsoluteURL(url) => Self::AbsoluteURL {
                url: url.to_string(),
            },
            tag::Tag::Method(method) => Self::Method {
                method: method.into(),
            },
            tag::Tag::Payload(p) => Self::Payload {
                hash: p.to_string(),
            },
            tag::Tag::Anon { msg } => Self::Anon { msg },
            tag::Tag::Proxy { id, protocol } => Self::Proxy {
                id,
                protocol: protocol.into(),
            },
            tag::Tag::Emoji { shortcode, url } => Self::Emoji {
                shortcode,
                url: url.to_string(),
            },
            tag::Tag::Encrypted => Self::Encrypted,
            tag::Tag::Request(event) => Self::Request {
                event: Arc::new(event.into()),
            },
            tag::Tag::DataVendingMachineStatus { status, extra_info } => {
                Self::DataVendingMachineStatusTag {
                    status: status.into(),
                    extra_info,
                }
            }
        }
    }
}

impl TryFrom<TagEnum> for tag::Tag {
    type Error = NostrError;

    fn try_from(value: TagEnum) -> Result<Self, Self::Error> {
        match value {
            TagEnum::Unknown { kind, data } => Ok(Self::Generic(kind.into(), data)),
            TagEnum::EventTag {
                event_id,
                relay_url,
                marker,
            } => Ok(Self::Event {
                event_id: **event_id,
                relay_url: relay_url.map(UncheckedUrl::from),
                marker: marker.map(tag::Marker::from),
            }),
            TagEnum::PublicKeyTag {
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
            TagEnum::EventReport { event_id, report } => {
                Ok(Self::EventReport(**event_id, report.into()))
            }
            TagEnum::PubKeyReport { public_key, report } => {
                Ok(Self::PubKeyReport(**public_key, report.into()))
            }
            TagEnum::PubKeyLiveEvent {
                public_key,
                relay_url,
                marker,
                proof,
            } => Ok(Self::PubKeyLiveEvent {
                public_key: **public_key,
                relay_url: relay_url.map(UncheckedUrl::from),
                marker: marker.into(),
                proof: match proof {
                    Some(proof) => Some(Signature::from_str(&proof)?),
                    None => None,
                },
            }),
            TagEnum::Reference { reference } => Ok(Self::Reference(reference)),
            TagEnum::RelayMetadataTag { relay_url, rw } => Ok(Self::RelayMetadata(
                UncheckedUrl::from(relay_url),
                rw.map(|rw| rw.into()),
            )),
            TagEnum::Hashtag { hashtag } => Ok(Self::Hashtag(hashtag)),
            TagEnum::Geohash { geohash } => Ok(Self::Geohash(geohash)),
            TagEnum::Identifier { identifier } => Ok(Self::Identifier(identifier)),
            TagEnum::ExternalIdentityTag { identity } => {
                Ok(Self::ExternalIdentity(identity.into()))
            }
            TagEnum::A {
                kind,
                public_key,
                identifier,
                relay_url,
            } => Ok(Self::A {
                kind: Kind::from(kind),
                public_key: **public_key,
                identifier,
                relay_url: relay_url.map(UncheckedUrl::from),
            }),
            TagEnum::RelayUrl { relay_url } => Ok(Self::Relay(UncheckedUrl::from(relay_url))),
            TagEnum::POW { nonce, difficulty } => Ok(Self::POW {
                nonce: nonce.parse()?,
                difficulty,
            }),
            TagEnum::Delegation {
                delegator,
                conditions,
                sig,
            } => Ok(Self::Delegation {
                delegator: **delegator,
                conditions: Conditions::from_str(&conditions)?,
                sig: Signature::from_str(&sig)?,
            }),
            TagEnum::ContentWarning { reason } => Ok(Self::ContentWarning { reason }),
            TagEnum::Expiration { timestamp } => Ok(Self::Expiration(**timestamp)),
            TagEnum::Subject { subject } => Ok(Self::Subject(subject)),
            TagEnum::Challenge { challenge } => Ok(Self::Challenge(challenge)),
            TagEnum::Title { title } => Ok(Self::Title(title)),
            TagEnum::Image { url, dimensions } => Ok(Self::Image(
                UncheckedUrl::from(url),
                dimensions.map(|d| d.as_ref().into()),
            )),
            TagEnum::Thumb { url, dimensions } => Ok(Self::Thumb(
                UncheckedUrl::from(url),
                dimensions.map(|d| d.as_ref().into()),
            )),
            TagEnum::Summary { summary } => Ok(Self::Summary(summary)),
            TagEnum::Description { desc } => Ok(Self::Description(desc)),
            TagEnum::Bolt11 { bolt11 } => Ok(Self::Bolt11(bolt11)),
            TagEnum::Preimage { preimage } => Ok(Self::Preimage(preimage)),
            TagEnum::Relays { urls } => Ok(Self::Relays(
                urls.into_iter().map(UncheckedUrl::from).collect(),
            )),
            TagEnum::Amount { millisats, bolt11 } => Ok(Self::Amount { millisats, bolt11 }),
            TagEnum::Lnurl { lnurl } => Ok(Self::Lnurl(lnurl)),
            TagEnum::Name { name } => Ok(Self::Name(name)),
            TagEnum::PublishedAt { timestamp } => Ok(Self::PublishedAt(**timestamp)),
            TagEnum::UrlTag { url } => Ok(Self::Url(Url::parse(&url)?)),
            TagEnum::MimeType { mime } => Ok(Self::MimeType(mime)),
            TagEnum::Aes256Gcm { key, iv } => Ok(Self::Aes256Gcm { key, iv }),
            TagEnum::Sha256 { hash } => Ok(Self::Sha256(Sha256Hash::from_str(&hash)?)),
            TagEnum::Size { size } => Ok(Self::Size(size as usize)),
            TagEnum::Dim { dimensions } => Ok(Self::Dim(dimensions.as_ref().into())),
            TagEnum::Magnet { uri } => Ok(Self::Magnet(uri)),
            TagEnum::Blurhash { blurhash } => Ok(Self::Blurhash(blurhash)),
            TagEnum::Streaming { url } => Ok(Self::Streaming(UncheckedUrl::from(url))),
            TagEnum::Recording { url } => Ok(Self::Recording(UncheckedUrl::from(url))),
            TagEnum::Starts { timestamp } => Ok(Self::Starts(**timestamp)),
            TagEnum::Ends { timestamp } => Ok(Self::Ends(**timestamp)),
            TagEnum::LiveEventStatusTag { status } => Ok(Self::LiveEventStatus(status.into())),
            TagEnum::CurrentParticipants { num } => Ok(Self::CurrentParticipants(num)),
            TagEnum::TotalParticipants { num } => Ok(Self::CurrentParticipants(num)),
            TagEnum::AbsoluteURL { url } => Ok(Self::AbsoluteURL(UncheckedUrl::from(url))),
            TagEnum::Method { method } => Ok(Self::Method(method.into())),
            TagEnum::Payload { hash } => Ok(Self::Payload(Sha256Hash::from_str(&hash)?)),
            TagEnum::Anon { msg } => Ok(Self::Anon { msg }),
            TagEnum::Proxy { id, protocol } => Ok(Self::Proxy {
                id,
                protocol: protocol.into(),
            }),
            TagEnum::Emoji { shortcode, url } => Ok(Self::Emoji {
                shortcode,
                url: UncheckedUrl::from(url),
            }),
            TagEnum::Encrypted => Ok(Self::Encrypted),
            TagEnum::Request { event } => Ok(Self::Request(event.as_ref().deref().clone())),
            TagEnum::DataVendingMachineStatusTag { status, extra_info } => {
                Ok(Self::DataVendingMachineStatus {
                    status: status.into(),
                    extra_info,
                })
            }
        }
    }
}

#[derive(Object)]
pub struct Tag {
    inner: tag::Tag,
}

impl From<tag::Tag> for Tag {
    fn from(inner: tag::Tag) -> Self {
        Self { inner }
    }
}

impl Deref for Tag {
    type Target = tag::Tag;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Tag {
    #[uniffi::constructor]
    pub fn parse(data: Vec<String>) -> Result<Self> {
        Ok(Self {
            inner: tag::Tag::parse(data)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_enum(e: TagEnum) -> Result<Self> {
        Ok(Self {
            inner: tag::Tag::try_from(e)?,
        })
    }

    /// Compose `["p", "<public-key>"]` tag
    #[uniffi::constructor]
    pub fn public_key(public_key: Arc<PublicKey>) -> Self {
        Self {
            inner: tag::Tag::public_key(**public_key),
        }
    }

    /// Compose `["e", "<event-id>"]` tag
    #[uniffi::constructor]
    pub fn event(event_id: Arc<EventId>) -> Self {
        Self {
            inner: tag::Tag::event(**event_id),
        }
    }

    pub fn as_enum(&self) -> TagEnum {
        self.inner.clone().into()
    }

    pub fn as_vec(&self) -> Vec<String> {
        self.inner.as_vec()
    }

    pub fn kind(&self) -> TagKind {
        self.inner.kind().into()
    }
}

/// Supported external identity providers
#[derive(Enum)]
pub enum ExternalIdentity {
    /// github.com
    GitHub,
    /// twitter.com
    Twitter,
    /// mastodon.social
    Mastodon,
    /// telegram.org
    Telegram,
}

impl From<ExternalIdentity> for tag::ExternalIdentity {
    fn from(value: ExternalIdentity) -> Self {
        match value {
            ExternalIdentity::GitHub => Self::GitHub,
            ExternalIdentity::Twitter => Self::Twitter,
            ExternalIdentity::Mastodon => Self::Mastodon,
            ExternalIdentity::Telegram => Self::Telegram,
        }
    }
}

impl From<tag::ExternalIdentity> for ExternalIdentity {
    fn from(value: tag::ExternalIdentity) -> Self {
        match value {
            tag::ExternalIdentity::GitHub => Self::GitHub,
            tag::ExternalIdentity::Twitter => Self::Twitter,
            tag::ExternalIdentity::Mastodon => Self::Mastodon,
            tag::ExternalIdentity::Telegram => Self::Telegram,
        }
    }
}

/// A NIP-39 external identity
#[derive(Record)]
pub struct Identity {
    /// The external identity provider
    pub platform: ExternalIdentity,
    /// The user's identity (username) on the provider
    pub ident: String,
    /// The user's proof on the provider
    pub proof: String,
}

impl From<Identity> for tag::Identity {
    fn from(value: Identity) -> Self {
        Self {
            platform: value.platform.into(),
            ident: value.ident,
            proof: value.proof,
        }
    }
}

impl From<tag::Identity> for Identity {
    fn from(value: tag::Identity) -> Self {
        Self {
            platform: value.platform.into(),
            ident: value.ident,
            proof: value.proof,
        }
    }
}
