// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hex::HexToArrayError;
use bitcoin::secp256k1;
use bitcoin::secp256k1::schnorr::Signature;
use serde::de::Error as DeserializerError;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::id::{self, EventId};
use crate::nips::nip01::{self, Coordinate};
use crate::nips::nip26::{Conditions, Error as Nip26Error};
use crate::nips::nip48::Protocol;
use crate::nips::nip53::{self, LiveEventMarker, LiveEventStatus};
use crate::nips::nip90::DataVendingMachineStatus;
use crate::types::filter::IntoGenericTagValue;
use crate::types::url::{ParseError, Url};
use crate::{
    key, Alphabet, Event, GenericTagValue, JsonUtil, Kind, PublicKey, SingleLetterTag, Timestamp,
    UncheckedUrl,
};

/// [`Tag`] error
#[derive(Debug)]
pub enum Error {
    /// Keys
    Keys(key::Error),
    /// Impossible to parse [`Marker`]
    MarkerParseError,
    /// Unknown [`Report`]
    UnknownReportType,
    /// Impossible to find tag kind
    KindNotFound,
    /// Invalid Zap Request
    InvalidZapRequest,
    /// Impossible to parse integer
    ParseIntError(ParseIntError),
    /// Secp256k1
    Secp256k1(secp256k1::Error),
    /// Hex decoding error
    Hex(HexToArrayError),
    /// Url parse error
    Url(ParseError),
    /// EventId error
    EventId(id::Error),
    /// NIP01 error
    NIP01(nip01::Error),
    /// NIP26 error
    NIP26(Nip26Error),
    /// NIP53 error
    NIP53(nip53::Error),
    /// Event Error
    Event(crate::event::Error),
    /// NIP-39 Error
    InvalidIdentity,
    /// Invalid Image Dimensions
    InvalidImageDimensions,
    /// Invalid HTTP Method
    InvalidHttpMethod(String),
    /// Invalid Relay Metadata
    InvalidRelayMetadata(String),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keys(e) => write!(f, "Keys: {e}"),
            Self::MarkerParseError => write!(f, "Impossible to parse marker"),
            Self::UnknownReportType => write!(f, "Unknown report type"),
            Self::KindNotFound => write!(f, "Impossible to find tag kind"),
            Self::InvalidZapRequest => write!(f, "Invalid Zap request"),
            Self::ParseIntError(e) => write!(f, "Parse integer: {e}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::Hex(e) => write!(f, "Hex: {e}"),
            Self::Url(e) => write!(f, "Url: {e}"),
            Self::EventId(e) => write!(f, "Event ID: {e}"),
            Self::NIP01(e) => write!(f, "NIP01: {e}"),
            Self::NIP26(e) => write!(f, "NIP26: {e}"),
            Self::NIP53(e) => write!(f, "NIP53: {e}"),
            Self::Event(e) => write!(f, "Event: {e}"),
            Self::InvalidIdentity => write!(f, "Invalid identity tag"),
            Self::InvalidImageDimensions => write!(f, "Invalid image dimensions"),
            Self::InvalidHttpMethod(m) => write!(f, "Invalid HTTP method: {m}"),
            Self::InvalidRelayMetadata(s) => write!(f, "Invalid relay metadata: {s}"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<HexToArrayError> for Error {
    fn from(e: HexToArrayError) -> Self {
        Self::Hex(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<id::Error> for Error {
    fn from(e: id::Error) -> Self {
        Self::EventId(e)
    }
}

impl From<nip01::Error> for Error {
    fn from(e: nip01::Error) -> Self {
        Self::NIP01(e)
    }
}

impl From<Nip26Error> for Error {
    fn from(e: Nip26Error) -> Self {
        Self::NIP26(e)
    }
}

impl From<nip53::Error> for Error {
    fn from(e: nip53::Error) -> Self {
        Self::NIP53(e)
    }
}

impl From<crate::event::Error> for Error {
    fn from(e: crate::event::Error) -> Self {
        Self::Event(e)
    }
}

/// Marker
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Marker {
    /// Root
    Root,
    /// Reply
    Reply,
    /// Mention
    Mention,
    /// Custom
    Custom(String),
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Root => write!(f, "root"),
            Self::Reply => write!(f, "reply"),
            Self::Mention => write!(f, "mention"),
            Self::Custom(m) => write!(f, "{m}"),
        }
    }
}

impl<S> From<S> for Marker
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        let s: String = s.into();
        match s.as_str() {
            "root" => Self::Root,
            "reply" => Self::Reply,
            "mention" => Self::Mention,
            _ => Self::Custom(s),
        }
    }
}

/// Report
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nudity => write!(f, "nudity"),
            Self::Profanity => write!(f, "profanity"),
            Self::Illegal => write!(f, "illegal"),
            Self::Spam => write!(f, "spam"),
            Self::Impersonation => write!(f, "impersonation"),
        }
    }
}

impl FromStr for Report {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nudity" => Ok(Self::Nudity),
            "profanity" => Ok(Self::Profanity),
            "illegal" => Ok(Self::Illegal),
            "spam" => Ok(Self::Spam),
            "impersonation" => Ok(Self::Impersonation),
            _ => Err(Error::UnknownReportType),
        }
    }
}

/// Simple struct to hold `width` x `height`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageDimensions {
    /// Width
    pub width: u64,
    /// Height
    pub height: u64,
}

impl ImageDimensions {
    /// Net image dimensions
    pub fn new(width: u64, height: u64) -> Self {
        Self { width, height }
    }
}

impl FromStr for ImageDimensions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dimensions: Vec<&str> = s.split('x').collect();
        if dimensions.len() == 2 {
            let (width, height) = (dimensions[0], dimensions[1]);
            Ok(Self::new(width.parse()?, height.parse()?))
        } else {
            Err(Error::InvalidImageDimensions)
        }
    }
}

impl fmt::Display for ImageDimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// HTTP Method
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HttpMethod {
    /// GET
    GET,
    /// POST
    POST,
    /// PUT
    PUT,
    /// PATCH
    PATCH,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GET => write!(f, "GET"),
            Self::POST => write!(f, "POST"),
            Self::PUT => write!(f, "PUT"),
            Self::PATCH => write!(f, "PATCH"),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "PATCH" => Ok(Self::PATCH),
            m => Err(Error::InvalidHttpMethod(m.to_string())),
        }
    }
}

/// Relay Metadata
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayMetadata {
    /// Read
    Read,
    /// Write
    Write,
}

impl fmt::Display for RelayMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
        }
    }
}

impl FromStr for RelayMetadata {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            s => Err(Error::InvalidRelayMetadata(s.to_string())),
        }
    }
}

/// Tag kind
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagKind {
    /// Single letter
    SingleLetter(SingleLetterTag),
    /// Relay
    Relay,
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
    /// Anon
    Anon,
    /// Proxy
    Proxy,
    /// Emoji
    Emoji,
    /// Encrypted
    Encrypted,
    /// Request (NIP90)
    Request,
    /// Word
    Word,
    /// Custom tag kind
    Custom(String),
}

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SingleLetter(s) => write!(f, "{s}"),
            Self::Relay => write!(f, "relay"),
            Self::Nonce => write!(f, "nonce"),
            Self::Delegation => write!(f, "delegation"),
            Self::ContentWarning => write!(f, "content-warning"),
            Self::Expiration => write!(f, "expiration"),
            Self::Subject => write!(f, "subject"),
            Self::Challenge => write!(f, "challenge"),
            Self::Title => write!(f, "title"),
            Self::Image => write!(f, "image"),
            Self::Thumb => write!(f, "thumb"),
            Self::Summary => write!(f, "summary"),
            Self::PublishedAt => write!(f, "published_at"),
            Self::Description => write!(f, "description"),
            Self::Bolt11 => write!(f, "bolt11"),
            Self::Preimage => write!(f, "preimage"),
            Self::Relays => write!(f, "relays"),
            Self::Amount => write!(f, "amount"),
            Self::Lnurl => write!(f, "lnurl"),
            Self::Name => write!(f, "name"),
            Self::Url => write!(f, "url"),
            Self::Aes256Gcm => write!(f, "aes-256-gcm"),
            Self::Size => write!(f, "size"),
            Self::Dim => write!(f, "dim"),
            Self::Magnet => write!(f, "magnet"),
            Self::Blurhash => write!(f, "blurhash"),
            Self::Streaming => write!(f, "streaming"),
            Self::Recording => write!(f, "recording"),
            Self::Starts => write!(f, "starts"),
            Self::Ends => write!(f, "ends"),
            Self::Status => write!(f, "status"),
            Self::CurrentParticipants => write!(f, "current_participants"),
            Self::TotalParticipants => write!(f, "total_participants"),
            Self::Method => write!(f, "method"),
            Self::Payload => write!(f, "payload"),
            Self::Anon => write!(f, "anon"),
            Self::Proxy => write!(f, "proxy"),
            Self::Emoji => write!(f, "emoji"),
            Self::Encrypted => write!(f, "encrypted"),
            Self::Request => write!(f, "request"),
            Self::Word => write!(f, "word"),
            Self::Custom(tag) => write!(f, "{tag}"),
        }
    }
}

impl<S> From<S> for TagKind
where
    S: AsRef<str>,
{
    fn from(tag: S) -> Self {
        match tag.as_ref() {
            "relay" => Self::Relay,
            "nonce" => Self::Nonce,
            "delegation" => Self::Delegation,
            "content-warning" => Self::ContentWarning,
            "expiration" => Self::Expiration,
            "subject" => Self::Subject,
            "challenge" => Self::Challenge,
            "title" => Self::Title,
            "image" => Self::Image,
            "thumb" => Self::Thumb,
            "summary" => Self::Summary,
            "published_at" => Self::PublishedAt,
            "description" => Self::Description,
            "bolt11" => Self::Bolt11,
            "preimage" => Self::Preimage,
            "relays" => Self::Relays,
            "amount" => Self::Amount,
            "lnurl" => Self::Lnurl,
            "name" => Self::Name,
            "url" => Self::Url,
            "aes-256-gcm" => Self::Aes256Gcm,
            "size" => Self::Size,
            "dim" => Self::Dim,
            "magnet" => Self::Magnet,
            "blurhash" => Self::Blurhash,
            "streaming" => Self::Streaming,
            "recording" => Self::Recording,
            "starts" => Self::Starts,
            "ends" => Self::Ends,
            "status" => Self::Status,
            "current_participants" => Self::CurrentParticipants,
            "total_participants" => Self::TotalParticipants,
            "method" => Self::Method,
            "payload" => Self::Payload,
            "anon" => Self::Anon,
            "proxy" => Self::Proxy,
            "emoji" => Self::Emoji,
            "encrypted" => Self::Encrypted,
            "request" => Self::Request,
            "word" => Self::Word,
            t => match SingleLetterTag::from_str(t) {
                Ok(s) => Self::SingleLetter(s),
                Err(..) => Self::Custom(t.to_owned()),
            },
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tag {
    Generic(TagKind, Vec<String>),
    Event {
        event_id: EventId,
        relay_url: Option<UncheckedUrl>,
        marker: Option<Marker>,
    },
    PublicKey {
        public_key: PublicKey,
        relay_url: Option<UncheckedUrl>,
        alias: Option<String>,
        /// Whether the p tag is an uppercase P or not
        uppercase: bool,
    },
    EventReport(EventId, Report),
    PubKeyReport(PublicKey, Report),
    PubKeyLiveEvent {
        public_key: PublicKey,
        relay_url: Option<UncheckedUrl>,
        marker: LiveEventMarker,
        proof: Option<Signature>,
    },
    Reference(String),
    RelayMetadata(UncheckedUrl, Option<RelayMetadata>),
    Hashtag(String),
    Geohash(String),
    Identifier(String),
    ExternalIdentity(Identity),
    A {
        coordinate: Coordinate,
        relay_url: Option<UncheckedUrl>,
    },
    Kind(Kind),
    Relay(UncheckedUrl),
    POW {
        nonce: u128,
        difficulty: u8,
    },
    Delegation {
        delegator: PublicKey,
        conditions: Conditions,
        sig: Signature,
    },
    ContentWarning {
        reason: Option<String>,
    },
    Expiration(Timestamp),
    Subject(String),
    Challenge(String),
    Title(String),
    Image(UncheckedUrl, Option<ImageDimensions>),
    Thumb(UncheckedUrl, Option<ImageDimensions>),
    Summary(String),
    Description(String),
    Bolt11(String),
    Preimage(String),
    Relays(Vec<UncheckedUrl>),
    Amount {
        millisats: u64,
        bolt11: Option<String>,
    },
    Lnurl(String),
    Name(String),
    PublishedAt(Timestamp),
    Url(Url),
    MimeType(String),
    Aes256Gcm {
        key: String,
        iv: String,
    },
    Sha256(Sha256Hash),
    Size(usize),
    Dim(ImageDimensions),
    Magnet(String),
    Blurhash(String),
    Streaming(UncheckedUrl),
    Recording(UncheckedUrl),
    Starts(Timestamp),
    Ends(Timestamp),
    LiveEventStatus(LiveEventStatus),
    CurrentParticipants(u64),
    TotalParticipants(u64),
    AbsoluteURL(UncheckedUrl),
    Method(HttpMethod),
    Payload(Sha256Hash),
    Anon {
        msg: Option<String>,
    },
    Proxy {
        id: String,
        protocol: Protocol,
    },
    Emoji {
        /// Name given for the emoji, which MUST be comprised of only alphanumeric characters and underscores
        shortcode: String,
        /// URL to the corresponding image file of the emoji
        url: UncheckedUrl,
    },
    Encrypted,
    Request(Event),
    DataVendingMachineStatus {
        status: DataVendingMachineStatus,
        extra_info: Option<String>,
    },
    Word(String),
}

impl Tag {
    /// Parse [`Tag`] from slice of string
    pub fn parse<S>(tag: &[S]) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let tag_len: usize = tag.len();
        let tag_kind: TagKind = match tag.first() {
            Some(kind) => TagKind::from(kind),
            None => return Err(Error::KindNotFound),
        };

        if tag_kind.eq(&TagKind::Relays) {
            // Relays vec is of unknown length so checked here based on kind
            let urls = tag
                .iter()
                .skip(1)
                .map(|u| UncheckedUrl::from(u.as_ref()))
                .collect::<Vec<UncheckedUrl>>();
            Ok(Self::Relays(urls))
        } else if tag_len == 1 {
            match tag_kind {
                TagKind::ContentWarning => Ok(Self::ContentWarning { reason: None }),
                TagKind::Anon => Ok(Self::Anon { msg: None }),
                TagKind::Encrypted => Ok(Self::Encrypted),
                _ => Ok(Self::Generic(tag_kind, Vec::new())),
            }
        } else if tag_len == 2 {
            let tag_1: &str = tag[1].as_ref();

            match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::A,
                    uppercase: false,
                }) => Ok(Self::A {
                    coordinate: Coordinate::from_str(tag_1)?,
                    relay_url: None,
                }),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::P,
                    uppercase,
                }) => {
                    let public_key = PublicKey::from_str(tag_1)?;
                    Ok(Self::PublicKey {
                        public_key,
                        relay_url: None,
                        alias: None,
                        uppercase,
                    })
                }
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::E,
                    uppercase: false,
                }) => Ok(Self::event(EventId::from_hex(tag_1)?)),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::R,
                    uppercase: false,
                }) => {
                    if tag_1.starts_with("ws://") || tag_1.starts_with("wss://") {
                        Ok(Self::RelayMetadata(UncheckedUrl::from(tag_1), None))
                    } else {
                        Ok(Self::Reference(tag_1.to_owned()))
                    }
                }
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::T,
                    uppercase: false,
                }) => Ok(Self::Hashtag(tag_1.to_owned())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::G,
                    uppercase: false,
                }) => Ok(Self::Geohash(tag_1.to_owned())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }) => Ok(Self::Identifier(tag_1.to_owned())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::K,
                    uppercase: false,
                }) => Ok(parse_or_generic::<Kind>(tag_kind, tag_1)),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::M,
                    uppercase: false,
                }) => Ok(Self::MimeType(tag_1.to_owned())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::X,
                    uppercase: false,
                }) => Ok(parse_or_generic::<Sha256Hash>(tag_kind, tag_1)),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::U,
                    uppercase: false,
                }) => Ok(Self::AbsoluteURL(UncheckedUrl::from(tag_1))),
                TagKind::Relay => Ok(Self::Relay(UncheckedUrl::from(tag_1))),
                TagKind::ContentWarning => Ok(Self::ContentWarning {
                    reason: Some(tag_1.to_owned()),
                }),
                TagKind::Expiration => Ok(Self::Expiration(Timestamp::from_str(tag_1)?)),
                TagKind::Subject => Ok(Self::Subject(tag_1.to_owned())),
                TagKind::Challenge => Ok(Self::Challenge(tag_1.to_owned())),
                TagKind::Title => Ok(Self::Title(tag_1.to_owned())),
                TagKind::Image => Ok(Self::Image(UncheckedUrl::from(tag_1), None)),
                TagKind::Thumb => Ok(Self::Thumb(UncheckedUrl::from(tag_1), None)),
                TagKind::Summary => Ok(Self::Summary(tag_1.to_owned())),
                TagKind::PublishedAt => Ok(Self::PublishedAt(Timestamp::from_str(tag_1)?)),
                TagKind::Description => Ok(Self::Description(tag_1.to_owned())),
                TagKind::Bolt11 => Ok(Self::Bolt11(tag_1.to_owned())),
                TagKind::Preimage => Ok(Self::Preimage(tag_1.to_owned())),
                TagKind::Amount => Ok(Self::Amount {
                    millisats: tag_1.parse()?,
                    bolt11: None,
                }),
                TagKind::Lnurl => Ok(Self::Lnurl(tag_1.to_owned())),
                TagKind::Name => Ok(Self::Name(tag_1.to_owned())),
                TagKind::Url => Ok(Self::Url(Url::parse(tag_1)?)),
                TagKind::Magnet => Ok(Self::Magnet(tag_1.to_owned())),
                TagKind::Blurhash => Ok(Self::Blurhash(tag_1.to_owned())),
                TagKind::Streaming => Ok(Self::Streaming(UncheckedUrl::from(tag_1))),
                TagKind::Recording => Ok(Self::Recording(UncheckedUrl::from(tag_1))),
                TagKind::Starts => Ok(Self::Starts(Timestamp::from_str(tag_1)?)),
                TagKind::Ends => Ok(Self::Ends(Timestamp::from_str(tag_1)?)),
                TagKind::Status => match DataVendingMachineStatus::from_str(tag_1) {
                    Ok(status) => Ok(Self::DataVendingMachineStatus {
                        status,
                        extra_info: None,
                    }),
                    Err(_) => Ok(Self::LiveEventStatus(LiveEventStatus::from(tag_1))), /* TODO: check if unknown status error? */
                },
                TagKind::CurrentParticipants => Ok(Self::CurrentParticipants(tag_1.parse()?)),
                TagKind::TotalParticipants => Ok(Self::TotalParticipants(tag_1.parse()?)),
                TagKind::Method => Ok(Self::Method(HttpMethod::from_str(tag_1)?)),
                TagKind::Payload => Ok(Self::Payload(Sha256Hash::from_str(tag_1)?)),
                TagKind::Anon => Ok(Self::Anon {
                    msg: (!tag_1.is_empty()).then_some(tag_1.to_owned()),
                }),
                TagKind::Request => Ok(Self::Request(Event::from_json(tag_1)?)),
                TagKind::Word => Ok(Self::Word(tag_1.to_string())),
                _ => Ok(Self::Generic(tag_kind, vec![tag_1.to_owned()])),
            }
        } else if tag_len == 3 {
            let tag_1: &str = tag[1].as_ref();
            let tag_2: &str = tag[2].as_ref();

            match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::P,
                    uppercase: false,
                }) => {
                    let public_key = PublicKey::from_str(tag_1)?;
                    if tag_2.is_empty() {
                        Ok(Self::PublicKey {
                            public_key,
                            relay_url: Some(UncheckedUrl::empty()),
                            alias: None,
                            uppercase: false,
                        })
                    } else {
                        match Report::from_str(tag_2) {
                            Ok(report) => Ok(Self::PubKeyReport(public_key, report)),
                            Err(_) => Ok(Self::PublicKey {
                                public_key,
                                relay_url: Some(UncheckedUrl::from(tag_2)),
                                alias: None,
                                uppercase: false,
                            }),
                        }
                    }
                }
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::E,
                    uppercase: false,
                }) => {
                    let event_id = EventId::from_hex(tag_1)?;
                    if tag_2.is_empty() {
                        Ok(Self::Event {
                            event_id,
                            relay_url: Some(UncheckedUrl::empty()),
                            marker: None,
                        })
                    } else {
                        match Report::from_str(tag_2) {
                            Ok(report) => Ok(Self::EventReport(event_id, report)),
                            Err(_) => Ok(Self::Event {
                                event_id,
                                relay_url: Some(UncheckedUrl::from(tag_2)),
                                marker: None,
                            }),
                        }
                    }
                }
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::I,
                    uppercase: false,
                }) => match Identity::new(tag_1, tag_2) {
                    Ok(identity) => Ok(Self::ExternalIdentity(identity)),
                    Err(_) => Ok(Self::Generic(
                        tag_kind,
                        tag[1..].iter().map(|s| s.as_ref().to_owned()).collect(),
                    )),
                },
                TagKind::Nonce => Ok(Self::POW {
                    nonce: tag_1.parse()?,
                    difficulty: tag_2.parse()?,
                }),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::A,
                    uppercase: false,
                }) => Ok(Self::A {
                    coordinate: Coordinate::from_str(tag_1)?,
                    relay_url: Some(UncheckedUrl::from(tag_2)),
                }),
                TagKind::Image => Ok(Self::Image(
                    UncheckedUrl::from(tag_1),
                    Some(ImageDimensions::from_str(tag_2)?),
                )),
                TagKind::Thumb => Ok(Self::Thumb(
                    UncheckedUrl::from(tag_1),
                    Some(ImageDimensions::from_str(tag_2)?),
                )),
                TagKind::Aes256Gcm => Ok(Self::Aes256Gcm {
                    key: tag_1.to_owned(),
                    iv: tag_2.to_owned(),
                }),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::R,
                    uppercase: false,
                }) => Ok(Self::RelayMetadata(
                    UncheckedUrl::from(tag_1),
                    Some(RelayMetadata::from_str(tag_2)?),
                )),
                TagKind::Proxy => Ok(Self::Proxy {
                    id: tag_1.to_owned(),
                    protocol: Protocol::from(tag_2),
                }),
                TagKind::Emoji => Ok(Self::Emoji {
                    shortcode: tag_1.to_owned(),
                    url: UncheckedUrl::from(tag_2),
                }),
                TagKind::Status => match DataVendingMachineStatus::from_str(tag_1) {
                    Ok(status) => Ok(Self::DataVendingMachineStatus {
                        status,
                        extra_info: Some(tag_2.to_string()),
                    }),
                    Err(_) => Ok(Self::Generic(
                        tag_kind,
                        tag[1..].iter().map(|s| s.as_ref().to_owned()).collect(),
                    )),
                },
                _ => Ok(Self::Generic(
                    tag_kind,
                    tag[1..].iter().map(|s| s.as_ref().to_owned()).collect(),
                )),
            }
        } else if tag_len == 4 {
            let tag_1: &str = tag[1].as_ref();
            let tag_2: &str = tag[2].as_ref();
            let tag_3: &str = tag[3].as_ref();

            match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::P,
                    uppercase,
                }) => {
                    let public_key: PublicKey = PublicKey::from_str(tag_1)?;
                    let relay_url: Option<UncheckedUrl> = Some(UncheckedUrl::from(tag_2));

                    match LiveEventMarker::from_str(tag_3) {
                        Ok(marker) => Ok(Self::PubKeyLiveEvent {
                            public_key,
                            relay_url,
                            marker,
                            proof: None,
                        }),
                        Err(_) => Ok(Self::PublicKey {
                            public_key,
                            relay_url,
                            alias: Some(tag_3.to_string()),
                            uppercase,
                        }),
                    }
                }
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::E,
                    uppercase: false,
                }) => Ok(Self::Event {
                    event_id: EventId::from_hex(tag_1)?,
                    relay_url: (!tag_2.is_empty()).then_some(UncheckedUrl::from(tag_2)),
                    marker: (!tag_3.is_empty()).then_some(Marker::from(tag_3)),
                }),
                TagKind::Delegation => Ok(Self::Delegation {
                    delegator: PublicKey::from_str(tag_1)?,
                    conditions: Conditions::from_str(tag_2)?,
                    sig: Signature::from_str(tag_3)?,
                }),
                _ => Ok(Self::Generic(
                    tag_kind,
                    tag[1..].iter().map(|s| s.as_ref().to_owned()).collect(),
                )),
            }
        } else if tag_len == 5 {
            let tag_1: &str = tag[1].as_ref();
            let tag_2: &str = tag[2].as_ref();
            let tag_3: &str = tag[3].as_ref();
            let tag_4: &str = tag[4].as_ref();

            match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::P,
                    ..
                }) => Ok(Self::PubKeyLiveEvent {
                    public_key: PublicKey::from_str(tag_1)?,
                    relay_url: (!tag_2.is_empty()).then_some(UncheckedUrl::from(tag_2)),
                    marker: LiveEventMarker::from_str(tag_3)?,
                    proof: Signature::from_str(tag_4).ok(),
                }),
                _ => Ok(Self::Generic(
                    tag_kind,
                    tag[1..].iter().map(|s| s.as_ref().to_owned()).collect(),
                )),
            }
        } else {
            Ok(Self::Generic(
                tag_kind,
                tag[1..].iter().map(|s| s.as_ref().to_owned()).collect(),
            ))
        }
    }

    /// Compose `Tag::Event` without `relay_url` and `marker`
    ///
    /// JSON: `["e", "event-id"]`
    #[inline]
    pub fn event(event_id: EventId) -> Self {
        Self::Event {
            event_id,
            relay_url: None,
            marker: None,
        }
    }

    /// Compose `Tag::PublicKey` without `relay_url` and `alias`
    ///
    /// JSON: `["p", "<public-key>"]`
    #[inline]
    pub fn public_key(public_key: PublicKey) -> Self {
        Self::PublicKey {
            public_key,
            relay_url: None,
            alias: None,
            uppercase: false,
        }
    }

    /// Check if [Tag] is an event `reply`
    #[inline]
    pub fn is_reply(&self) -> bool {
        matches!(
            self,
            Tag::Event {
                marker: Some(Marker::Reply),
                ..
            }
        )
    }

    /// Get [`Tag`] as string vector
    ///
    /// Internally clone tag and convert it to `Vec<String>`. To avoid tag clone, use `Tag::to_vec`.
    #[inline]
    pub fn as_vec(&self) -> Vec<String> {
        self.clone().into()
    }

    /// Consume [`Tag`] and return string vector
    #[inline]
    pub fn to_vec(self) -> Vec<String> {
        self.into()
    }

    /// Get tag kind
    pub fn kind(&self) -> TagKind {
        match self {
            Self::Generic(kind, ..) => kind.clone(),
            Self::Event { .. } | Self::EventReport(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::E,
                uppercase: false,
            }),
            Self::PublicKey { uppercase, .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::P,
                uppercase: *uppercase,
            }),
            Self::PubKeyReport(..) | Self::PubKeyLiveEvent { .. } => {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::P,
                    uppercase: false,
                })
            }
            Self::Reference(..) | Self::RelayMetadata(..) => {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::R,
                    uppercase: false,
                })
            }
            Self::Hashtag(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::T,
                uppercase: false,
            }),
            Self::Geohash(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::G,
                uppercase: false,
            }),
            Self::Identifier(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::D,
                uppercase: false,
            }),
            Self::ExternalIdentity(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::I,
                uppercase: false,
            }),
            Self::A { .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::A,
                uppercase: false,
            }),
            Self::Kind(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::K,
                uppercase: false,
            }),
            Self::Relay(..) => TagKind::Relay,
            Self::POW { .. } => TagKind::Nonce,
            Self::Delegation { .. } => TagKind::Delegation,
            Self::ContentWarning { .. } => TagKind::ContentWarning,
            Self::Expiration(..) => TagKind::Expiration,
            Self::Subject(..) => TagKind::Subject,
            Self::Challenge(..) => TagKind::Challenge,
            Self::Title(..) => TagKind::Title,
            Self::Image(..) => TagKind::Image,
            Self::Thumb(..) => TagKind::Thumb,
            Self::Summary(..) => TagKind::Summary,
            Self::PublishedAt(..) => TagKind::PublishedAt,
            Self::Description(..) => TagKind::Description,
            Self::Bolt11(..) => TagKind::Bolt11,
            Self::Preimage(..) => TagKind::Preimage,
            Self::Relays(..) => TagKind::Relays,
            Self::Amount { .. } => TagKind::Amount,
            Self::Name(..) => TagKind::Name,
            Self::Lnurl(..) => TagKind::Lnurl,
            Self::Url(..) => TagKind::Url,
            Self::MimeType(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::M,
                uppercase: false,
            }),
            Self::Aes256Gcm { .. } => TagKind::Aes256Gcm,
            Self::Sha256(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::X,
                uppercase: false,
            }),
            Self::Size(..) => TagKind::Size,
            Self::Dim(..) => TagKind::Dim,
            Self::Magnet(..) => TagKind::Magnet,
            Self::Blurhash(..) => TagKind::Blurhash,
            Self::Streaming(..) => TagKind::Streaming,
            Self::Recording(..) => TagKind::Recording,
            Self::Starts(..) => TagKind::Starts,
            Self::Ends(..) => TagKind::Ends,
            Self::LiveEventStatus(..) | Self::DataVendingMachineStatus { .. } => TagKind::Status,
            Self::CurrentParticipants(..) => TagKind::CurrentParticipants,
            Self::TotalParticipants(..) => TagKind::TotalParticipants,
            Self::AbsoluteURL(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::U,
                uppercase: false,
            }),
            Self::Method(..) => TagKind::Method,
            Self::Payload(..) => TagKind::Payload,
            Self::Anon { .. } => TagKind::Anon,
            Self::Proxy { .. } => TagKind::Proxy,
            Self::Emoji { .. } => TagKind::Emoji,
            Self::Encrypted => TagKind::Encrypted,
            Self::Request(..) => TagKind::Request,
            Self::Word(..) => TagKind::Word,
        }
    }

    /// Get [SingleLetterTag]
    #[inline]
    pub fn single_letter_tag(&self) -> Option<SingleLetterTag> {
        match self.kind() {
            TagKind::SingleLetter(s) => Some(s),
            _ => None,
        }
    }

    /// Return the **first** tag value (index `1`), if exists.
    pub fn content(&self) -> Option<GenericTagValue> {
        match self {
            Self::Generic(_, l) => l.first().map(|v| v.into_generic_tag_value()),
            Self::Event { event_id, .. } | Self::EventReport(event_id, ..) => {
                Some((*event_id).into_generic_tag_value())
            }
            Self::PublicKey { public_key, .. }
            | Self::PubKeyReport(public_key, ..)
            | Self::PubKeyLiveEvent { public_key, .. } => {
                Some((*public_key).into_generic_tag_value())
            }
            Self::Reference(val) => Some(val.into_generic_tag_value()),
            Self::RelayMetadata(url, ..) => Some(url.to_string().into_generic_tag_value()),
            Self::Hashtag(val) => Some(val.into_generic_tag_value()),
            Self::Geohash(val, ..) => Some(val.into_generic_tag_value()),
            Self::Identifier(val, ..) => Some(val.into_generic_tag_value()),
            Self::ExternalIdentity(..) => None,
            Self::A { coordinate, .. } => Some(coordinate.to_string().into_generic_tag_value()),
            Self::Kind(kind) => Some(kind.to_string().into_generic_tag_value()),
            Self::Relay(url) => Some(url.to_string().into_generic_tag_value()),
            Self::POW { nonce, .. } => Some(nonce.to_string().into_generic_tag_value()),
            Self::Delegation { delegator, .. } => Some(delegator.into_generic_tag_value()),
            Self::ContentWarning { reason } => reason.clone().map(GenericTagValue::String),
            Self::Expiration(timestamp) => Some(timestamp.to_string().into_generic_tag_value()),
            Self::Subject(val) => Some(val.into_generic_tag_value()),
            Self::Challenge(val) => Some(val.into_generic_tag_value()),
            Self::Title(val) => Some(val.into_generic_tag_value()),
            Self::Image(url, ..) => Some(url.to_string().into_generic_tag_value()),
            Self::Thumb(url, ..) => Some(url.to_string().into_generic_tag_value()),
            Self::Summary(val) => Some(val.into_generic_tag_value()),
            Self::PublishedAt(timestamp) => Some(timestamp.to_string().into_generic_tag_value()),
            Self::Description(val) => Some(val.into_generic_tag_value()),
            Self::Bolt11(val) => Some(val.into_generic_tag_value()),
            Self::Preimage(val) => Some(val.into_generic_tag_value()),
            Self::Relays(list) => list.first().map(|r| r.to_string().into_generic_tag_value()),
            Self::Amount { millisats, .. } => Some(millisats.to_string().into_generic_tag_value()),
            Self::Name(val) => Some(val.into_generic_tag_value()),
            Self::Lnurl(val) => Some(val.into_generic_tag_value()),
            Self::Url(url) => Some(url.to_string().into_generic_tag_value()),
            Self::MimeType(val) => Some(val.into_generic_tag_value()),
            Self::Aes256Gcm { key, .. } => Some(key.clone().into_generic_tag_value()),
            Self::Sha256(val) => Some(val.to_string().into_generic_tag_value()),
            Self::Size(val) => Some(val.to_string().into_generic_tag_value()),
            Self::Dim(val) => Some(val.to_string().into_generic_tag_value()),
            Self::Magnet(val) => Some(val.into_generic_tag_value()),
            Self::Blurhash(val) => Some(val.into_generic_tag_value()),
            Self::Streaming(url) => Some(url.to_string().into_generic_tag_value()),
            Self::Recording(url) => Some(url.to_string().into_generic_tag_value()),
            Self::Starts(timestamp) => Some(timestamp.to_string().into_generic_tag_value()),
            Self::Ends(timestamp) => Some(timestamp.to_string().into_generic_tag_value()),
            Self::LiveEventStatus(val) => Some(val.to_string().into_generic_tag_value()),
            Self::DataVendingMachineStatus { status, .. } => {
                Some(status.to_string().into_generic_tag_value())
            }
            Self::CurrentParticipants(num) => Some(num.to_string().into_generic_tag_value()),
            Self::TotalParticipants(num) => Some(num.to_string().into_generic_tag_value()),
            Self::AbsoluteURL(url) => Some(url.to_string().into_generic_tag_value()),
            Self::Method(val) => Some(val.to_string().into_generic_tag_value()),
            Self::Payload(val) => Some(val.to_string().into_generic_tag_value()),
            Self::Anon { msg } => msg.clone().map(GenericTagValue::String),
            Self::Proxy { id, .. } => Some(id.into_generic_tag_value()),
            Self::Emoji { shortcode, .. } => Some(shortcode.into_generic_tag_value()),
            Self::Encrypted => None,
            Self::Request(val) => Some(val.as_json().into_generic_tag_value()),
            Self::Word(val) => Some(val.into_generic_tag_value()),
        }
    }
}

impl From<Tag> for Vec<String> {
    fn from(tag: Tag) -> Self {
        let tag_kind: TagKind = tag.kind();

        match tag {
            Tag::Generic(kind, data) => [vec![kind.to_string()], data].concat(),
            Tag::Event {
                event_id,
                relay_url,
                marker,
            } => {
                let mut tag = vec![tag_kind.to_string(), event_id.to_hex()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url.to_string());
                }
                if let Some(marker) = marker {
                    if tag.len() == 2 {
                        tag.push(String::new());
                    }
                    tag.push(marker.to_string());
                }
                tag
            }
            Tag::PublicKey {
                public_key,
                relay_url,
                alias,
                ..
            } => {
                let mut tag = vec![tag_kind.to_string(), public_key.to_string()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url.to_string());
                }
                if let Some(alias) = alias {
                    tag.push(alias);
                }
                tag
            }
            Tag::EventReport(id, report) => {
                vec![tag_kind.to_string(), id.to_hex(), report.to_string()]
            }
            Tag::PubKeyReport(pk, report) => {
                vec![tag_kind.to_string(), pk.to_string(), report.to_string()]
            }
            Tag::PubKeyLiveEvent {
                public_key,
                relay_url,
                marker,
                proof,
            } => {
                let mut tag = vec![
                    tag_kind.to_string(),
                    public_key.to_string(),
                    relay_url.map(|u| u.to_string()).unwrap_or_default(),
                    marker.to_string(),
                ];
                if let Some(proof) = proof {
                    tag.push(proof.to_string());
                }
                tag
            }
            Tag::Reference(r) => vec![tag_kind.to_string(), r],
            Tag::RelayMetadata(url, rw) => {
                let mut tag = vec![tag_kind.to_string(), url.to_string()];
                if let Some(rw) = rw {
                    tag.push(rw.to_string());
                }
                tag
            }
            Tag::Hashtag(t) => vec![tag_kind.to_string(), t],
            Tag::Geohash(g) => vec![tag_kind.to_string(), g],
            Tag::Identifier(d) => vec![tag_kind.to_string(), d],
            Tag::A {
                coordinate,
                relay_url,
            } => {
                let mut vec = vec![tag_kind.to_string(), coordinate.to_string()];
                if let Some(relay) = relay_url {
                    vec.push(relay.to_string());
                }
                vec
            }
            Tag::ExternalIdentity(identity) => identity.into(),
            Tag::Kind(kind) => vec![tag_kind.to_string(), kind.to_string()],
            Tag::Relay(url) => vec![tag_kind.to_string(), url.to_string()],
            Tag::POW { nonce, difficulty } => vec![
                tag_kind.to_string(),
                nonce.to_string(),
                difficulty.to_string(),
            ],
            Tag::Delegation {
                delegator,
                conditions,
                sig,
            } => vec![
                tag_kind.to_string(),
                delegator.to_string(),
                conditions.to_string(),
                sig.to_string(),
            ],
            Tag::ContentWarning { reason } => {
                let mut tag = vec![tag_kind.to_string()];
                if let Some(reason) = reason {
                    tag.push(reason);
                }
                tag
            }
            Tag::Expiration(timestamp) => {
                vec![tag_kind.to_string(), timestamp.to_string()]
            }
            Tag::Subject(sub) => vec![tag_kind.to_string(), sub],
            Tag::Challenge(challenge) => vec![tag_kind.to_string(), challenge],
            Tag::Title(title) => vec![tag_kind.to_string(), title],
            Tag::Image(image, dimensions) => {
                let mut tag = vec![tag_kind.to_string(), image.to_string()];
                if let Some(dim) = dimensions {
                    tag.push(dim.to_string());
                }
                tag
            }
            Tag::Thumb(thumb, dimensions) => {
                let mut tag = vec![tag_kind.to_string(), thumb.to_string()];
                if let Some(dim) = dimensions {
                    tag.push(dim.to_string());
                }
                tag
            }
            Tag::Summary(summary) => vec![tag_kind.to_string(), summary],
            Tag::PublishedAt(timestamp) => {
                vec![tag_kind.to_string(), timestamp.to_string()]
            }
            Tag::Description(description) => {
                vec![tag_kind.to_string(), description]
            }
            Tag::Bolt11(bolt11) => {
                vec![tag_kind.to_string(), bolt11]
            }
            Tag::Preimage(preimage) => {
                vec![tag_kind.to_string(), preimage]
            }
            Tag::Relays(relays) => vec![tag_kind.to_string()]
                .into_iter()
                .chain(relays.iter().map(|relay| relay.to_string()))
                .collect::<Vec<_>>(),
            Tag::Amount { millisats, bolt11 } => {
                let mut tag = vec![tag_kind.to_string(), millisats.to_string()];
                if let Some(bolt11) = bolt11 {
                    tag.push(bolt11);
                }
                tag
            }
            Tag::Name(name) => {
                vec![tag_kind.to_string(), name]
            }
            Tag::Lnurl(lnurl) => {
                vec![tag_kind.to_string(), lnurl]
            }
            Tag::Url(url) => vec![tag_kind.to_string(), url.to_string()],
            Tag::MimeType(mime) => vec![tag_kind.to_string(), mime],
            Tag::Aes256Gcm { key, iv } => vec![tag_kind.to_string(), key, iv],
            Tag::Sha256(hash) => vec![tag_kind.to_string(), hash.to_string()],
            Tag::Size(bytes) => vec![tag_kind.to_string(), bytes.to_string()],
            Tag::Dim(dim) => vec![tag_kind.to_string(), dim.to_string()],
            Tag::Magnet(uri) => vec![tag_kind.to_string(), uri],
            Tag::Blurhash(data) => vec![tag_kind.to_string(), data],
            Tag::Streaming(url) => vec![tag_kind.to_string(), url.to_string()],
            Tag::Recording(url) => vec![tag_kind.to_string(), url.to_string()],
            Tag::Starts(timestamp) => {
                vec![tag_kind.to_string(), timestamp.to_string()]
            }
            Tag::Ends(timestamp) => {
                vec![tag_kind.to_string(), timestamp.to_string()]
            }
            Tag::LiveEventStatus(s) => {
                vec![tag_kind.to_string(), s.to_string()]
            }
            Tag::CurrentParticipants(num) => {
                vec![tag_kind.to_string(), num.to_string()]
            }
            Tag::TotalParticipants(num) => {
                vec![tag_kind.to_string(), num.to_string()]
            }
            Tag::AbsoluteURL(url) => {
                vec![tag_kind.to_string(), url.to_string()]
            }
            Tag::Method(method) => {
                vec![tag_kind.to_string(), method.to_string()]
            }
            Tag::Payload(p) => vec![tag_kind.to_string(), p.to_string()],
            Tag::Anon { msg } => {
                let mut tag = vec![tag_kind.to_string()];
                if let Some(msg) = msg {
                    tag.push(msg);
                }
                tag
            }
            Tag::Proxy { id, protocol } => {
                vec![tag_kind.to_string(), id, protocol.to_string()]
            }
            Tag::Emoji { shortcode, url } => {
                vec![tag_kind.to_string(), shortcode, url.to_string()]
            }
            Tag::Encrypted => vec![tag_kind.to_string()],
            Tag::Request(event) => vec![tag_kind.to_string(), event.as_json()],
            Tag::DataVendingMachineStatus { status, extra_info } => {
                let mut tag = vec![tag_kind.to_string(), status.to_string()];
                if let Some(extra_info) = extra_info {
                    tag.push(extra_info);
                }
                tag
            }
            Tag::Word(word) => vec![TagKind::Word.to_string(), word],
        }
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data: Vec<String> = self.as_vec();
        let mut seq = serializer.serialize_seq(Some(data.len()))?;
        for element in data.into_iter() {
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        type Data = Vec<String>;
        let tag: Vec<String> = Data::deserialize(deserializer)?;
        Self::parse(&tag).map_err(DeserializerError::custom)
    }
}

/// Supported external identity providers
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl fmt::Display for ExternalIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::Twitter => write!(f, "twitter"),
            Self::Mastodon => write!(f, "mastodon"),
            Self::Telegram => write!(f, "telegram"),
        }
    }
}

impl TryFrom<String> for ExternalIdentity {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "github" => Ok(Self::GitHub),
            "twitter" => Ok(Self::Twitter),
            "mastodon" => Ok(Self::Mastodon),
            "telegram" => Ok(Self::Telegram),
            _ => Err(Error::InvalidIdentity),
        }
    }
}

/// A NIP-39 external identity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity {
    /// The external identity provider
    pub platform: ExternalIdentity,
    /// The user's identity (username) on the provider
    pub ident: String,
    /// The user's proof on the provider
    pub proof: String,
}

impl Identity {
    /// New [`Identity`]
    pub fn new<S>(platform_iden: S, proof: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let i: String = platform_iden.into();
        let (platform, ident) = i.rsplit_once(':').ok_or(Error::InvalidIdentity)?;
        let platform: ExternalIdentity = platform.to_string().try_into()?;

        Ok(Self {
            platform,
            ident: ident.to_string(),
            proof: proof.into(),
        })
    }
}

impl TryFrom<Tag> for Identity {
    type Error = Error;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::ExternalIdentity(iden) => Ok(iden),
            _ => Err(Error::InvalidIdentity),
        }
    }
}

impl From<Identity> for Tag {
    fn from(value: Identity) -> Self {
        Self::ExternalIdentity(value)
    }
}

impl From<Identity> for Vec<String> {
    fn from(value: Identity) -> Self {
        vec![
            TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::I,
                uppercase: false,
            })
            .to_string(),
            format!("{}:{}", value.platform, value.ident),
            value.proof,
        ]
    }
}

/// Try to parse string. If fails, return `Tag::Generic`
#[inline]
fn parse_or_generic<T>(kind: TagKind, val: &str) -> Tag
where
    T: FromStr + Into<Tag>,
{
    match T::from_str(val) {
        Ok(res) => res.into(),
        Err(..) => Tag::Generic(kind, vec![val.to_string()]),
    }
}

impl From<Kind> for Tag {
    fn from(value: Kind) -> Self {
        Self::Kind(value)
    }
}

impl From<Sha256Hash> for Tag {
    fn from(value: Sha256Hash) -> Self {
        Self::Sha256(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, JsonUtil, Timestamp};

    #[test]
    fn test_tag_is_reply() {
        let tag = Tag::Relay(UncheckedUrl::new("wss://relay.damus.io"));
        assert!(!tag.is_reply());

        let tag = Tag::Event {
            event_id: EventId::from_hex(
                "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
            )
            .unwrap(),
            relay_url: None,
            marker: Some(Marker::Reply),
        };
        assert!(tag.is_reply());

        let tag = Tag::Event {
            event_id: EventId::from_hex(
                "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
            )
            .unwrap(),
            relay_url: None,
            marker: Some(Marker::Root),
        };
        assert!(!tag.is_reply());
    }

    #[test]
    fn test_extract_tag_content() {
        let t: Tag = Tag::parse(&["aaaaaa", "bbbbbb"]).unwrap();
        assert_eq!(
            t.content(),
            Some(GenericTagValue::String(String::from("bbbbbb")))
        );

        // Test extract public key
        let t: Tag = Tag::parse(&[
            "custom-p",
            "f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785",
        ])
        .unwrap();
        assert_eq!(
            t.content(),
            Some(GenericTagValue::String(String::from(
                "f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785"
            )))
        );

        // Test extract event ID
        let t: Tag = Tag::parse(&[
            "custom-e",
            "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
        ])
        .unwrap();
        assert_eq!(
            t.content(),
            Some(GenericTagValue::String(String::from(
                "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45"
            )))
        );
    }

    #[test]
    fn test_deserialize_tag_from_event() {
        // Got this fresh off the wire
        let event: &str = r#"{"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","created_at":1640839235,"kind":4,"tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]],"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"}"#;
        let event = Event::from_json(event).unwrap();
        let tag = event.tags().first().unwrap();

        assert_eq!(
            tag,
            &Tag::public_key(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn test_serialize_tag_to_event() {
        let public_key =
            PublicKey::from_str("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();
        let event = Event::new(
            EventId::from_str("378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7")
                .unwrap(),
            PublicKey::from_str("79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3").unwrap(),
            Timestamp::from(1671739153),
            Kind::EncryptedDirectMessage,
            [Tag::public_key(public_key)],
            "8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==",
            Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap()
        );

        let event_json: &str = r#"{"id":"378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1671739153,"kind":4,"tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"]],"content":"8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==","sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8"}"#;

        assert_eq!(&event.as_json(), event_json);
    }

    #[test]
    fn test_tag_as_vec() {
        assert_eq!(
            vec!["content-warning"],
            Tag::ContentWarning { reason: None }.as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ],
            Tag::public_key(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap()
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ],
            Tag::event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap()
            )
            .as_vec()
        );

        assert_eq!(
            vec!["expiration", "1600000000"],
            Tag::Expiration(Timestamp::from(1600000000)).as_vec()
        );

        assert_eq!(
            vec!["content-warning", "reason"],
            Tag::ContentWarning {
                reason: Some(String::from("reason"))
            }
            .as_vec()
        );

        assert_eq!(
            vec!["subject", "textnote with subject"],
            Tag::Subject(String::from("textnote with subject")).as_vec()
        );

        assert_eq!(
            vec!["client", "nostr-sdk"],
            Tag::Generic(
                TagKind::Custom("client".to_string()),
                vec!["nostr-sdk".to_string()]
            )
            .as_vec()
        );

        assert_eq!(
            vec!["d", "test"],
            Tag::Identifier("test".to_string()).as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ],
            Tag::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: None,
                uppercase: false,
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ],
            Tag::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::empty()),
                marker: None
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ],
            Tag::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: None
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "spam"
            ],
            Tag::PubKeyReport(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Spam
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "nudity"
            ],
            Tag::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Nudity,
            )
            .as_vec()
        );

        assert_eq!(
            vec!["nonce", "1", "20"],
            Tag::POW {
                nonce: 1,
                difficulty: 20
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ],
            Tag::A {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(UncheckedUrl::from_str("wss://relay.nostr.org").unwrap())
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "Speaker",
            ],
            Tag::PubKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: LiveEventMarker::Speaker,
                proof: None
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "",
                "Participant",
            ],
            Tag::PubKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: None,
                marker: LiveEventMarker::Participant,
                proof: None
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "alias",
            ],
            Tag::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: Some(String::from("alias")),
                uppercase: false,
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ],
            Tag::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply)
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ],
            Tag::Delegation { delegator: PublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ).unwrap(), conditions: Conditions::from_str("kind=1").unwrap(), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap() }
            .as_vec()
        );

        assert_eq!(
            vec!["lnurl", "lnurl1dp68gurn8ghj7um5v93kketj9ehx2amn9uh8wetvdskkkmn0wahz7mrww4excup0dajx2mrv92x9xp"],
            Tag::Lnurl(String::from("lnurl1dp68gurn8ghj7um5v93kketj9ehx2amn9uh8wetvdskkkmn0wahz7mrww4excup0dajx2mrv92x9xp")).as_vec(),
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "Host",
                "a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"
            ],
            Tag::PubKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                ).unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: LiveEventMarker::Host,
                proof: Some(Signature::from_str("a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd").unwrap())
            }
            .as_vec()
        );
    }

    #[test]
    fn test_tag_parser() {
        match Tag::parse::<String>(&[]) {
            Err(Error::KindNotFound) => (),
            _ => panic!(),
        }

        assert_eq!(
            Tag::parse(&["content-warning"]).unwrap(),
            Tag::ContentWarning { reason: None }
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])
            .unwrap(),
            Tag::public_key(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap()
            )
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ])
            .unwrap(),
            Tag::event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap()
            )
        );

        assert_eq!(
            Tag::parse(&["expiration", "1600000000"]).unwrap(),
            Tag::Expiration(Timestamp::from(1600000000))
        );

        assert_eq!(
            Tag::parse(&["content-warning", "reason"]).unwrap(),
            Tag::ContentWarning {
                reason: Some(String::from("reason"))
            }
        );

        assert_eq!(
            Tag::parse(&["subject", "textnote with subject"]).unwrap(),
            Tag::Subject(String::from("textnote with subject"))
        );

        assert_eq!(
            Tag::parse(&["client", "nostr-sdk"]).unwrap(),
            Tag::Generic(
                TagKind::Custom("client".to_string()),
                vec!["nostr-sdk".to_string()]
            )
        );

        assert_eq!(
            Tag::parse(&["d", "test"]).unwrap(),
            Tag::Identifier("test".to_string())
        );

        assert_eq!(
            Tag::parse(&["r", "https://example.com",]).unwrap(),
            Tag::Reference(String::from("https://example.com"),)
        );

        assert_eq!(
            Tag::parse(&["r", "wss://alicerelay.example.com",]).unwrap(),
            Tag::RelayMetadata(UncheckedUrl::from("wss://alicerelay.example.com"), None)
        );

        assert_eq!(
            Tag::parse(&["i", "github:12345678", "abcdefghijklmnop"]).unwrap(),
            Tag::ExternalIdentity(Identity {
                platform: ExternalIdentity::GitHub,
                ident: "12345678".to_string(),
                proof: "abcdefghijklmnop".to_string()
            })
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            Tag::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: None,
                uppercase: false
            }
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ])
            .unwrap(),
            Tag::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::empty()),
                marker: None
            }
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            Tag::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: None
            }
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "impersonation"
            ])
            .unwrap(),
            Tag::PubKeyReport(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Impersonation
            )
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "profanity"
            ])
            .unwrap(),
            Tag::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Profanity
            )
        );

        assert_eq!(
            Tag::parse(&["nonce", "1", "20"]).unwrap(),
            Tag::POW {
                nonce: 1,
                difficulty: 20
            }
        );

        assert_eq!(
            Tag::parse(&[
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ])
            .unwrap(),
            Tag::A {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(UncheckedUrl::from_str("wss://relay.nostr.org").unwrap())
            }
        );

        assert_eq!(
            Tag::parse(&["r", "wss://alicerelay.example.com", "read"]).unwrap(),
            Tag::RelayMetadata(
                UncheckedUrl::from("wss://alicerelay.example.com"),
                Some(RelayMetadata::Read)
            )
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "alias",
            ])
            .unwrap(),
            Tag::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: Some(String::from("alias")),
                uppercase: false,
            }
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ])
            .unwrap(),
            Tag::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply)
            }
        );

        assert_eq!(
            Tag::parse(&[
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ]).unwrap(),
            Tag::Delegation { delegator: PublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ).unwrap(), conditions: Conditions::from_str("kind=1").unwrap(), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap() }
        );

        assert_eq!(
            Tag::parse(&[
                "relays",
                "wss://relay.damus.io/",
                "wss://nostr-relay.wlvs.space/",
                "wss://nostr.fmt.wiz.biz",
                "wss//nostr.fmt.wiz.biz"
            ])
            .unwrap(),
            Tag::Relays(vec![
                UncheckedUrl::from("wss://relay.damus.io/"),
                UncheckedUrl::from("wss://nostr-relay.wlvs.space/"),
                UncheckedUrl::from("wss://nostr.fmt.wiz.biz"),
                UncheckedUrl::from("wss//nostr.fmt.wiz.biz")
            ])
        );

        assert_eq!(
            Tag::parse(&[
                "bolt11",
                "lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0"]).unwrap(),
                Tag::Bolt11("lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0".to_string())
        );

        assert_eq!(
            Tag::parse(&[
                "preimage",
                "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f"
            ])
            .unwrap(),
            Tag::Preimage(
                "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f".to_string()
            )
        );

        assert_eq!(
            Tag::parse(&[
                "description",
                "{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}"
            ]).unwrap(),
            Tag::Description("{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}".to_string())
        );

        assert_eq!(
            Tag::parse(&["amount", "10000"]).unwrap(),
            Tag::Amount {
                millisats: 10_000,
                bolt11: None
            }
        );
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn parse_p_tag(bh: &mut Bencher) {
        let tag = &[
            "p",
            "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
        ];
        bh.iter(|| {
            black_box(Tag::parse(tag)).unwrap();
        });
    }

    #[bench]
    pub fn parse_e_tag(bh: &mut Bencher) {
        let tag = &[
            "e",
            "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "wss://relay.damus.io",
        ];
        bh.iter(|| {
            black_box(Tag::parse(tag)).unwrap();
        });
    }

    #[bench]
    pub fn parse_a_tag(bh: &mut Bencher) {
        let tag = &[
            "a",
            "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
            "wss://relay.nostr.org",
        ];
        bh.iter(|| {
            black_box(Tag::parse(tag)).unwrap();
        });
    }
}
