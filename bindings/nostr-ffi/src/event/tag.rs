// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::event::tag;

use crate::{EventId, PublicKey, Timestamp};

pub enum TagKind {
    Known { known: TagKindKnown },
    Unknown { unknown: String },
}

pub enum TagKindKnown {
    /// Public key
    P,
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
}

impl From<tag::TagKind> for TagKind {
    fn from(value: tag::TagKind) -> Self {
        match value {
            tag::TagKind::P => Self::Known {
                known: TagKindKnown::P,
            },
            tag::TagKind::E => Self::Known {
                known: TagKindKnown::E,
            },
            tag::TagKind::R => Self::Known {
                known: TagKindKnown::R,
            },
            tag::TagKind::T => Self::Known {
                known: TagKindKnown::T,
            },
            tag::TagKind::G => Self::Known {
                known: TagKindKnown::G,
            },
            tag::TagKind::D => Self::Known {
                known: TagKindKnown::D,
            },
            tag::TagKind::A => Self::Known {
                known: TagKindKnown::A,
            },
            tag::TagKind::I => Self::Known {
                known: TagKindKnown::I,
            },
            tag::TagKind::M => Self::Known {
                known: TagKindKnown::M,
            },
            tag::TagKind::U => Self::Known {
                known: TagKindKnown::U,
            },
            tag::TagKind::X => Self::Known {
                known: TagKindKnown::X,
            },
            tag::TagKind::Relay => Self::Known {
                known: TagKindKnown::RelayUrl,
            },
            tag::TagKind::Nonce => Self::Known {
                known: TagKindKnown::Nonce,
            },
            tag::TagKind::Delegation => Self::Known {
                known: TagKindKnown::Delegation,
            },
            tag::TagKind::ContentWarning => Self::Known {
                known: TagKindKnown::ContentWarning,
            },
            tag::TagKind::Expiration => Self::Known {
                known: TagKindKnown::Expiration,
            },
            tag::TagKind::Subject => Self::Known {
                known: TagKindKnown::Subject,
            },
            tag::TagKind::Challenge => Self::Known {
                known: TagKindKnown::Challenge,
            },
            tag::TagKind::Title => Self::Known {
                known: TagKindKnown::Title,
            },
            tag::TagKind::Image => Self::Known {
                known: TagKindKnown::Image,
            },
            tag::TagKind::Thumb => Self::Known {
                known: TagKindKnown::Thumb,
            },
            tag::TagKind::Summary => Self::Known {
                known: TagKindKnown::Summary,
            },
            tag::TagKind::PublishedAt => Self::Known {
                known: TagKindKnown::PublishedAt,
            },
            tag::TagKind::Description => Self::Known {
                known: TagKindKnown::Description,
            },
            tag::TagKind::Bolt11 => Self::Known {
                known: TagKindKnown::Bolt11,
            },
            tag::TagKind::Preimage => Self::Known {
                known: TagKindKnown::Preimage,
            },
            tag::TagKind::Relays => Self::Known {
                known: TagKindKnown::Relays,
            },
            tag::TagKind::Amount => Self::Known {
                known: TagKindKnown::Amount,
            },
            tag::TagKind::Lnurl => Self::Known {
                known: TagKindKnown::Lnurl,
            },
            tag::TagKind::Name => Self::Known {
                known: TagKindKnown::Name,
            },
            tag::TagKind::Url => Self::Known {
                known: TagKindKnown::Url,
            },
            tag::TagKind::Aes256Gcm => Self::Known {
                known: TagKindKnown::Aes256Gcm,
            },
            tag::TagKind::Size => Self::Known {
                known: TagKindKnown::Size,
            },
            tag::TagKind::Dim => Self::Known {
                known: TagKindKnown::Dim,
            },
            tag::TagKind::Magnet => Self::Known {
                known: TagKindKnown::Magnet,
            },
            tag::TagKind::Blurhash => Self::Known {
                known: TagKindKnown::Blurhash,
            },
            tag::TagKind::Streaming => Self::Known {
                known: TagKindKnown::Streaming,
            },
            tag::TagKind::Recording => Self::Known {
                known: TagKindKnown::Recording,
            },
            tag::TagKind::Starts => Self::Known {
                known: TagKindKnown::Starts,
            },
            tag::TagKind::Ends => Self::Known {
                known: TagKindKnown::Ends,
            },
            tag::TagKind::Status => Self::Known {
                known: TagKindKnown::Status,
            },
            tag::TagKind::CurrentParticipants => Self::Known {
                known: TagKindKnown::CurrentParticipants,
            },
            tag::TagKind::TotalParticipants => Self::Known {
                known: TagKindKnown::TotalParticipants,
            },
            tag::TagKind::Method => Self::Known {
                known: TagKindKnown::Method,
            },
            tag::TagKind::Payload => Self::Known {
                known: TagKindKnown::Payload,
            },
            tag::TagKind::Custom(unknown) => Self::Unknown { unknown },
        }
    }
}

pub enum Tag {
    Unknown {
        kind: TagKind,
        data: Vec<String>,
    },
    Event {
        event_id: EventId,
        relay_url: Option<String>,
        marker: Option<String>,
    },
    PubKey {
        public_key: PublicKey,
        relay_url: Option<String>,
    },
    EventReport {
        event_id: EventId,
        report: String,
    },
    PubKeyReport {
        public_key: PublicKey,
        report: String,
    },
    PubKeyLiveEvent {
        pk: PublicKey,
        relay_url: Option<String>,
        marker: String,
        proof: Option<String>,
    },
    Reference {
        reference: String,
    },
    RelayMetadata {
        relay_url: String,
        rw: Option<String>,
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
        identity: String,
        proof: String,
    },
    A {
        kind: u64,
        public_key: PublicKey,
        identifier: String,
        relay_url: Option<String>,
    },
    RelayUrl {
        relay_url: String,
    },
    ContactList {
        pk: PublicKey,
        relay_url: Option<String>,
        alias: Option<String>,
    },
    POW {
        nonce: String,
        difficulty: u8,
    },
    Delegation {
        delegator_pk: PublicKey,
        conditions: String,
        sig: String,
    },
    ContentWarning {
        reason: Option<String>,
    },
    Expiration {
        timestamp: Timestamp,
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
        dimensions: Option<String>,
    },
    Thumb {
        url: String,
        dimensions: Option<String>,
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
        amount: u64,
    },
    Lnurl {
        lnurl: String,
    },
    Name {
        name: String,
    },
    PublishedAt {
        timestamp: Timestamp,
    },
    Url {
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
    Dim {
        dimensions: String,
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
        timestamp: Timestamp,
    },
    Ends {
        timestamp: Timestamp,
    },
    Status {
        status: String,
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
        method: String,
    },
    Payload {
        hash: String,
    },
}

impl From<tag::Tag> for Tag {
    fn from(value: tag::Tag) -> Self {
        match value {
            tag::Tag::Generic(kind, data) => Self::Unknown {
                kind: kind.into(),
                data,
            },
            tag::Tag::Event(id, relay_url, marker) => Self::Event {
                event_id: id.into(),
                relay_url: relay_url.map(|u| u.to_string()),
                marker: marker.map(|m| m.to_string()),
            },
            tag::Tag::PubKey(pk, relay_url) => Self::PubKey {
                public_key: pk.into(),
                relay_url: relay_url.map(|u| u.to_string()),
            },
            tag::Tag::EventReport(id, report) => Self::EventReport {
                event_id: id.into(),
                report: report.to_string(),
            },
            tag::Tag::PubKeyReport(pk, report) => Self::PubKeyReport {
                public_key: pk.into(),
                report: report.to_string(),
            },
            tag::Tag::PubKeyLiveEvent {
                pk,
                relay_url,
                marker,
                proof,
            } => Self::PubKeyLiveEvent {
                pk: pk.into(),
                relay_url: relay_url.map(|u| u.to_string()),
                marker: marker.to_string(),
                proof: proof.map(|p| p.to_string()),
            },
            tag::Tag::Reference(r) => Self::Reference { reference: r },
            tag::Tag::RelayMetadata(url, rw) => Self::RelayMetadata {
                relay_url: url.to_string(),
                rw,
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
                public_key: public_key.into(),
                identifier,
                relay_url: relay_url.map(|u| u.to_string()),
            },
            tag::Tag::ExternalIdentity(identity) => Self::ExternalIdentity {
                identity: format!("{}:{}", identity.platform, identity.ident),
                proof: identity.proof,
            },
            tag::Tag::Relay(url) => Self::RelayUrl {
                relay_url: url.to_string(),
            },
            tag::Tag::ContactList {
                pk,
                relay_url,
                alias,
            } => Self::ContactList {
                pk: pk.into(),
                relay_url: relay_url.map(|u| u.to_string()),
                alias: alias.map(|a| a.to_string()),
            },
            tag::Tag::POW { nonce, difficulty } => Self::POW {
                nonce: nonce.to_string(),
                difficulty,
            },
            tag::Tag::Delegation {
                delegator_pk,
                conditions,
                sig,
            } => Self::Delegation {
                delegator_pk: delegator_pk.into(),
                conditions: conditions.to_string(),
                sig: sig.to_string(),
            },
            tag::Tag::ContentWarning { reason } => Self::ContentWarning {
                reason: reason.map(|r| r.to_string()),
            },
            tag::Tag::Expiration(timestamp) => Self::Expiration {
                timestamp: timestamp.into(),
            },
            tag::Tag::Subject(sub) => Self::Subject { subject: sub },
            tag::Tag::Challenge(challenge) => Self::Challenge { challenge },
            tag::Tag::Title(title) => Self::Title { title },
            tag::Tag::Image(image, dimensions) => Self::Image {
                url: image.to_string(),
                dimensions: dimensions.map(|d| d.to_string()),
            },
            tag::Tag::Thumb(thumb, dimensions) => Self::Thumb {
                url: thumb.to_string(),
                dimensions: dimensions.map(|d| d.to_string()),
            },
            tag::Tag::Summary(summary) => Self::Summary { summary },
            tag::Tag::PublishedAt(timestamp) => Self::PublishedAt {
                timestamp: timestamp.into(),
            },
            tag::Tag::Description(description) => Self::Description { desc: description },
            tag::Tag::Bolt11(bolt11) => Self::Bolt11 { bolt11 },
            tag::Tag::Preimage(preimage) => Self::Preimage { preimage },
            tag::Tag::Relays(relays) => Self::Relays {
                urls: relays.into_iter().map(|r| r.to_string()).collect(),
            },
            tag::Tag::Amount(amount) => Self::Amount { amount },
            tag::Tag::Name(name) => Self::Name { name },
            tag::Tag::Lnurl(lnurl) => Self::Lnurl { lnurl },
            tag::Tag::Url(url) => Self::Url {
                url: url.to_string(),
            },
            tag::Tag::MimeType(mime) => Self::MimeType { mime },
            tag::Tag::Aes256Gcm { key, iv } => Self::Aes256Gcm { key, iv },
            tag::Tag::Sha256(hash) => Self::Sha256 {
                hash: hash.to_string(),
            },
            tag::Tag::Size(bytes) => Self::Size { size: bytes as u64 },
            tag::Tag::Dim(dim) => Self::Dim {
                dimensions: dim.to_string(),
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
                timestamp: timestamp.into(),
            },
            tag::Tag::Ends(timestamp) => Self::Ends {
                timestamp: timestamp.into(),
            },
            tag::Tag::Status(s) => Self::Status {
                status: s.to_string(),
            },
            tag::Tag::CurrentParticipants(num) => Self::CurrentParticipants { num },
            tag::Tag::TotalParticipants(num) => Self::TotalParticipants { num },
            tag::Tag::AbsoluteURL(url) => Self::AbsoluteURL {
                url: url.to_string(),
            },
            tag::Tag::Method(method) => Self::Method {
                method: method.to_string(),
            },
            tag::Tag::Payload(p) => Self::Payload {
                hash: p.to_string(),
            },
        }
    }
}

impl Tag {
    /* pub fn parse<S>(data: Vec<S>) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Tag::try_from(data)
    } */

    /* pub fn as_vec(&self) -> Vec<String> {
        self.clone().into()
    } */

    /* pub fn kind(&self) -> TagKind {

    } */
}

// UDL

/* [Enum]
interface Tag {
    Unknown(TagKind kind, sequence<string> data);
    Event(EventId event_id, string? relay_url, string? marker);
    PubKey(PublicKey public_key, string? relay_url);
    EventReport(EventId event_id, string report);
    PubKeyReport(PublicKey public_key, string report);
    PubKeyLiveEvent(PublicKey pk, string? relay_url, string marker, string? proof);
    Reference(string reference);
    RelayMetadata(string relay_url, string? rw);
    Hashtag(string hashtag);
    Geohash(string geohash);
    Identifier(string identifier);
    ExternalIdentity(string identity, string proof);
    A(u64 kind, PublicKey public_key, string identifier, string? relay_url);
    RelayUrl(string relay_url);
    ContactList(PublicKey pk, string? relay_url, string? alias);
    POW(string nonce, u8 difficulty);
    Delegation(PublicKey delegator_pk, string conditions, string sig);
    ContentWarning(string? reason);
    Expiration(Timestamp timestamp);
    Subject(string subject);
    Challenge(string challenge);
    Title(string title);
    Image(string url, string? dimensions);
    Thumb(string url, string? dimensions);
    Summary(string summary);
    Description(string desc);
    Bolt11(string bolt11);
    Preimage(string preimage);
    Relays(sequence<string> urls);
    Amount(u64 amount);
    Lnurl(string lnurl);
    Name(string name);
    PublishedAt(Timestamp timestamp);
    Url(string url);
    MimeType(string mime);
    Aes256Gcm(string key, string iv);
    Sha256(string hash);
    Size(u64 size);
    Dim(string dimensions);
    Magnet(string uri);
    Blurhash(string blurhash);
    Streaming(string url);
    Recording(string url);
    Starts(Timestamp timestamp);
    Ends(Timestamp timestamp);
    Status(string status);
    CurrentParticipants(u64 num);
    TotalParticipants(u64 num);
    AbsoluteURL(string url);
    Method(string method);
    Payload(string hash);
}; */
