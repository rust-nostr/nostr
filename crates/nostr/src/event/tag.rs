// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag

use alloc::borrow::{Cow, ToOwned};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::num::ParseIntError;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hex::HexToArrayError;
use bitcoin::secp256k1;
use bitcoin::secp256k1::schnorr::Signature;
#[cfg(feature = "std")]
use once_cell::sync::OnceCell; // TODO: when MSRV will be >= 1.70.0, use `std::cell::OnceLock` instead and remove `once_cell` dep.
#[cfg(not(feature = "std"))]
use once_cell::unsync::OnceCell; // TODO: when MSRV will be >= 1.70.0, use `core::cell::OnceCell` instead and remove `once_cell` dep.
use serde::de::Error as DeserializerError;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::id::{self, EventId};
use crate::nips::nip01::{self, Coordinate};
use crate::nips::nip26::{Conditions, Error as Nip26Error};
use crate::nips::nip48::Protocol;
use crate::nips::nip53::{self, LiveEventMarker, LiveEventStatus};
use crate::nips::nip90::DataVendingMachineStatus;
use crate::types::url::{ParseError, Url};
use crate::util::IntoPublicKey;
use crate::{
    key, Alphabet, Event, JsonUtil, Kind, PublicKey, SingleLetterTag, Timestamp, UncheckedUrl,
};

/// Tag error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Keys
    Keys(key::Error),
    /// Impossible to parse [`Marker`]
    MarkerParseError,
    /// Unknown [`Report`]
    UnknownReportType,
    /// Impossible to find tag kind
    KindNotFound,
    /// Empty tag
    EmptyTag,
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
    /// Unknown standardized tag
    UnknownStardardizedTag,
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
            Self::EmptyTag => write!(f, "Empty tag"),
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
            Self::UnknownStardardizedTag => write!(f, "Unknown standardized tag"),
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
///
/// <https://github.com/nostr-protocol/nips/blob/master/56.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Report {
    /// Depictions of nudity, porn, etc
    Nudity,
    /// Profanity, hateful speech, etc.
    Profanity,
    /// Something which may be illegal in some jurisdiction
    Illegal,
    /// Spam
    Spam,
    /// Someone pretending to be someone else
    Impersonation,
    ///  Reports that don't fit in the above categories
    Other,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nudity => write!(f, "nudity"),
            Self::Profanity => write!(f, "profanity"),
            Self::Illegal => write!(f, "illegal"),
            Self::Spam => write!(f, "spam"),
            Self::Impersonation => write!(f, "impersonation"),
            Self::Other => write!(f, "other"),
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
            "other" => Ok(Self::Other),
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
    #[inline]
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
pub enum TagKind<'a> {
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
    Custom(Cow<'a, str>),
}

impl<'a> fmt::Display for TagKind<'a> {
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

impl<'a> From<&'a str> for TagKind<'a> {
    fn from(kind: &'a str) -> Self {
        match kind {
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
            k => match SingleLetterTag::from_str(k) {
                Ok(s) => Self::SingleLetter(s),
                Err(..) => Self::Custom(Cow::Borrowed(k)),
            },
        }
    }
}

/// Tag
#[derive(Debug, Clone)]
pub struct Tag {
    buf: Vec<String>,
    standardized: Arc<OnceCell<Option<TagStandard>>>,
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.buf == other.buf
    }
}

impl Eq for Tag {}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.buf.cmp(&other.buf)
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buf.hash(state);
    }
}

impl Tag {
    #[inline]
    fn new(buf: Vec<String>, standardized: Option<TagStandard>) -> Self {
        Self {
            buf,
            standardized: Arc::new(OnceCell::from(standardized)),
        }
    }

    #[inline]
    fn new_with_empty_cell(buf: Vec<String>) -> Self {
        Self {
            buf,
            standardized: Arc::new(OnceCell::new()),
        }
    }

    /// Parse tag
    ///
    /// Return error if the tag is empty!
    pub fn parse<S>(tag: &[S]) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        // Check if it's empty
        if tag.is_empty() {
            return Err(Error::EmptyTag);
        }

        // NOT USE `Self::new`!
        Ok(Self::new_with_empty_cell(
            tag.iter().map(|v| v.as_ref().to_string()).collect(),
        ))
    }

    /// Construct from standardized tag
    #[inline]
    pub fn from_standardized(standardized: TagStandard) -> Self {
        Self::new(standardized.clone().to_vec(), Some(standardized))
    }

    /// Construct from standardized tag without initialize cell (avoid a clone)
    #[inline]
    pub fn from_standardized_without_cell(standardized: TagStandard) -> Self {
        Self::new_with_empty_cell(standardized.to_vec())
    }

    /// Get tag kind
    #[inline]
    pub fn kind(&self) -> TagKind {
        // SAFETY: `buf` must not be empty, checked during parsing.
        let key: &str = &self.buf[0];
        TagKind::from(key)
    }

    /// Return the **first** tag value (index `1`), if exists.
    #[inline]
    pub fn content(&self) -> Option<&str> {
        self.buf.get(1).map(|s| s.as_str())
    }

    /// Get [SingleLetterTag]
    #[inline]
    pub fn single_letter_tag(&self) -> Option<SingleLetterTag> {
        match self.kind() {
            TagKind::SingleLetter(s) => Some(s),
            _ => None,
        }
    }

    /// Get reference of standardized tag
    #[inline]
    pub fn as_standardized(&self) -> Option<&TagStandard> {
        self.standardized
            .get_or_init(|| TagStandard::parse(self.as_vec()).ok())
            .as_ref()
    }

    /// Consume tag and get standardized tag
    pub fn to_standardized(self) -> Option<TagStandard> {
        // TODO: replace with `Arc::unwrap_or_clone(self.standardized)` when MSRV will be >= 1.76.0
        let standardized: OnceCell<Option<TagStandard>> =
            Arc::try_unwrap(self.standardized).unwrap_or_else(|arc| (*arc).clone());
        match standardized.into_inner() {
            Some(inner) => inner,
            None => TagStandard::parse(&self.buf).ok(),
        }
    }

    /// Get reference of array of strings
    #[inline]
    pub fn as_vec(&self) -> &[String] {
        &self.buf
    }

    /// Consume tag and return array of strings
    #[inline]
    pub fn to_vec(self) -> Vec<String> {
        self.buf
    }

    /// Compose `["e", "<event-id">]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn event(event_id: EventId) -> Self {
        Self::from_standardized_without_cell(TagStandard::event(event_id))
    }

    /// Compose `["p", "<public-key>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn public_key<T>(public_key: T) -> Self
    where
        T: IntoPublicKey,
    {
        Self::from_standardized_without_cell(TagStandard::public_key(public_key.into_public_key()))
    }

    /// Compose `["d", "<identifier>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn identifier<T>(identifier: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_standardized_without_cell(TagStandard::Identifier(identifier.into()))
    }

    /// Compose `["a", "<coordinate>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn coordinate(coordinate: Coordinate) -> Self {
        Self::from_standardized_without_cell(TagStandard::Coordinate {
            coordinate,
            relay_url: None,
        })
    }

    /// Compose `["nonce", "<nonce>", "<difficulty>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    #[inline]
    pub fn pow(nonce: u128, difficulty: u8) -> Self {
        Self::from_standardized_without_cell(TagStandard::POW { nonce, difficulty })
    }

    /// Compose `["expiration", "<timestamp>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[inline]
    pub fn expiration(timestamp: Timestamp) -> Self {
        Self::from_standardized_without_cell(TagStandard::Expiration(timestamp))
    }

    /// Compose `["e", "<event-id>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    pub fn event_report(event_id: EventId, report: Report) -> Self {
        Self::from_standardized_without_cell(TagStandard::EventReport(event_id, report))
    }

    /// Compose `["p", "<public-key>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    pub fn public_key_report(public_key: PublicKey, report: Report) -> Self {
        Self::from_standardized_without_cell(TagStandard::PublicKeyReport(public_key, report))
    }

    /// Compose `["r", "<relay-url>", "<metadata>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[inline]
    pub fn relay_metadata(relay_url: Url, metadata: Option<RelayMetadata>) -> Self {
        Self::from_standardized_without_cell(TagStandard::RelayMetadata {
            relay_url,
            metadata,
        })
    }

    /// Compose `["t", "<hashtag>"]` tag
    #[inline]
    pub fn hashtag<T>(hashtag: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_standardized_without_cell(TagStandard::Hashtag(hashtag.into()))
    }

    /// Compose custom tag
    ///
    /// JSON: `["<kind>", "<value-1>", "<value-2>", ...]`
    #[inline]
    pub fn custom<I, S>(kind: TagKind, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        // Collect values
        let values: Vec<String> = values.into_iter().map(|v| v.into()).collect();

        // Compose tag
        let mut buf: Vec<String> = Vec::with_capacity(1 + values.len());
        buf.push(kind.to_string());
        buf.extend(values);

        // NOT USE `Self::new`!
        Self::new_with_empty_cell(buf)
    }

    /// Check if tag is an event `reply`
    #[inline]
    pub fn is_reply(&self) -> bool {
        matches!(
            self.as_standardized(),
            Some(TagStandard::Event {
                marker: Some(Marker::Reply),
                ..
            })
        )
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.buf.len()))?;
        for element in self.buf.iter() {
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

/// Standardized tag
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagStandard {
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
    /// Report event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    EventReport(EventId, Report),
    /// Report public key
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    PublicKeyReport(PublicKey, Report),
    PubKeyLiveEvent {
        public_key: PublicKey,
        relay_url: Option<UncheckedUrl>,
        marker: LiveEventMarker,
        proof: Option<Signature>,
    },
    Reference(String),
    /// Relay Metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    RelayMetadata {
        relay_url: Url,
        metadata: Option<RelayMetadata>,
    },
    Hashtag(String),
    Geohash(String),
    Identifier(String),
    ExternalIdentity(Identity),
    Coordinate {
        coordinate: Coordinate,
        relay_url: Option<UncheckedUrl>,
    },
    Kind(Kind),
    Relay(UncheckedUrl),
    /// Proof of Work
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
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
    /// Label namespace
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    LabelNamespace(String),
    /// Label
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    Label(Vec<String>),
}

impl TagStandard {
    /// Parse [`Tag`] from slice of string
    #[inline]
    pub fn parse<S>(tag: &[S]) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let tag_kind: TagKind = match tag.first() {
            Some(kind) => TagKind::from(kind.as_ref()),
            None => return Err(Error::KindNotFound),
        };

        Self::inaternal_parse(&tag_kind, tag)
    }

    fn inaternal_parse<S>(tag_kind: &TagKind, tag: &[S]) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let tag_len: usize = tag.len();

        // Check `relays` tag
        if tag_kind.eq(&TagKind::Relays) {
            // Relays vec is of unknown length so checked here based on kind
            let urls = tag
                .iter()
                .skip(1)
                .map(|u| UncheckedUrl::from(u.as_ref()))
                .collect::<Vec<UncheckedUrl>>();
            return Ok(Self::Relays(urls));
        }

        // Check `l` tag
        if tag_kind.eq(&TagKind::SingleLetter(SingleLetterTag {
            character: Alphabet::L,
            uppercase: false,
        })) {
            let labels = tag.iter().skip(1).map(|u| u.as_ref().to_string()).collect();
            return Ok(Self::Label(labels));
        }

        if tag_len == 1 {
            return match tag_kind {
                TagKind::ContentWarning => Ok(Self::ContentWarning { reason: None }),
                TagKind::Anon => Ok(Self::Anon { msg: None }),
                TagKind::Encrypted => Ok(Self::Encrypted),
                _ => Err(Error::UnknownStardardizedTag),
            };
        }

        if tag_len == 2 {
            let tag_1: &str = tag[1].as_ref();

            return match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::A,
                    uppercase: false,
                }) => Ok(Self::Coordinate {
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
                        uppercase: *uppercase,
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
                        Ok(Self::RelayMetadata {
                            relay_url: Url::parse(tag_1)?,
                            metadata: None,
                        })
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
                }) => Ok(Self::Kind(Kind::from_str(tag_1)?)),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::M,
                    uppercase: false,
                }) => Ok(Self::MimeType(tag_1.to_owned())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::X,
                    uppercase: false,
                }) => Ok(Self::Sha256(Sha256Hash::from_str(tag_1)?)),
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
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::L,
                    uppercase: true,
                }) => Ok(Self::LabelNamespace(tag_1.to_string())),
                TagKind::Dim => Ok(Self::Dim(ImageDimensions::from_str(tag_1)?)),
                _ => Err(Error::UnknownStardardizedTag),
            };
        }

        if tag_len == 3 {
            let tag_1: &str = tag[1].as_ref();
            let tag_2: &str = tag[2].as_ref();

            return match tag_kind {
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
                            Ok(report) => Ok(Self::PublicKeyReport(public_key, report)),
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
                }) => Ok(Self::ExternalIdentity(Identity::new(tag_1, tag_2)?)),
                TagKind::Nonce => Ok(Self::POW {
                    nonce: tag_1.parse()?,
                    difficulty: tag_2.parse()?,
                }),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::A,
                    uppercase: false,
                }) => Ok(Self::Coordinate {
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
                }) => {
                    if (tag_1.starts_with("ws://") || tag_1.starts_with("wss://"))
                        && !tag_2.is_empty()
                    {
                        Ok(Self::RelayMetadata {
                            relay_url: Url::from_str(tag_1)?,
                            metadata: Some(RelayMetadata::from_str(tag_2)?),
                        })
                    } else {
                        Err(Error::UnknownStardardizedTag)
                    }
                }
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
                    Err(_) => Err(Error::UnknownStardardizedTag),
                },
                _ => Err(Error::UnknownStardardizedTag),
            };
        }

        if tag_len == 4 {
            let tag_1: &str = tag[1].as_ref();
            let tag_2: &str = tag[2].as_ref();
            let tag_3: &str = tag[3].as_ref();

            return match tag_kind {
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
                            uppercase: *uppercase,
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
                _ => Err(Error::UnknownStardardizedTag),
            };
        }

        if tag_len == 5 {
            let tag_1: &str = tag[1].as_ref();
            let tag_2: &str = tag[2].as_ref();
            let tag_3: &str = tag[3].as_ref();
            let tag_4: &str = tag[4].as_ref();

            return match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::P,
                    ..
                }) => Ok(Self::PubKeyLiveEvent {
                    public_key: PublicKey::from_str(tag_1)?,
                    relay_url: (!tag_2.is_empty()).then_some(UncheckedUrl::from(tag_2)),
                    marker: LiveEventMarker::from_str(tag_3)?,
                    proof: Signature::from_str(tag_4).ok(),
                }),
                _ => Err(Error::UnknownStardardizedTag),
            };
        }

        Err(Error::UnknownStardardizedTag)
    }

    /// Compose `TagStandard::Event` without `relay_url` and `marker`
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

    /// Compose `TagStandard::PublicKey` without `relay_url` and `alias`
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

    /// Check if tag is an event `reply`
    #[inline]
    pub fn is_reply(&self) -> bool {
        matches!(
            self,
            Self::Event {
                marker: Some(Marker::Reply),
                ..
            }
        )
    }

    /// Get tag kind
    pub fn kind(&self) -> TagKind {
        match self {
            Self::Event { .. } | Self::EventReport(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::E,
                uppercase: false,
            }),
            Self::PublicKey { uppercase, .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::P,
                uppercase: *uppercase,
            }),
            Self::PublicKeyReport(..) | Self::PubKeyLiveEvent { .. } => {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::P,
                    uppercase: false,
                })
            }
            Self::Reference(..) | Self::RelayMetadata { .. } => {
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
            Self::Coordinate { .. } => TagKind::SingleLetter(SingleLetterTag {
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
            Self::LabelNamespace(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::L,
                uppercase: true,
            }),
            Self::Label(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::L,
                uppercase: false,
            }),
        }
    }

    /// Consume [`Tag`] and return string vector
    #[inline]
    pub fn to_vec(self) -> Vec<String> {
        self.into()
    }
}

impl From<TagStandard> for Vec<String> {
    fn from(tag: TagStandard) -> Self {
        let tag_kind: String = tag.kind().to_string();

        match tag {
            TagStandard::Event {
                event_id,
                relay_url,
                marker,
            } => {
                let mut tag = vec![tag_kind, event_id.to_hex()];
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
            TagStandard::PublicKey {
                public_key,
                relay_url,
                alias,
                ..
            } => {
                let mut tag = vec![tag_kind, public_key.to_string()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url.to_string());
                }
                if let Some(alias) = alias {
                    tag.push(alias);
                }
                tag
            }
            TagStandard::EventReport(id, report) => {
                vec![tag_kind, id.to_hex(), report.to_string()]
            }
            TagStandard::PublicKeyReport(pk, report) => {
                vec![tag_kind, pk.to_string(), report.to_string()]
            }
            TagStandard::PubKeyLiveEvent {
                public_key,
                relay_url,
                marker,
                proof,
            } => {
                let mut tag = vec![
                    tag_kind,
                    public_key.to_string(),
                    relay_url.map(|u| u.to_string()).unwrap_or_default(),
                    marker.to_string(),
                ];
                if let Some(proof) = proof {
                    tag.push(proof.to_string());
                }
                tag
            }
            TagStandard::Reference(r) => vec![tag_kind, r],
            TagStandard::RelayMetadata {
                relay_url,
                metadata,
            } => {
                let mut tag = vec![tag_kind, relay_url.to_string()];
                if let Some(metadata) = metadata {
                    tag.push(metadata.to_string());
                }
                tag
            }
            TagStandard::Hashtag(t) => vec![tag_kind, t],
            TagStandard::Geohash(g) => vec![tag_kind, g],
            TagStandard::Identifier(d) => vec![tag_kind, d],
            TagStandard::Coordinate {
                coordinate,
                relay_url,
            } => {
                let mut vec = vec![tag_kind, coordinate.to_string()];
                if let Some(relay) = relay_url {
                    vec.push(relay.to_string());
                }
                vec
            }
            TagStandard::ExternalIdentity(identity) => identity.into(),
            TagStandard::Kind(kind) => vec![tag_kind, kind.to_string()],
            TagStandard::Relay(url) => vec![tag_kind, url.to_string()],
            TagStandard::POW { nonce, difficulty } => {
                vec![tag_kind, nonce.to_string(), difficulty.to_string()]
            }
            TagStandard::Delegation {
                delegator,
                conditions,
                sig,
            } => vec![
                tag_kind,
                delegator.to_string(),
                conditions.to_string(),
                sig.to_string(),
            ],
            TagStandard::ContentWarning { reason } => {
                let mut tag = vec![tag_kind];
                if let Some(reason) = reason {
                    tag.push(reason);
                }
                tag
            }
            TagStandard::Expiration(timestamp) => {
                vec![tag_kind, timestamp.to_string()]
            }
            TagStandard::Subject(sub) => vec![tag_kind, sub],
            TagStandard::Challenge(challenge) => vec![tag_kind, challenge],
            TagStandard::Title(title) => vec![tag_kind, title],
            TagStandard::Image(image, dimensions) => {
                let mut tag = vec![tag_kind, image.to_string()];
                if let Some(dim) = dimensions {
                    tag.push(dim.to_string());
                }
                tag
            }
            TagStandard::Thumb(thumb, dimensions) => {
                let mut tag = vec![tag_kind, thumb.to_string()];
                if let Some(dim) = dimensions {
                    tag.push(dim.to_string());
                }
                tag
            }
            TagStandard::Summary(summary) => vec![tag_kind, summary],
            TagStandard::PublishedAt(timestamp) => {
                vec![tag_kind, timestamp.to_string()]
            }
            TagStandard::Description(description) => {
                vec![tag_kind, description]
            }
            TagStandard::Bolt11(bolt11) => {
                vec![tag_kind, bolt11]
            }
            TagStandard::Preimage(preimage) => {
                vec![tag_kind, preimage]
            }
            TagStandard::Relays(relays) => vec![tag_kind]
                .into_iter()
                .chain(relays.iter().map(|relay| relay.to_string()))
                .collect::<Vec<_>>(),
            TagStandard::Amount { millisats, bolt11 } => {
                let mut tag = vec![tag_kind, millisats.to_string()];
                if let Some(bolt11) = bolt11 {
                    tag.push(bolt11);
                }
                tag
            }
            TagStandard::Name(name) => {
                vec![tag_kind, name]
            }
            TagStandard::Lnurl(lnurl) => {
                vec![tag_kind, lnurl]
            }
            TagStandard::Url(url) => vec![tag_kind, url.to_string()],
            TagStandard::MimeType(mime) => vec![tag_kind, mime],
            TagStandard::Aes256Gcm { key, iv } => vec![tag_kind, key, iv],
            TagStandard::Sha256(hash) => vec![tag_kind, hash.to_string()],
            TagStandard::Size(bytes) => vec![tag_kind, bytes.to_string()],
            TagStandard::Dim(dim) => vec![tag_kind, dim.to_string()],
            TagStandard::Magnet(uri) => vec![tag_kind, uri],
            TagStandard::Blurhash(data) => vec![tag_kind, data],
            TagStandard::Streaming(url) => vec![tag_kind, url.to_string()],
            TagStandard::Recording(url) => vec![tag_kind, url.to_string()],
            TagStandard::Starts(timestamp) => {
                vec![tag_kind, timestamp.to_string()]
            }
            TagStandard::Ends(timestamp) => {
                vec![tag_kind, timestamp.to_string()]
            }
            TagStandard::LiveEventStatus(s) => {
                vec![tag_kind, s.to_string()]
            }
            TagStandard::CurrentParticipants(num) => {
                vec![tag_kind, num.to_string()]
            }
            TagStandard::TotalParticipants(num) => {
                vec![tag_kind, num.to_string()]
            }
            TagStandard::AbsoluteURL(url) => {
                vec![tag_kind, url.to_string()]
            }
            TagStandard::Method(method) => {
                vec![tag_kind, method.to_string()]
            }
            TagStandard::Payload(p) => vec![tag_kind, p.to_string()],
            TagStandard::Anon { msg } => {
                let mut tag = vec![tag_kind];
                if let Some(msg) = msg {
                    tag.push(msg);
                }
                tag
            }
            TagStandard::Proxy { id, protocol } => {
                vec![tag_kind, id, protocol.to_string()]
            }
            TagStandard::Emoji { shortcode, url } => {
                vec![tag_kind, shortcode, url.to_string()]
            }
            TagStandard::Encrypted => vec![tag_kind],
            TagStandard::Request(event) => vec![tag_kind, event.as_json()],
            TagStandard::DataVendingMachineStatus { status, extra_info } => {
                let mut tag = vec![tag_kind, status.to_string()];
                if let Some(extra_info) = extra_info {
                    tag.push(extra_info);
                }
                tag
            }
            TagStandard::Word(word) => vec![tag_kind, word],
            TagStandard::LabelNamespace(n) => vec![tag_kind, n],
            TagStandard::Label(l) => {
                let mut tag = Vec::with_capacity(1 + l.len());
                tag.push(tag_kind);
                tag.extend(l);
                tag
            }
        }
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

impl TryFrom<TagStandard> for Identity {
    type Error = Error;

    fn try_from(value: TagStandard) -> Result<Self, Self::Error> {
        match value {
            TagStandard::ExternalIdentity(iden) => Ok(iden),
            _ => Err(Error::InvalidIdentity),
        }
    }
}

impl From<Identity> for TagStandard {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, JsonUtil, Timestamp};

    #[test]
    fn test_tag_match_standardized() {
        let tag: Tag = Tag::parse(&["d", "bravery"]).unwrap();
        assert_eq!(
            tag.as_standardized(),
            Some(&TagStandard::Identifier(String::from("bravery")))
        );

        let tag: Tag = Tag::parse(&["d", "test"]).unwrap();
        assert_eq!(
            tag.to_standardized(),
            Some(TagStandard::Identifier(String::from("test")))
        );
    }

    #[test]
    fn test_tag_standard_is_reply() {
        let tag = TagStandard::Relay(UncheckedUrl::new("wss://relay.damus.io"));
        assert!(!tag.is_reply());

        let tag = TagStandard::Event {
            event_id: EventId::from_hex(
                "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
            )
            .unwrap(),
            relay_url: None,
            marker: Some(Marker::Reply),
        };
        assert!(tag.is_reply());

        let tag = TagStandard::Event {
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
        assert_eq!(t.content(), Some("bbbbbb"));

        // Test extract public key
        let t: Tag = Tag::parse(&[
            "custom-p",
            "f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785",
        ])
        .unwrap();
        assert_eq!(
            t.content(),
            Some("f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785")
        );

        // Test extract event ID
        let t: Tag = Tag::parse(&[
            "custom-e",
            "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
        ])
        .unwrap();
        assert_eq!(
            t.content(),
            Some("2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45")
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
            Tag::from_standardized_without_cell(TagStandard::ContentWarning { reason: None })
                .to_vec()
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
            .to_vec()
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
            .to_vec()
        );

        assert_eq!(
            vec!["expiration", "1600000000"],
            Tag::expiration(Timestamp::from(1600000000)).to_vec()
        );

        assert_eq!(
            vec!["content-warning", "reason"],
            Tag::from_standardized_without_cell(TagStandard::ContentWarning {
                reason: Some(String::from("reason"))
            })
            .to_vec()
        );

        assert_eq!(
            vec!["subject", "textnote with subject"],
            Tag::from_standardized_without_cell(TagStandard::Subject(String::from(
                "textnote with subject"
            )))
            .to_vec()
        );

        assert_eq!(
            vec!["client", "rust-nostr"],
            Tag::custom(TagKind::Custom(Cow::Borrowed("client")), ["rust-nostr"]).to_vec()
        );

        assert_eq!(vec!["d", "test"], Tag::identifier("test").to_vec());

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ],
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: None,
                uppercase: false,
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ],
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::empty()),
                marker: None
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ],
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: None
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "spam"
            ],
            Tag::public_key_report(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Spam
            )
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "nudity"
            ],
            Tag::event_report(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Nudity,
            )
            .to_vec()
        );

        assert_eq!(vec!["nonce", "1", "20"], Tag::pow(1, 20).to_vec());

        assert_eq!(
            vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum"
            ],
            Tag::coordinate(
                Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
            )
            .to_vec()
        );

        assert_eq!(
            vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ],
            Tag::from_standardized_without_cell(TagStandard::Coordinate {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(UncheckedUrl::from_str("wss://relay.nostr.org").unwrap())
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "Speaker",
            ],
            Tag::from_standardized_without_cell(TagStandard::PubKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: LiveEventMarker::Speaker,
                proof: None
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "",
                "Participant",
            ],
            Tag::from_standardized_without_cell(TagStandard::PubKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: None,
                marker: LiveEventMarker::Participant,
                proof: None
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "alias",
            ],
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: Some(String::from("alias")),
                uppercase: false,
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ],
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply)
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ],
            Tag::from_standardized_without_cell(TagStandard::Delegation { delegator: PublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ).unwrap(), conditions: Conditions::from_str("kind=1").unwrap(), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap() })
            .to_vec()
        );

        assert_eq!(
            vec!["lnurl", "lnurl1dp68gurn8ghj7um5v93kketj9ehx2amn9uh8wetvdskkkmn0wahz7mrww4excup0dajx2mrv92x9xp"],
            Tag::from_standardized_without_cell(TagStandard::Lnurl(String::from("lnurl1dp68gurn8ghj7um5v93kketj9ehx2amn9uh8wetvdskkkmn0wahz7mrww4excup0dajx2mrv92x9xp"))).to_vec(),
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "Host",
                "a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"
            ],
            Tag::from_standardized_without_cell(TagStandard::PubKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                ).unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: LiveEventMarker::Host,
                proof: Some(Signature::from_str("a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd").unwrap())
            })
            .to_vec()
        );

        assert_eq!(
            vec!["L", "#t"],
            Tag::from_standardized_without_cell(TagStandard::LabelNamespace("#t".to_string()))
                .to_vec()
        );

        assert_eq!(
            vec!["l", "IT-MI"],
            Tag::from_standardized_without_cell(TagStandard::Label(vec!["IT-MI".to_string()]))
                .to_vec()
        );

        assert_eq!(
            vec!["l", "IT-MI", "ISO-3166-2"],
            Tag::from_standardized_without_cell(TagStandard::Label(vec![
                "IT-MI".to_string(),
                "ISO-3166-2".to_string()
            ]))
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land/"],
            Tag::relay_metadata(Url::from_str("wss://atlas.nostr.land").unwrap(), None).to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land/", "read"],
            Tag::relay_metadata(
                Url::from_str("wss://atlas.nostr.land").unwrap(),
                Some(RelayMetadata::Read)
            )
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land/", "write"],
            Tag::relay_metadata(
                Url::from_str("wss://atlas.nostr.land").unwrap(),
                Some(RelayMetadata::Write)
            )
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land", ""],
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                ["wss://atlas.nostr.land", ""]
            )
            .to_vec()
        );

        assert_eq!(
            vec![
                "r",
                "3dbee968d1ddcdf07521e246e405e1fbb549080f1f4ef4e42526c4528f124220",
                ""
            ],
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                [
                    "3dbee968d1ddcdf07521e246e405e1fbb549080f1f4ef4e42526c4528f124220",
                    ""
                ]
            )
            .to_vec()
        );
    }

    #[test]
    fn test_tag_parser() {
        assert_eq!(Tag::parse::<String>(&[]).unwrap_err(), Error::EmptyTag);

        assert_eq!(
            Tag::parse(&["content-warning"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::ContentWarning { reason: None })
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
            Tag::expiration(Timestamp::from(1600000000))
        );

        assert_eq!(
            Tag::parse(&["content-warning", "reason"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::ContentWarning {
                reason: Some(String::from("reason"))
            })
        );

        assert_eq!(
            Tag::parse(&["subject", "textnote with subject"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Subject(String::from(
                "textnote with subject"
            )))
        );

        assert_eq!(
            Tag::parse(&["client", "nostr-sdk"]).unwrap(),
            Tag::custom(TagKind::Custom(Cow::Borrowed("client")), ["nostr-sdk"])
        );

        assert_eq!(Tag::parse(&["d", "test"]).unwrap(), Tag::identifier("test"));

        assert_eq!(
            Tag::parse(&["r", "https://example.com"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Reference(String::from(
                "https://example.com"
            )))
        );

        assert_eq!(
            Tag::parse(&["r", "wss://alicerelay.example.com/"]).unwrap(),
            Tag::relay_metadata(Url::from_str("wss://alicerelay.example.com").unwrap(), None)
        );

        assert_eq!(
            Tag::parse(&["i", "github:12345678", "abcdefghijklmnop"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::ExternalIdentity(Identity {
                platform: ExternalIdentity::GitHub,
                ident: "12345678".to_string(),
                proof: "abcdefghijklmnop".to_string()
            }))
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: None,
                uppercase: false
            })
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::empty()),
                marker: None
            })
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: None
            })
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "impersonation"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::PublicKeyReport(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Impersonation
            ))
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "other"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::PublicKeyReport(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Other
            ))
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "profanity"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Profanity
            ))
        );

        assert_eq!(Tag::parse(&["nonce", "1", "20"]).unwrap(), Tag::pow(1, 20));

        assert_eq!(
            Tag::parse(&[
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Coordinate {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(UncheckedUrl::from_str("wss://relay.nostr.org").unwrap())
            })
        );

        assert_eq!(
            Tag::parse(&["r", "wss://alicerelay.example.com/", "read"]).unwrap(),
            Tag::relay_metadata(
                Url::from_str("wss://alicerelay.example.com").unwrap(),
                Some(RelayMetadata::Read)
            )
        );

        assert_eq!(
            Tag::parse(&["r", "wss://atlas.nostr.land/"]).unwrap(),
            Tag::relay_metadata(Url::from_str("wss://atlas.nostr.land").unwrap(), None)
        );

        assert_eq!(
            Tag::parse(&["r", "wss://atlas.nostr.land/", "read"]).unwrap(),
            Tag::relay_metadata(
                Url::from_str("wss://atlas.nostr.land").unwrap(),
                Some(RelayMetadata::Read)
            )
        );

        assert_eq!(
            Tag::parse(&["r", "wss://atlas.nostr.land/", "write"]).unwrap(),
            Tag::relay_metadata(
                Url::from_str("wss://atlas.nostr.land").unwrap(),
                Some(RelayMetadata::Write)
            )
        );

        assert_eq!(
            Tag::parse(&["r", "wss://atlas.nostr.land", ""]).unwrap(),
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                ["wss://atlas.nostr.land", ""]
            )
        );

        assert_eq!(
            Tag::parse(&[
                "r",
                "3dbee968d1ddcdf07521e246e405e1fbb549080f1f4ef4e42526c4528f124220",
                ""
            ])
            .unwrap(),
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                [
                    "3dbee968d1ddcdf07521e246e405e1fbb549080f1f4ef4e42526c4528f124220",
                    ""
                ]
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
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: Some(String::from("alias")),
                uppercase: false,
            })
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply)
            })
        );

        assert_eq!(
            Tag::parse(&[
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Delegation { delegator: PublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ).unwrap(), conditions: Conditions::from_str("kind=1").unwrap(), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap() })
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
            Tag::from_standardized_without_cell(TagStandard::Relays(vec![
                UncheckedUrl::from("wss://relay.damus.io/"),
                UncheckedUrl::from("wss://nostr-relay.wlvs.space/"),
                UncheckedUrl::from("wss://nostr.fmt.wiz.biz"),
                UncheckedUrl::from("wss//nostr.fmt.wiz.biz")
            ]))
        );

        assert_eq!(
            Tag::parse(&[
                "bolt11",
                "lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Bolt11("lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0".to_string()))
        );

        assert_eq!(
            Tag::parse(&[
                "preimage",
                "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Preimage(
                "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f".to_string()
            ))
        );

        assert_eq!(
            Tag::parse(&[
                "description",
                "{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}"
            ]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Description("{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}".to_string()))
        );

        assert_eq!(
            Tag::parse(&["amount", "10000"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Amount {
                millisats: 10_000,
                bolt11: None
            })
        );

        assert_eq!(
            Tag::parse(&["L", "#t"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::LabelNamespace("#t".to_string()))
        );

        assert_eq!(
            Tag::parse(&["l", "IT-MI"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Label(vec!["IT-MI".to_string()]))
        );

        assert_eq!(
            Tag::parse(&["l", "IT-MI", "ISO-3166-2"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Label(vec![
                "IT-MI".to_string(),
                "ISO-3166-2".to_string()
            ]))
        );
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn get_tag_kind(bh: &mut Bencher) {
        let tag = Tag::identifier("id");
        bh.iter(|| {
            black_box(tag.kind());
        });
    }

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
    pub fn parse_p_standardized_tag(bh: &mut Bencher) {
        let tag = &[
            "p",
            "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
        ];
        bh.iter(|| {
            black_box(TagStandard::parse(tag)).unwrap();
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
    pub fn parse_e_standardized_tag(bh: &mut Bencher) {
        let tag = &[
            "e",
            "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "wss://relay.damus.io",
        ];
        bh.iter(|| {
            black_box(TagStandard::parse(tag)).unwrap();
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

    #[bench]
    pub fn parse_a_standardized_tag(bh: &mut Bencher) {
        let tag = &[
            "a",
            "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
            "wss://relay.nostr.org",
        ];
        bh.iter(|| {
            black_box(TagStandard::parse(tag)).unwrap();
        });
    }
}
