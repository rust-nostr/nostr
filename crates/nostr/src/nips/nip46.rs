// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP46: Nostr Connect
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;
use std::collections::HashMap;

#[cfg(feature = "std")]
use bitcoin::secp256k1::rand;
use bitcoin::secp256k1::rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;

use crate::event::unsigned::{self, UnsignedEvent};
use crate::types::url::{self, ParseError, RelayUrl, Url};
use crate::{key, Event, JsonUtil, PublicKey};

/// NIP46 URI Scheme
pub const NOSTR_CONNECT_URI_SCHEME: &str = "nostrconnect";
/// NIP46 bunker URI Scheme
pub const NOSTR_CONNECT_BUNKER_URI_SCHEME: &str = "bunker";

/// NIP46 error
#[derive(Debug)]
pub enum Error {
    /// Key error
    Key(key::Error),
    /// JSON error
    Json(serde_json::Error),
    /// Relay Url parse error
    RelayUrl(url::Error),
    /// Url parse error
    Url(ParseError),
    /// Unsigned event error
    Unsigned(unsigned::Error),
    /// Invalid request
    InvalidRequest,
    /// Too many/few params
    InvalidParamsLength,
    /// Unsupported method
    UnsupportedMethod(String),
    /// Invalid URI
    InvalidURI,
    /// Invalid URI scheme
    InvalidURIScheme,
    /// Not request
    NotRequest,
    /// Unexpected result
    UnexpectedResult,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "Key: {e}"),
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::RelayUrl(e) => write!(f, "{e}"),
            Self::Url(e) => write!(f, "{e}"),
            Self::Unsigned(e) => write!(f, "Unsigned event: {e}"),
            Self::InvalidRequest => write!(f, "Invalid request"),
            Self::InvalidParamsLength => write!(f, "Too many/few params"),
            Self::UnsupportedMethod(name) => write!(f, "Unsupported method: {name}"),
            Self::InvalidURI => write!(f, "Invalid uri"),
            Self::InvalidURIScheme => write!(f, "Invalid uri scheme"),
            Self::NotRequest => write!(f, "This message is not a request"),
            Self::UnexpectedResult => write!(f, "Unexpected result"),
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

impl From<url::Error> for Error {
    fn from(e: url::Error) -> Self {
        Self::RelayUrl(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<unsigned::Error> for Error {
    fn from(e: unsigned::Error) -> Self {
        Self::Unsigned(e)
    }
}

/// NIP46 method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    /// Connect
    Connect,
    /// Get public key
    GetPublicKey,
    /// Sign event
    SignEvent,
    /// Get relays
    GetRelays,
    /// Encrypt text (NIP04)
    Nip04Encrypt,
    /// Decrypt (NIP04)
    Nip04Decrypt,
    /// Encrypt text (NIP44)
    Nip44Encrypt,
    /// Decrypt (NIP44)
    Nip44Decrypt,
    /// Ping
    Ping,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => write!(f, "connect"),
            Self::GetPublicKey => write!(f, "get_public_key"),
            Self::SignEvent => write!(f, "sign_event"),
            Self::GetRelays => write!(f, "get_relays"),
            Self::Nip04Encrypt => write!(f, "nip04_encrypt"),
            Self::Nip04Decrypt => write!(f, "nip04_decrypt"),
            Self::Nip44Encrypt => write!(f, "nip44_encrypt"),
            Self::Nip44Decrypt => write!(f, "nip44_decrypt"),
            Self::Ping => write!(f, "ping"),
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method {
            "connect" => Ok(Self::Connect),
            "get_public_key" => Ok(Self::GetPublicKey),
            "sign_event" => Ok(Self::SignEvent),
            "get_relays" => Ok(Self::GetRelays),
            "nip04_encrypt" => Ok(Self::Nip04Encrypt),
            "nip04_decrypt" => Ok(Self::Nip04Decrypt),
            "nip44_encrypt" => Ok(Self::Nip44Encrypt),
            "nip44_decrypt" => Ok(Self::Nip44Decrypt),
            "ping" => Ok(Self::Ping),
            other => Err(Error::UnsupportedMethod(other.to_string())),
        }
    }
}

impl Serialize for Method {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let method: String = String::deserialize(deserializer)?;
        Self::from_str(&method).map_err(serde::de::Error::custom)
    }
}

/// Request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Connect
    Connect {
        /// Remote public key
        public_key: PublicKey,
        /// Optional secret
        secret: Option<String>,
    },
    /// Get public key
    GetPublicKey,
    /// Sign [`UnsignedEvent`]
    SignEvent(UnsignedEvent),
    /// Get relays
    GetRelays,
    /// Encrypt text (NIP04)
    Nip04Encrypt {
        /// Pubkey
        public_key: PublicKey,
        /// Plain text
        text: String,
    },
    /// Decrypt (NIP04)
    Nip04Decrypt {
        /// Pubkey
        public_key: PublicKey,
        /// Ciphertext
        ciphertext: String,
    },
    /// Encrypt text (NIP44)
    Nip44Encrypt {
        /// Pubkey
        public_key: PublicKey,
        /// Plain text
        text: String,
    },
    /// Decrypt (NIP44)
    Nip44Decrypt {
        /// Pubkey
        public_key: PublicKey,
        /// Ciphertext
        ciphertext: String,
    },
    /// Ping
    Ping,
}

impl Request {
    /// Compose [Request] from message details
    pub fn from_message(method: Method, params: Vec<String>) -> Result<Self, Error> {
        match method {
            Method::Connect => {
                let public_key = params.first().ok_or(Error::InvalidRequest)?;
                let public_key: PublicKey = PublicKey::from_hex(public_key)?;
                let secret: Option<String> = params.get(1).cloned();
                Ok(Self::Connect { public_key, secret })
            }
            Method::GetPublicKey => Ok(Self::GetPublicKey),
            Method::SignEvent => {
                let unsigned: &String = params.first().ok_or(Error::InvalidRequest)?;
                let unsigned_event: UnsignedEvent = UnsignedEvent::from_json(unsigned)?;
                Ok(Self::SignEvent(unsigned_event))
            }
            Method::GetRelays => Ok(Self::GetRelays),
            Method::Nip04Encrypt => {
                if params.len() != 2 {
                    return Err(Error::InvalidParamsLength);
                }

                Ok(Self::Nip04Encrypt {
                    public_key: PublicKey::from_hex(&params[0])?,
                    text: params[1].to_owned(),
                })
            }
            Method::Nip04Decrypt => {
                if params.len() != 2 {
                    return Err(Error::InvalidParamsLength);
                }

                Ok(Self::Nip04Decrypt {
                    public_key: PublicKey::from_hex(&params[0])?,
                    ciphertext: params[1].to_owned(),
                })
            }
            Method::Nip44Encrypt => {
                if params.len() != 2 {
                    return Err(Error::InvalidParamsLength);
                }

                Ok(Self::Nip44Encrypt {
                    public_key: PublicKey::from_hex(&params[0])?,
                    text: params[1].to_owned(),
                })
            }
            Method::Nip44Decrypt => {
                if params.len() != 2 {
                    return Err(Error::InvalidParamsLength);
                }

                Ok(Self::Nip44Decrypt {
                    public_key: PublicKey::from_hex(&params[0])?,
                    ciphertext: params[1].to_owned(),
                })
            }
            Method::Ping => Ok(Self::Ping),
        }
    }

    /// Get req method
    pub fn method(&self) -> Method {
        match self {
            Self::Connect { .. } => Method::Connect,
            Self::GetPublicKey => Method::GetPublicKey,
            Self::SignEvent(_) => Method::SignEvent,
            Self::GetRelays => Method::GetRelays,
            Self::Nip04Encrypt { .. } => Method::Nip04Encrypt,
            Self::Nip04Decrypt { .. } => Method::Nip04Decrypt,
            Self::Nip44Encrypt { .. } => Method::Nip44Encrypt,
            Self::Nip44Decrypt { .. } => Method::Nip44Decrypt,
            Self::Ping => Method::Ping,
        }
    }

    /// Get req params
    pub fn params(&self) -> Vec<String> {
        match self {
            Self::Connect { public_key, secret } => {
                let mut params = vec![public_key.to_hex()];
                if let Some(secret) = secret {
                    params.push(secret.to_owned());
                }
                params
            }
            Self::GetPublicKey => Vec::new(),
            Self::SignEvent(event) => vec![event.as_json()],
            Self::GetRelays => Vec::new(),
            Self::Nip04Encrypt { public_key, text } | Self::Nip44Encrypt { public_key, text } => {
                vec![public_key.to_hex(), text.to_owned()]
            }
            Self::Nip04Decrypt {
                public_key,
                ciphertext,
            }
            | Self::Nip44Decrypt {
                public_key,
                ciphertext,
            } => vec![public_key.to_hex(), ciphertext.to_owned()],
            Self::Ping => Vec::new(),
        }
    }
}

/// Relay permission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayPermissions {
    /// Read
    pub read: bool,
    /// Write
    pub write: bool,
}

/// Response
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseResult {
    /// Connect ACK
    Connect,
    /// Get public key
    GetPublicKey(PublicKey),
    /// Sign event
    SignEvent(Box<Event>),
    /// Get relays
    GetRelays(HashMap<RelayUrl, RelayPermissions>),
    /// NIP04/NIP44 encryption/decryption
    EncryptionDecryption(String),
    /// Pong
    Pong,
    /// Auth Challenges
    AuthUrl,
    /// Error
    Error,
}

impl fmt::Display for ResponseResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => write!(f, "ack"),
            Self::Pong => write!(f, "pong"),
            Self::AuthUrl => write!(f, "auth_url"),
            Self::Error => write!(f, "error"),
            Self::GetPublicKey(public_key) => write!(f, "{public_key}"),
            Self::SignEvent(event) => write!(f, "{}", event.as_json()),
            Self::GetRelays(map) => write!(f, "{}", json!(map)),
            Self::EncryptionDecryption(val) => write!(f, "{val}"),
        }
    }
}

#[allow(missing_docs)]
impl ResponseResult {
    pub fn parse(res: &str) -> Result<Self, Error> {
        match res {
            "ack" => Ok(Self::Connect),
            "pong" => Ok(Self::Pong),
            "auth_url" => Ok(Self::AuthUrl),
            "error" => Ok(Self::Error),
            other => {
                if let Ok(public_key) = PublicKey::from_hex(other) {
                    Ok(Self::GetPublicKey(public_key))
                } else if let Ok(event) = Event::from_json(other) {
                    Ok(Self::SignEvent(Box::new(event)))
                } else if let Ok(map) = serde_json::from_str(other) {
                    Ok(Self::GetRelays(map))
                } else {
                    Ok(Self::EncryptionDecryption(other.to_string()))
                }
            }
        }
    }
    #[inline]
    pub fn is_auth_url(&self) -> bool {
        matches!(self, Self::AuthUrl)
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    #[inline]
    pub fn to_connect(self) -> Result<(), Error> {
        if let Self::Connect = self {
            Ok(())
        } else {
            Err(Error::UnexpectedResult)
        }
    }

    #[inline]
    pub fn to_get_public_key(self) -> Result<PublicKey, Error> {
        if let Self::GetPublicKey(val) = self {
            Ok(val)
        } else {
            Err(Error::UnexpectedResult)
        }
    }

    #[inline]
    pub fn to_get_relays(self) -> Result<HashMap<RelayUrl, RelayPermissions>, Error> {
        if let Self::GetRelays(val) = self {
            Ok(val)
        } else {
            Err(Error::UnexpectedResult)
        }
    }

    #[inline]
    pub fn to_sign_event(self) -> Result<Event, Error> {
        if let Self::SignEvent(val) = self {
            Ok(*val)
        } else {
            Err(Error::UnexpectedResult)
        }
    }

    #[inline]
    pub fn to_pong(self) -> Result<(), Error> {
        if let Self::Pong = self {
            Ok(())
        } else {
            Err(Error::UnexpectedResult)
        }
    }

    #[inline]
    pub fn to_encrypt_decrypt(self) -> Result<String, Error> {
        if let Self::EncryptionDecryption(val) = self {
            Ok(val)
        } else {
            Err(Error::UnexpectedResult)
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum MessageIntermediate {
    Request {
        id: String,
        method: Method,
        params: Vec<String>,
    },
    Response {
        id: String,
        result: Option<String>,
        error: Option<String>,
    },
}

/// Message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Request
    Request {
        /// Request id
        id: String,
        /// Request
        req: Request,
    },
    /// Response
    Response {
        /// Request id
        id: String,
        /// Result
        result: Option<ResponseResult>,
        /// Reason, if failed
        error: Option<String>,
    },
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_json())
    }
}

impl Message {
    /// Compose [`Request`] message
    #[inline]
    #[cfg(feature = "std")]
    pub fn request(req: Request) -> Self {
        Self::request_with_rng(&mut rand::thread_rng(), req)
    }

    /// Compose [`Request`] message
    #[inline]
    pub fn request_with_rng<R>(rng: &mut R, req: Request) -> Self
    where
        R: RngCore,
    {
        Self::Request {
            id: rng.next_u32().to_string(),
            req,
        }
    }

    /// Compose `Response` message
    #[inline]
    pub fn response<S>(req_id: S, result: Option<ResponseResult>, error: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self::Response {
            id: req_id.into(),
            result,
            error: error.map(|e| e.into()),
        }
    }

    /// Check if current [`Message`] is a request
    #[inline]
    pub fn is_request(&self) -> bool {
        match self {
            Message::Request { .. } => true,
            Message::Response { .. } => false,
        }
    }

    /* pub fn as_request(&self) -> Result<&Request, Error> {
        match self {
            Self::Request { req, .. } => Ok(req),
            _ => Err(Error::NotRequest)
        }
    } */

    /// Consume [Message] and return [Request]
    #[inline]
    pub fn to_request(self) -> Result<Request, Error> {
        match self {
            Self::Request { req, .. } => Ok(req),
            _ => Err(Error::NotRequest),
        }
    }

    /// Get [`Message`] id
    #[inline]
    pub fn id(&self) -> &str {
        match self {
            Self::Request { id, .. } => id,
            Self::Response { id, .. } => id,
        }
    }

    /// Generate response error message for a request
    pub fn generate_error_response<S>(&self, error: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        // Check if Message is a Request
        if self.is_request() {
            let error: &str = error.as_ref();
            Ok(Self::response(self.id(), None, Some(error)))
        } else {
            Err(Error::NotRequest)
        }
    }

    /// Check if result response is `auth_url`
    pub fn is_auth_url(&self) -> bool {
        match self {
            Self::Request { .. } => false,
            Self::Response { result, .. } => match result {
                Some(result) => result.is_auth_url(),
                None => false,
            },
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let intermediate: MessageIntermediate = match self {
            Self::Request { id, req } => MessageIntermediate::Request {
                id: id.to_owned(),
                method: req.method(),
                params: req.params(),
            },
            Self::Response { id, result, error } => MessageIntermediate::Response {
                id: id.to_owned(),
                result: result.as_ref().map(|res| res.to_string()),
                error: error.clone(),
            },
        };
        intermediate.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let intermediate: MessageIntermediate = MessageIntermediate::deserialize(deserializer)?;
        match intermediate {
            MessageIntermediate::Request { id, method, params } => Ok(Self::Request {
                id,
                req: Request::from_message(method, params).map_err(serde::de::Error::custom)?,
            }),
            MessageIntermediate::Response { id, result, error } => {
                let result: Option<ResponseResult> = match result {
                    Some(res) => {
                        // Deserialize response
                        let res: ResponseResult =
                            ResponseResult::parse(&res).map_err(serde::de::Error::custom)?;

                        // Check if is error
                        if res.is_error() {
                            None
                        } else {
                            Some(res)
                        }
                    }
                    None => None,
                };
                Ok(Self::Response { id, result, error })
            }
        }
    }
}

impl JsonUtil for Message {
    type Err = Error;
}

/// Nostr Connect Metadata
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NostrConnectMetadata {
    /// Human-readable name of the `App`
    pub name: String,
    /// URL of the website requesting the connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Description of the `App`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Array of URLs for icons of the `App`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<Url>>,
}

impl NostrConnectMetadata {
    /// New Nostr Connect Metadata
    #[inline]
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            url: None,
            description: None,
            icons: None,
        }
    }

    /// Set url
    pub fn url(self, url: Url) -> Self {
        Self {
            url: Some(url),
            ..self
        }
    }

    /// Set description
    pub fn description<S>(self, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            description: Some(description.into()),
            ..self
        }
    }

    /// Set icons
    pub fn icons(self, icons: Vec<Url>) -> Self {
        Self {
            icons: Some(icons),
            ..self
        }
    }
}

impl JsonUtil for NostrConnectMetadata {
    type Err = Error;
}

/// Nostr Connect URI
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum NostrConnectURI {
    /// Direct connection initiated by remote signer
    Bunker {
        /// Remote signer public key
        remote_signer_public_key: PublicKey,
        /// List of relays to use
        relays: Vec<RelayUrl>,
        /// Optional secret
        secret: Option<String>,
    },
    /// Direct connection initiated by the client
    Client {
        /// App Pubkey
        public_key: PublicKey,
        /// URLs of the relays of choice where the `App` is connected and the `Signer` must send and listen for messages.
        relays: Vec<RelayUrl>,
        /// Metadata
        metadata: NostrConnectMetadata,
    },
}

impl NostrConnectURI {
    /// Construct [NostrConnectURI] initiated by the client
    #[inline]
    pub fn client<I, S>(public_key: PublicKey, relays: I, app_name: S) -> Self
    where
        I: IntoIterator<Item = RelayUrl>,
        S: Into<String>,
    {
        Self::Client {
            public_key,
            relays: relays.into_iter().collect(),
            metadata: NostrConnectMetadata::new(app_name),
        }
    }

    /// Parse Nostr Connect URI
    pub fn parse<S>(uri: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let uri: &str = uri.as_ref();
        let uri: Url = Url::parse(uri)?;

        match uri.scheme() {
            NOSTR_CONNECT_BUNKER_URI_SCHEME => {
                if let Some(pubkey) = uri.domain() {
                    let public_key = PublicKey::from_hex(pubkey)?;

                    let mut relays: Vec<RelayUrl> = Vec::new();
                    let mut secret: Option<String> = None;

                    for (key, value) in uri.query_pairs() {
                        match key {
                            Cow::Borrowed("relay") => {
                                let value = value.to_string();
                                relays.push(RelayUrl::parse(&value)?);
                            }
                            Cow::Borrowed("secret") => {
                                secret = Some(value.to_string());
                            }
                            _ => (),
                        }
                    }

                    return Ok(Self::Bunker {
                        remote_signer_public_key: public_key,
                        relays,
                        secret,
                    });
                }

                Err(Error::InvalidURI)
            }
            NOSTR_CONNECT_URI_SCHEME => {
                if let Some(pubkey) = uri.domain() {
                    let public_key = PublicKey::from_hex(pubkey)?;

                    let mut relays: Vec<RelayUrl> = Vec::new();
                    let mut metadata: Option<NostrConnectMetadata> = None;

                    for (key, value) in uri.query_pairs() {
                        match key {
                            Cow::Borrowed("relay") => {
                                let value = value.to_string();
                                relays.push(RelayUrl::parse(&value)?);
                            }
                            Cow::Borrowed("metadata") => {
                                let value = value.to_string();
                                metadata = Some(serde_json::from_str(&value)?);
                            }
                            _ => (),
                        }
                    }

                    if let Some(metadata) = metadata {
                        return Ok(Self::Client {
                            public_key,
                            relays,
                            metadata,
                        });
                    }
                }

                Err(Error::InvalidURI)
            }
            _ => Err(Error::InvalidURIScheme),
        }
    }

    /// Check if is `bunker` URI
    #[inline]
    pub fn is_bunker(&self) -> bool {
        matches!(self, Self::Bunker { .. })
    }

    /// Get remote signer public key (exists only for `bunker` URIs)
    ///
    /// This public key MAY be same as the user one, but not necessarily.
    #[inline]
    pub fn remote_signer_public_key(&self) -> Option<&PublicKey> {
        match self {
            Self::Bunker {
                remote_signer_public_key,
                ..
            } => Some(remote_signer_public_key),
            Self::Client { .. } => None,
        }
    }

    /// Get relays
    #[inline]
    pub fn relays(&self) -> &[RelayUrl] {
        match self {
            Self::Bunker { relays, .. } => relays.as_slice(),
            Self::Client { relays, .. } => relays.as_slice(),
        }
    }

    /// Get secret
    #[inline]
    pub fn secret(&self) -> Option<&str> {
        match self {
            Self::Bunker { secret, .. } => secret.as_deref(),
            Self::Client { .. } => None,
        }
    }
}

impl FromStr for NostrConnectURI {
    type Err = Error;

    #[inline]
    fn from_str(uri: &str) -> Result<Self, Self::Err> {
        Self::parse(uri)
    }
}

impl fmt::Display for NostrConnectURI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bunker {
                remote_signer_public_key,
                relays,
                secret,
            } => {
                let mut query: String = String::new();

                for relay_url in relays.iter() {
                    let relay_url = relay_url.to_string();
                    let relay_url = relay_url.strip_suffix('/').unwrap_or(&relay_url);

                    if !query.is_empty() {
                        query.push('&');
                    }

                    query.push_str("relay=");
                    query.push_str(relay_url);
                }

                if let Some(secret) = secret {
                    if !query.is_empty() {
                        query.push('&');
                    }

                    query.push_str("secret=");
                    query.push_str(secret);
                }

                if query.is_empty() {
                    write!(
                        f,
                        "{NOSTR_CONNECT_BUNKER_URI_SCHEME}://{remote_signer_public_key}"
                    )
                } else {
                    write!(
                        f,
                        "{NOSTR_CONNECT_BUNKER_URI_SCHEME}://{remote_signer_public_key}?{query}"
                    )
                }
            }
            Self::Client {
                public_key,
                relays,
                metadata,
            } => {
                let mut relays_str: String = String::new();

                for relay_url in relays.iter() {
                    let relay_url = relay_url.to_string();
                    let relay_url = relay_url.strip_suffix('/').unwrap_or(&relay_url);

                    relays_str.push_str("&relay=");
                    relays_str.push_str(relay_url);
                }

                write!(
                    f,
                    "{NOSTR_CONNECT_URI_SCHEME}://{}?metadata={}{relays_str}",
                    public_key,
                    metadata.as_json()
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_bunker_uri() {
        let uri = "bunker://79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3?relay=wss://relay.nsec.app";
        let uri = NostrConnectURI::parse(uri).unwrap();

        let remote_signer_public_key =
            PublicKey::parse("79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3")
                .unwrap();
        let relay_url = RelayUrl::parse("wss://relay.nsec.app").unwrap();
        assert_eq!(uri.relays(), vec![relay_url.clone()]);
        assert_eq!(
            uri,
            NostrConnectURI::Bunker {
                remote_signer_public_key,
                relays: vec![relay_url],
                secret: None
            }
        );
    }

    #[test]
    fn test_parse_client_uri() {
        let uri = r#"nostrconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?metadata={"name":"Example"}&relay=wss://relay.damus.io"#;
        let uri = NostrConnectURI::parse(uri).unwrap();

        let pubkey =
            PublicKey::parse("b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4")
                .unwrap();
        let relay_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let app_name = "Example";
        assert_eq!(uri, NostrConnectURI::client(pubkey, [relay_url], app_name));
    }

    #[test]
    fn test_bunker_uri_serialization() {
        let uri = "bunker://79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3?relay=wss://relay.nsec.app&secret=abcd";

        let remote_signer_public_key =
            PublicKey::parse("79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3")
                .unwrap();
        let relay_url = RelayUrl::parse("wss://relay.nsec.app").unwrap();
        assert_eq!(
            NostrConnectURI::Bunker {
                remote_signer_public_key,
                relays: vec![relay_url],
                secret: Some(String::from("abcd"))
            }
            .to_string(),
            uri
        );
    }

    #[test]
    fn test_client_uri_serialization() {
        let uri = r#"nostrconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?metadata={"name":"Example"}&relay=wss://relay.damus.io"#;

        let pubkey =
            PublicKey::parse("b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4")
                .unwrap();
        let relay_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let app_name = "Example";
        assert_eq!(
            NostrConnectURI::client(pubkey, [relay_url], app_name).to_string(),
            uri
        );
    }

    #[test]
    fn test_parse_response_result() {
        let res: ResponseResult = ResponseResult::parse("ack").unwrap();
        assert_eq!(res, ResponseResult::Connect);

        let pubkey =
            PublicKey::parse("b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4")
                .unwrap();
        let res: ResponseResult = ResponseResult::parse(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )
        .unwrap();
        assert_eq!(res, ResponseResult::GetPublicKey(pubkey));

        let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let event = Event::from_json(json).unwrap();
        let res: ResponseResult = ResponseResult::parse(json).unwrap();
        assert_eq!(res, ResponseResult::SignEvent(Box::new(event)));

        let res: ResponseResult = ResponseResult::parse("pong").unwrap();
        assert_eq!(res, ResponseResult::Pong);
    }

    #[test]
    fn test_message_serialization() {
        // Error
        let message = Message::response(
            "2581081643",
            Some(ResponseResult::Error),
            Some("Empty response"),
        );
        let json = r#"{"id":"2581081643","result":"error","error":"Empty response"}"#;
        assert_eq!(message.as_json(), json);

        // Sign event
        let unsigned = UnsignedEvent::from_json(r#"{"created_at":1710854115,"content":"Testing rust-nostr NIP46 signer [bunker]","tags":[],"kind":1,"pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","id":"236ad3390704e1bf435f40143fb3de163723aeaa8f25c3bf12a0ac4d9a4b56a7"}"#).unwrap();
        let json = r#"{"id":"3047714669","method":"sign_event","params":["{\"id\":\"236ad3390704e1bf435f40143fb3de163723aeaa8f25c3bf12a0ac4d9a4b56a7\",\"pubkey\":\"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3\",\"created_at\":1710854115,\"kind\":1,\"tags\":[],\"content\":\"Testing rust-nostr NIP46 signer [bunker]\"}"]}"#;
        let message = Message::Request {
            id: String::from("3047714669"),
            req: Request::SignEvent(unsigned),
        };
        assert_eq!(message.as_json(), json);
    }

    #[test]
    fn test_message_deserialization() {
        // Connect
        let json = r#"{"id":"2845841889","method":"connect","params":["79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3"]}"#;
        let message = Message::from_json(json).unwrap();
        assert_eq!(
            message,
            Message::Request {
                id: String::from("2845841889"),
                req: Request::Connect {
                    public_key: PublicKey::from_hex(
                        "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3"
                    )
                    .unwrap(),
                    secret: None
                }
            }
        );

        // Connect ACK
        let json = r#"{"id":"2581081643","result":"ack","error":null}"#;
        let message = Message::from_json(json).unwrap();
        assert_eq!(
            message,
            Message::response("2581081643", Some(ResponseResult::Connect), None)
        );

        // Error
        let json = r#"{"id":"2581081643","result":"error","error":"Empty response"}"#;
        let message = Message::from_json(json).unwrap();
        assert_eq!(
            message,
            Message::response("2581081643", None, Some("Empty response"))
        );

        // Sign event
        let event = Event::from_json(r#"{"created_at":1710854115,"content":"Testing rust-nostr NIP46 signer [bunker]","tags":[],"kind":1,"pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","id":"236ad3390704e1bf435f40143fb3de163723aeaa8f25c3bf12a0ac4d9a4b56a7","sig":"509b8fe51c1e4c4cc55a0b2032b70bfb683f1da6c62e4e5b0da7175eab99b18c67862deaaea80cf31acedb9ad3022ebf54fd0cb6c9d1297a96541848d2035d92"}"#).unwrap();
        let json = r#"{"id":"3047714669","result":"{\"created_at\":1710854115,\"content\":\"Testing rust-nostr NIP46 signer [bunker]\",\"tags\":[],\"kind\":1,\"pubkey\":\"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3\",\"id\":\"236ad3390704e1bf435f40143fb3de163723aeaa8f25c3bf12a0ac4d9a4b56a7\",\"sig\":\"509b8fe51c1e4c4cc55a0b2032b70bfb683f1da6c62e4e5b0da7175eab99b18c67862deaaea80cf31acedb9ad3022ebf54fd0cb6c9d1297a96541848d2035d92\"}","error":null}"#;
        let message = Message::from_json(json).unwrap();
        assert_eq!(
            message,
            Message::response(
                "3047714669",
                Some(ResponseResult::SignEvent(Box::new(event))),
                None
            )
        );

        // Encryption
        let ciphertext = "ArY1I2xC2yDwIbuNHN/1ynXdGgzHLqdCrXUPMwELJPc7s7JqlCMJBAIIjfkpHReBPXeoMCyuClwgbT419jUWU1PwaNl4FEQYKCDKVJz+97Mp3K+Q2YGa77B6gpxB/lr1QgoqpDf7wDVrDmOqGoiPjWDqy8KzLueKDcm9BVP8xeTJIxs=";
        let json = r#"{"id":"3047714669","result":"ArY1I2xC2yDwIbuNHN/1ynXdGgzHLqdCrXUPMwELJPc7s7JqlCMJBAIIjfkpHReBPXeoMCyuClwgbT419jUWU1PwaNl4FEQYKCDKVJz+97Mp3K+Q2YGa77B6gpxB/lr1QgoqpDf7wDVrDmOqGoiPjWDqy8KzLueKDcm9BVP8xeTJIxs=","error":null}"#;
        let message = Message::from_json(json).unwrap();
        assert_eq!(
            message,
            Message::response(
                "3047714669",
                Some(ResponseResult::EncryptionDecryption(ciphertext.to_string())),
                None
            )
        );

        // Decryption
        let plaintext = "Hello world!";
        let json = r#"{"id":"3047714669","result":"Hello world!","error":null}"#;
        let message = Message::from_json(json).unwrap();
        assert_eq!(
            message,
            Message::response(
                "3047714669",
                Some(ResponseResult::EncryptionDecryption(plaintext.to_string())),
                None
            )
        );
    }
}
