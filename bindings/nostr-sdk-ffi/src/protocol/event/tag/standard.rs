// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::event::tag;
use nostr::hashes::sha1::Hash as Sha1Hash;
use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::nips::nip10;
use nostr::nips::nip26::Conditions;
use nostr::secp256k1::schnorr::Signature;
use nostr::{RelayUrl, Url};
use uniffi::{Enum, Record};

use crate::error::NostrSdkError;
use crate::protocol::event::kind::Kind;
use crate::protocol::event::{Event, EventId};
use crate::protocol::key::PublicKey;
use crate::protocol::nips::nip01::Coordinate;
use crate::protocol::nips::nip10::Marker;
use crate::protocol::nips::nip39::Identity;
use crate::protocol::nips::nip48::Protocol;
use crate::protocol::nips::nip53::{LiveEventMarker, LiveEventStatus};
use crate::protocol::nips::nip56::Report;
use crate::protocol::nips::nip65::RelayMetadata;
use crate::protocol::nips::nip73::ExternalContentId;
use crate::protocol::nips::nip90::DataVendingMachineStatus;
use crate::protocol::nips::nip98::HttpMethod;
use crate::protocol::types::{ImageDimensions, Timestamp};

#[derive(Record)]
pub struct TagClientAddress {
    /// Coordinate
    pub coordinate: Arc<Coordinate>,
    /// Relay hint
    pub hint: Option<String>,
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
        /// Whether the e tag is an uppercase E or not
        uppercase: bool,
    },
    Quote {
        event_id: Arc<EventId>,
        relay_url: Option<String>,
        /// Should be the public key of the author of the referenced event
        public_key: Option<Arc<PublicKey>>,
    },
    /// Git clone (`clone` tag)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitClone {
        urls: Vec<String>,
    },
    /// Git commit
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitCommit {
        hash: String,
    },
    /// Git earliest unique commit ID
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitEarliestUniqueCommitId {
        commit: String,
    },
    /// Git repo maintainers
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitMaintainers {
        public_keys: Vec<Arc<PublicKey>>,
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
    ExternalContent {
        content: ExternalContentId,
        /// Hint URL
        hint: Option<String>,
        uppercase: bool,
    },
    ExternalIdentity {
        identity: Identity,
    },
    CoordinateTag {
        coordinate: Arc<Coordinate>,
        relay_url: Option<String>,
        /// Whether the a tag is an uppercase A or not
        uppercase: bool,
    },
    KindTag {
        kind: Arc<Kind>,
        /// Whether the k tag is an uppercase K or not
        uppercase: bool,
    },
    RelayUrl {
        relay_url: String,
    },
    /// All relays tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/62.md>
    AllRelays,
    POW {
        nonce: String,
        difficulty: u8,
    },
    Client {
        name: String,
        address: Option<TagClientAddress>,
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
        dimensions: Option<ImageDimensions>,
    },
    Thumb {
        url: String,
        dimensions: Option<ImageDimensions>,
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
        dimensions: ImageDimensions,
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
    Word {
        word: String,
    },
    Web {
        urls: Vec<String>,
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
                uppercase,
            } => Self::EventTag {
                event_id: Arc::new(event_id.into()),
                relay_url: relay_url.map(|u| u.to_string()),
                marker: marker.map(|m| m.into()),
                public_key: public_key.map(|p| Arc::new(p.into())),
                uppercase,
            },
            tag::TagStandard::Quote {
                event_id,
                relay_url,
                public_key,
            } => Self::Quote {
                event_id: Arc::new(event_id.into()),
                relay_url: relay_url.map(|u| u.to_string()),
                public_key: public_key.map(|p| Arc::new(p.into())),
            },
            tag::TagStandard::GitClone(urls) => Self::GitClone {
                urls: urls.into_iter().map(|r| r.to_string()).collect(),
            },
            tag::TagStandard::GitCommit(hash) => Self::GitCommit {
                hash: hash.to_string(),
            },
            tag::TagStandard::GitEarliestUniqueCommitId(commit) => {
                Self::GitEarliestUniqueCommitId {
                    commit: commit.to_string(),
                }
            }
            tag::TagStandard::GitMaintainers(public_keys) => Self::GitMaintainers {
                public_keys: public_keys
                    .into_iter()
                    .map(|p| Arc::new(p.into()))
                    .collect(),
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
                uppercase,
            } => Self::CoordinateTag {
                coordinate: Arc::new(coordinate.into()),
                relay_url: relay_url.map(|u| u.to_string()),
                uppercase,
            },
            tag::TagStandard::ExternalContent {
                content,
                hint,
                uppercase,
            } => Self::ExternalContent {
                content: content.into(),
                hint: hint.map(|u| u.to_string()),
                uppercase,
            },
            tag::TagStandard::ExternalIdentity(identity) => Self::ExternalIdentity {
                identity: identity.into(),
            },
            tag::TagStandard::Kind { kind, uppercase } => Self::KindTag {
                kind: Arc::new(kind.into()),
                uppercase,
            },
            tag::TagStandard::Relay(url) => Self::RelayUrl {
                relay_url: url.to_string(),
            },
            tag::TagStandard::AllRelays => Self::AllRelays,
            tag::TagStandard::POW { nonce, difficulty } => Self::POW {
                nonce: nonce.to_string(),
                difficulty,
            },
            tag::TagStandard::Client { name, address } => Self::Client {
                name,
                address: address.map(|(coordinate, hint)| TagClientAddress {
                    coordinate: Arc::new(coordinate.into()),
                    hint: hint.map(|url| url.to_string()),
                }),
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
                dimensions: dimensions.map(|d| d.into()),
            },
            tag::TagStandard::Thumb(thumb, dimensions) => Self::Thumb {
                url: thumb.to_string(),
                dimensions: dimensions.map(|d| d.into()),
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
                dimensions: dim.into(),
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
            tag::TagStandard::Web(urls) => Self::Web {
                urls: urls.into_iter().map(|r| r.to_string()).collect(),
            },
        }
    }
}

impl TryFrom<TagStandard> for tag::TagStandard {
    type Error = NostrSdkError;

    fn try_from(value: TagStandard) -> Result<Self, Self::Error> {
        match value {
            TagStandard::EventTag {
                event_id,
                relay_url,
                marker,
                public_key,
                uppercase,
            } => Ok(Self::Event {
                event_id: **event_id,
                relay_url: match relay_url {
                    Some(url) => Some(RelayUrl::parse(&url)?),
                    None => None,
                },
                marker: marker.map(nip10::Marker::from),
                public_key: public_key.map(|p| **p),
                uppercase,
            }),
            TagStandard::Quote {
                event_id,
                relay_url,
                public_key,
            } => Ok(Self::Quote {
                event_id: **event_id,
                relay_url: match relay_url {
                    Some(url) => Some(RelayUrl::parse(&url)?),
                    None => None,
                },
                public_key: public_key.map(|p| **p),
            }),
            TagStandard::GitClone { urls } => {
                let mut parsed_urls: Vec<Url> = Vec::with_capacity(urls.len());
                for url in urls.into_iter() {
                    parsed_urls.push(Url::parse(&url)?);
                }
                Ok(Self::GitClone(parsed_urls))
            }
            TagStandard::GitCommit { hash } => Ok(Self::GitCommit(Sha1Hash::from_str(&hash)?)),
            TagStandard::GitEarliestUniqueCommitId { commit } => Ok(
                Self::GitEarliestUniqueCommitId(Sha1Hash::from_str(&commit)?),
            ),
            TagStandard::GitMaintainers { public_keys } => Ok(Self::GitMaintainers(
                public_keys.into_iter().map(|p| **p).collect(),
            )),
            TagStandard::PublicKeyTag {
                public_key,
                relay_url,
                alias,
                uppercase,
            } => Ok(Self::PublicKey {
                public_key: **public_key,
                relay_url: match relay_url {
                    Some(url) => Some(RelayUrl::parse(&url)?),
                    None => None,
                },
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
                relay_url: match relay_url {
                    Some(url) => Some(RelayUrl::parse(&url)?),
                    None => None,
                },
                marker: marker.into(),
                proof: match proof {
                    Some(proof) => Some(Signature::from_str(&proof)?),
                    None => None,
                },
            }),
            TagStandard::Reference { reference } => Ok(Self::Reference(reference)),
            TagStandard::RelayMetadataTag { relay_url, rw } => Ok(Self::RelayMetadata {
                relay_url: RelayUrl::parse(&relay_url)?,
                metadata: rw.map(|rw| rw.into()),
            }),
            TagStandard::Hashtag { hashtag } => Ok(Self::Hashtag(hashtag)),
            TagStandard::Geohash { geohash } => Ok(Self::Geohash(geohash)),
            TagStandard::Identifier { identifier } => Ok(Self::Identifier(identifier)),
            TagStandard::ExternalContent {
                content,
                hint,
                uppercase,
            } => Ok(Self::ExternalContent {
                content: content.try_into()?,
                hint: match hint {
                    Some(url) => Some(Url::parse(&url)?),
                    None => None,
                },
                uppercase,
            }),
            TagStandard::ExternalIdentity { identity } => {
                Ok(Self::ExternalIdentity(identity.into()))
            }
            TagStandard::CoordinateTag {
                coordinate,
                relay_url,
                uppercase,
            } => Ok(Self::Coordinate {
                coordinate: coordinate.as_ref().deref().clone(),
                relay_url: match relay_url {
                    Some(url) => Some(RelayUrl::parse(&url)?),
                    None => None,
                },
                uppercase,
            }),
            TagStandard::KindTag { kind, uppercase } => Ok(Self::Kind {
                kind: **kind,
                uppercase,
            }),
            TagStandard::RelayUrl { relay_url } => Ok(Self::Relay(RelayUrl::parse(&relay_url)?)),
            TagStandard::AllRelays => Ok(Self::AllRelays),
            TagStandard::POW { nonce, difficulty } => Ok(Self::POW {
                nonce: nonce.parse()?,
                difficulty,
            }),
            TagStandard::Client { name, address } => Ok(Self::Client {
                name,
                address: match address {
                    Some(address) => {
                        let hint: Option<RelayUrl> = match address.hint {
                            Some(url) => Some(RelayUrl::parse(&url)?),
                            None => None,
                        };
                        Some((address.coordinate.as_ref().deref().clone(), hint))
                    }
                    None => None,
                },
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
            TagStandard::Image { url, dimensions } => {
                Ok(Self::Image(Url::parse(&url)?, dimensions.map(|d| d.into())))
            }
            TagStandard::Thumb { url, dimensions } => {
                Ok(Self::Thumb(Url::parse(&url)?, dimensions.map(|d| d.into())))
            }
            TagStandard::Summary { summary } => Ok(Self::Summary(summary)),
            TagStandard::Description { desc } => Ok(Self::Description(desc)),
            TagStandard::Bolt11 { bolt11 } => Ok(Self::Bolt11(bolt11)),
            TagStandard::Preimage { preimage } => Ok(Self::Preimage(preimage)),
            TagStandard::Relays { urls } => {
                let mut parsed_urls: Vec<RelayUrl> = Vec::with_capacity(urls.len());
                for url in urls.into_iter() {
                    parsed_urls.push(RelayUrl::parse(&url)?);
                }
                Ok(Self::Relays(parsed_urls))
            }
            TagStandard::Amount { millisats, bolt11 } => Ok(Self::Amount { millisats, bolt11 }),
            TagStandard::Lnurl { lnurl } => Ok(Self::Lnurl(lnurl)),
            TagStandard::Name { name } => Ok(Self::Name(name)),
            TagStandard::PublishedAt { timestamp } => Ok(Self::PublishedAt(**timestamp)),
            TagStandard::UrlTag { url } => Ok(Self::Url(Url::parse(&url)?)),
            TagStandard::MimeType { mime } => Ok(Self::MimeType(mime)),
            TagStandard::Aes256Gcm { key, iv } => Ok(Self::Aes256Gcm { key, iv }),
            TagStandard::Sha256 { hash } => Ok(Self::Sha256(Sha256Hash::from_str(&hash)?)),
            TagStandard::Size { size } => Ok(Self::Size(size as usize)),
            TagStandard::Dim { dimensions } => Ok(Self::Dim(dimensions.into())),
            TagStandard::Magnet { uri } => Ok(Self::Magnet(uri)),
            TagStandard::Blurhash { blurhash } => Ok(Self::Blurhash(blurhash)),
            TagStandard::Streaming { url } => Ok(Self::Streaming(Url::parse(&url)?)),
            TagStandard::Recording { url } => Ok(Self::Recording(Url::parse(&url)?)),
            TagStandard::Starts { timestamp } => Ok(Self::Starts(**timestamp)),
            TagStandard::Ends { timestamp } => Ok(Self::Ends(**timestamp)),
            TagStandard::LiveEventStatusTag { status } => Ok(Self::LiveEventStatus(status.into())),
            TagStandard::CurrentParticipants { num } => Ok(Self::CurrentParticipants(num)),
            TagStandard::TotalParticipants { num } => Ok(Self::CurrentParticipants(num)),
            TagStandard::AbsoluteURL { url } => Ok(Self::AbsoluteURL(Url::parse(&url)?)),
            TagStandard::Method { method } => Ok(Self::Method(method.into())),
            TagStandard::Payload { hash } => Ok(Self::Payload(Sha256Hash::from_str(&hash)?)),
            TagStandard::Anon { msg } => Ok(Self::Anon { msg }),
            TagStandard::Proxy { id, protocol } => Ok(Self::Proxy {
                id,
                protocol: protocol.into(),
            }),
            TagStandard::Emoji { shortcode, url } => Ok(Self::Emoji {
                shortcode,
                url: Url::parse(&url)?,
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
            TagStandard::Web { urls } => {
                let mut parsed_urls: Vec<Url> = Vec::with_capacity(urls.len());
                for url in urls.into_iter() {
                    parsed_urls.push(Url::parse(&url)?);
                }
                Ok(Self::Web(parsed_urls))
            }
        }
    }
}
