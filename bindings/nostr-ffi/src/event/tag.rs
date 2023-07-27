// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::event::tag;

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

impl From<TagKind> for tag::TagKind {
    fn from(value: TagKind) -> Self {
        match value {
            TagKind::Known { known } => match known {
                TagKindKnown::P => Self::P,
                TagKindKnown::E => Self::E,
                TagKindKnown::R => Self::R,
                TagKindKnown::T => Self::T,
                TagKindKnown::G => Self::G,
                TagKindKnown::D => Self::D,
                TagKindKnown::A => Self::A,
                TagKindKnown::I => Self::I,
                TagKindKnown::M => Self::M,
                TagKindKnown::U => Self::U,
                TagKindKnown::X => Self::X,
                TagKindKnown::RelayUrl => Self::Relay,
                TagKindKnown::Nonce => Self::Nonce,
                TagKindKnown::Delegation => Self::Delegation,
                TagKindKnown::ContentWarning => Self::ContentWarning,
                TagKindKnown::Expiration => Self::Expiration,
                TagKindKnown::Subject => Self::Subject,
                TagKindKnown::Challenge => Self::Challenge,
                TagKindKnown::Title => Self::Title,
                TagKindKnown::Image => Self::Image,
                TagKindKnown::Thumb => Self::Thumb,
                TagKindKnown::Summary => Self::Summary,
                TagKindKnown::PublishedAt => Self::PublishedAt,
                TagKindKnown::Description => Self::Description,
                TagKindKnown::Bolt11 => Self::Bolt11,
                TagKindKnown::Preimage => Self::Preimage,
                TagKindKnown::Relays => Self::Relays,
                TagKindKnown::Amount => Self::Amount,
                TagKindKnown::Lnurl => Self::Lnurl,
                TagKindKnown::Name => Self::Name,
                TagKindKnown::Url => Self::Url,
                TagKindKnown::Aes256Gcm => Self::Aes256Gcm,
                TagKindKnown::Size => Self::Size,
                TagKindKnown::Dim => Self::Dim,
                TagKindKnown::Magnet => Self::Magnet,
                TagKindKnown::Blurhash => Self::Blurhash,
                TagKindKnown::Streaming => Self::Streaming,
                TagKindKnown::Recording => Self::Recording,
                TagKindKnown::Starts => Self::Starts,
                TagKindKnown::Ends => Self::Ends,
                TagKindKnown::Status => Self::Status,
                TagKindKnown::CurrentParticipants => Self::CurrentParticipants,
                TagKindKnown::TotalParticipants => Self::TotalParticipants,
                TagKindKnown::Method => Self::Method,
                TagKindKnown::Payload => Self::Payload,
            }
            TagKind::Unknown { unknown } => Self::Custom(unknown),
        }
    }
}

pub enum Tag {
    Unknown {
        kind: TagKind,
        data: Vec<String>,
    },
    E {
        event_id: String,
        relay_url: Option<String>,
        marker: Option<String>,
    },
    PubKey {
        public_key: String,
        relay_url: Option<String>,
    },
    EventReport {
        event_id: String,
        report: String,
    },
    PubKeyReport {
        public_key: String,
        report: String,
    },
    PubKeyLiveEvent {
        pk: String,
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
        public_key: String,
        identifier: String,
        relay_url: Option<String>,
    },
    RelayUrl {
        relay_url: String,
    },
    ContactList {
        pk: String,
        relay_url: Option<String>,
        alias: Option<String>,
    },
    POW {
        nonce: String,
        difficulty: u8,
    },
    Delegation {
        delegator_pk: String,
        conditions: String,
        sig: String,
    },
    ContentWarning {
        reason: Option<String>,
    },
    Expiration {
        timestamp: u64,
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
        timestamp: u64,
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
        timestamp: u64,
    },
    Ends {
        timestamp: u64,
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

impl From<tag::Tag> for Tag {
    fn from(value: tag::Tag) -> Self {
        match value {
            tag::Tag::Generic(kind, data) => Self::Unknown {
                kind: kind.into(),
                data,
            },
            tag::Tag::Event(id, relay_url, marker) => Self::E {
                event_id: id.to_hex(),
                relay_url: relay_url.map(|u| u.to_string()),
                marker: marker.map(|m| m.to_string()),
            },
            tag::Tag::PubKey(pk, relay_url) => Self::PubKey {
                public_key: pk.to_string(),
                relay_url: relay_url.map(|u| u.to_string()),
            },
            tag::Tag::EventReport(id, report) => Self::EventReport {
                event_id: id.into(),
                report: report.to_string(),
            },
            tag::Tag::PubKeyReport(pk, report) => Self::PubKeyReport {
                public_key: pk.to_string(),
                report: report.to_string(),
            },
            tag::Tag::PubKeyLiveEvent {
                pk,
                relay_url,
                marker,
                proof,
            } => Self::PubKeyLiveEvent {
                pk: pk.to_string(),
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
                public_key: public_key.to_string(),
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
                pk: pk.to_string(),
                relay_url: relay_url.map(|u| u.to_string()),
                alias,
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
                delegator_pk: delegator_pk.to_string(),
                conditions: conditions.to_string(),
                sig: sig.to_string(),
            },
            tag::Tag::ContentWarning { reason } => Self::ContentWarning { reason },
            tag::Tag::Expiration(timestamp) => Self::Expiration {
                timestamp: timestamp.as_u64(),
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
                timestamp: timestamp.as_u64(),
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
                timestamp: timestamp.as_u64(),
            },
            tag::Tag::Ends(timestamp) => Self::Ends {
                timestamp: timestamp.as_u64(),
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
