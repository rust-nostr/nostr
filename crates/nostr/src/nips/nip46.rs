// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
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
use secp256k1::rand;
use secp256k1::rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;

use crate::event::unsigned::UnsignedEvent;
use crate::types::url::{self, ParseError, RelayUrl, Url};
use crate::{event, key, Event, JsonUtil, PublicKey};

/// NIP46 URI Scheme
pub const NOSTR_CONNECT_URI_SCHEME: &str = "nostrconnect";
/// NIP46 bunker URI Scheme
pub const NOSTR_CONNECT_BUNKER_URI_SCHEME: &str = "bunker";

/// NIP46 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Key error
    Key(key::Error),
    /// JSON error
    Json(String),
    /// Relay Url parse error
    RelayUrl(url::Error),
    /// Url parse error
    Url(ParseError),
    /// Event error
    Event(event::Error),
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
    /// Not a request
    NotRequest,
    /// Not a response
    NotResponse,
    /// Unexpected result
    UnexpectedResult,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::RelayUrl(e) => write!(f, "{e}"),
            Self::Url(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::InvalidRequest => write!(f, "Invalid request"),
            Self::InvalidParamsLength => write!(f, "Invalid params len"),
            Self::UnsupportedMethod(name) => write!(f, "Unsupported method: {name}"),
            Self::InvalidURI => write!(f, "Invalid uri"),
            Self::InvalidURIScheme => write!(f, "Invalid uri scheme"),
            Self::NotRequest => write!(f, "Not a request"),
            Self::NotResponse => write!(f, "Not a response"),
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
        Self::Json(e.to_string())
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

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

/// NIP46 method
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NostrConnectMethod {
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

impl fmt::Display for NostrConnectMethod {
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

impl FromStr for NostrConnectMethod {
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

impl Serialize for NostrConnectMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for NostrConnectMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let method: String = String::deserialize(deserializer)?;
        Self::from_str(&method).map_err(serde::de::Error::custom)
    }
}

/// Nostr Connect Request
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NostrConnectRequest {
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

impl NostrConnectRequest {
    /// Compose [`NostrConnectRequest`] from message details
    pub fn from_message(method: NostrConnectMethod, params: Vec<String>) -> Result<Self, Error> {
        match method {
            NostrConnectMethod::Connect => {
                let public_key = params.first().ok_or(Error::InvalidRequest)?;
                let public_key: PublicKey = PublicKey::from_hex(public_key)?;
                let secret: Option<String> = params.get(1).cloned();
                Ok(Self::Connect { public_key, secret })
            }
            NostrConnectMethod::GetPublicKey => Ok(Self::GetPublicKey),
            NostrConnectMethod::SignEvent => {
                let unsigned: &String = params.first().ok_or(Error::InvalidRequest)?;
                let unsigned_event: UnsignedEvent = UnsignedEvent::from_json(unsigned)?;
                Ok(Self::SignEvent(unsigned_event))
            }
            NostrConnectMethod::GetRelays => Ok(Self::GetRelays),
            NostrConnectMethod::Nip04Encrypt => {
                if params.len() != 2 {
                    return Err(Error::InvalidParamsLength);
                }

                Ok(Self::Nip04Encrypt {
                    public_key: PublicKey::from_hex(&params[0])?,
                    text: params[1].to_owned(),
                })
            }
            NostrConnectMethod::Nip04Decrypt => {
                if params.len() != 2 {
                    return Err(Error::InvalidParamsLength);
                }

                Ok(Self::Nip04Decrypt {
                    public_key: PublicKey::from_hex(&params[0])?,
                    ciphertext: params[1].to_owned(),
                })
            }
            NostrConnectMethod::Nip44Encrypt => {
                if params.len() != 2 {
                    return Err(Error::InvalidParamsLength);
                }

                Ok(Self::Nip44Encrypt {
                    public_key: PublicKey::from_hex(&params[0])?,
                    text: params[1].to_owned(),
                })
            }
            NostrConnectMethod::Nip44Decrypt => {
                if params.len() != 2 {
                    return Err(Error::InvalidParamsLength);
                }

                Ok(Self::Nip44Decrypt {
                    public_key: PublicKey::from_hex(&params[0])?,
                    ciphertext: params[1].to_owned(),
                })
            }
            NostrConnectMethod::Ping => Ok(Self::Ping),
        }
    }

    /// Get req method
    pub fn method(&self) -> NostrConnectMethod {
        match self {
            Self::Connect { .. } => NostrConnectMethod::Connect,
            Self::GetPublicKey => NostrConnectMethod::GetPublicKey,
            Self::SignEvent(_) => NostrConnectMethod::SignEvent,
            Self::GetRelays => NostrConnectMethod::GetRelays,
            Self::Nip04Encrypt { .. } => NostrConnectMethod::Nip04Encrypt,
            Self::Nip04Decrypt { .. } => NostrConnectMethod::Nip04Decrypt,
            Self::Nip44Encrypt { .. } => NostrConnectMethod::Nip44Encrypt,
            Self::Nip44Decrypt { .. } => NostrConnectMethod::Nip44Decrypt,
            Self::Ping => NostrConnectMethod::Ping,
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
    Ack,
    /// Get public key
    GetPublicKey(PublicKey),
    /// Sign event
    SignEvent(Box<Event>),
    /// Get relays
    GetRelays(HashMap<RelayUrl, RelayPermissions>),
    /// Encrypt text (NIP04)
    Nip04Encrypt {
        /// Cipher text
        ciphertext: String,
    },
    /// Decrypt (NIP04)
    Nip04Decrypt {
        /// Plain text
        plaintext: String,
    },
    /// Encrypt text (NIP44)
    Nip44Encrypt {
        /// Cipher text
        ciphertext: String,
    },
    /// Decrypt (NIP44)
    Nip44Decrypt {
        /// Plain text
        plaintext: String,
    },
    /// Pong
    Pong,
    /// Auth Challenges
    AuthUrl,
    /// Error
    Error,
}

/// Nostr Connect Response
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NostrConnectResponse {
    /// Response result
    pub result: Option<ResponseResult>,
    /// Response error
    pub error: Option<String>,
}

impl NostrConnectResponse {
    /// New response
    #[inline]
    pub fn new(result: Option<ResponseResult>, error: Option<String>) -> Self {
        Self { result, error }
    }

    /// New response with result
    #[inline]
    pub fn with_result(result: ResponseResult) -> Self {
        Self {
            result: Some(result),
            error: None,
        }
    }

    /// New response with error
    #[inline]
    pub fn with_error<S>(error: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            result: None,
            error: Some(error.into()),
        }
    }

    /// Parse response
    pub fn parse(
        method: NostrConnectMethod,
        result: Option<String>,
        error: Option<String>,
    ) -> Result<Self, Error> {
        Ok(Self {
            result: match result {
                Some(result) => Some(ResponseResult::parse(method, result)?),
                None => None,
            },
            error,
        })
    }

    /// Check if the response is an auth URL
    #[inline]
    pub fn is_auth_url(&self) -> bool {
        match &self.result {
            Some(res) => res.is_auth_url(),
            None => false,
        }
    }
}

impl fmt::Display for ResponseResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ack => write!(f, "ack"),
            Self::GetPublicKey(public_key) => write!(f, "{public_key}"),
            Self::SignEvent(event) => write!(f, "{}", event.as_json()),
            Self::GetRelays(map) => write!(f, "{}", json!(map)),
            Self::Nip04Encrypt { ciphertext } | Self::Nip44Encrypt { ciphertext } => {
                write!(f, "{ciphertext}")
            }
            Self::Nip04Decrypt { plaintext } | Self::Nip44Decrypt { plaintext } => {
                write!(f, "{plaintext}")
            }
            Self::Pong => write!(f, "pong"),
            Self::AuthUrl => write!(f, "auth_url"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[allow(missing_docs)]
impl ResponseResult {
    pub fn parse<S>(method: NostrConnectMethod, response: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let response: String = response.into();

        // Check if the response is an Auth URL or an error
        match response.as_str() {
            "auth_url" => return Ok(Self::AuthUrl),
            "error" => return Ok(Self::Error),
            _ => {}
        };

        // Parse response depending on the request method
        match method {
            NostrConnectMethod::Connect => {
                if response == "ack" {
                    Ok(Self::Ack)
                } else {
                    Err(Error::UnexpectedResult)
                }
            }
            NostrConnectMethod::GetPublicKey => {
                Ok(Self::GetPublicKey(PublicKey::from_hex(&response)?))
            }
            NostrConnectMethod::SignEvent => {
                Ok(Self::SignEvent(Box::new(Event::from_json(response)?)))
            }
            NostrConnectMethod::GetRelays => Ok(Self::GetRelays(serde_json::from_str(&response)?)),
            NostrConnectMethod::Nip04Encrypt => Ok(Self::Nip04Encrypt {
                ciphertext: response,
            }),
            NostrConnectMethod::Nip04Decrypt => Ok(Self::Nip04Decrypt {
                plaintext: response,
            }),
            NostrConnectMethod::Nip44Encrypt => Ok(Self::Nip44Encrypt {
                ciphertext: response,
            }),
            NostrConnectMethod::Nip44Decrypt => Ok(Self::Nip44Decrypt {
                plaintext: response,
            }),
            NostrConnectMethod::Ping => {
                if response == "pong" {
                    Ok(Self::Pong)
                } else {
                    Err(Error::UnexpectedResult)
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
    pub fn to_ack(self) -> Result<(), Error> {
        if let Self::Ack = self {
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
    pub fn to_nip04_encrypt(self) -> Result<String, Error> {
        if let Self::Nip04Encrypt { ciphertext } = self {
            Ok(ciphertext)
        } else {
            Err(Error::UnexpectedResult)
        }
    }

    #[inline]
    pub fn to_nip04_decrypt(self) -> Result<String, Error> {
        if let Self::Nip04Decrypt { plaintext } = self {
            Ok(plaintext)
        } else {
            Err(Error::UnexpectedResult)
        }
    }

    #[inline]
    pub fn to_nip44_encrypt(self) -> Result<String, Error> {
        if let Self::Nip44Encrypt { ciphertext } = self {
            Ok(ciphertext)
        } else {
            Err(Error::UnexpectedResult)
        }
    }

    #[inline]
    pub fn to_nip44_decrypt(self) -> Result<String, Error> {
        if let Self::Nip44Decrypt { plaintext } = self {
            Ok(plaintext)
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
}

/// Nostr Connect Message
///
/// <https://github.com/nostr-protocol/nips/blob/master/46.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NostrConnectMessage {
    /// Request
    Request {
        /// Request ID
        id: String,
        /// Request method
        method: NostrConnectMethod,
        /// Request params
        params: Vec<String>,
    },
    /// Response
    Response {
        /// Request id
        id: String,
        /// Result
        result: Option<String>,
        /// Reason, if failed
        error: Option<String>,
    },
}

impl fmt::Display for NostrConnectMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_json())
    }
}

impl NostrConnectMessage {
    /// Compose [`NostrConnectMessage::Request`] from [`NostrConnectRequest`].
    #[inline]
    #[cfg(feature = "std")]
    pub fn request(req: &NostrConnectRequest) -> Self {
        Self::request_with_rng(&mut rand::thread_rng(), req)
    }

    /// Compose [`NostrConnectMessage::Request`] from [`NostrConnectRequest`].
    #[inline]
    pub fn request_with_rng<R>(rng: &mut R, req: &NostrConnectRequest) -> Self
    where
        R: RngCore,
    {
        Self::Request {
            id: rng.next_u32().to_string(),
            method: req.method(),
            params: req.params(),
        }
    }

    /// Compose [`NostrConnectMessage::Response`] from [`Response`].
    #[inline]
    pub fn response<S>(req_id: S, res: NostrConnectResponse) -> Self
    where
        S: Into<String>,
    {
        Self::Response {
            id: req_id.into(),
            result: res.result.map(|res| res.to_string()),
            error: res.error,
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

    /// Check if the current [`Message`] is a request.
    #[inline]
    pub fn is_request(&self) -> bool {
        matches!(self, Self::Request { .. })
    }

    /// Check if the current [`Message`] is a response.
    #[inline]
    pub fn is_response(&self) -> bool {
        matches!(self, Self::Response { .. })
    }

    /// Convert [`NostrConnectMessage::Request`] to [`Request`].
    #[inline]
    pub fn to_request(self) -> Result<NostrConnectRequest, Error> {
        match self {
            Self::Request { method, params, .. } => {
                NostrConnectRequest::from_message(method, params)
            }
            _ => Err(Error::NotRequest),
        }
    }

    /// Convert [`NostrConnectMessage::Response`] to [`Response`].
    #[inline]
    pub fn to_response(self, method: NostrConnectMethod) -> Result<NostrConnectResponse, Error> {
        match self {
            Self::Response { result, error, .. } => {
                NostrConnectResponse::parse(method, result, error)
            }
            _ => Err(Error::NotRequest),
        }
    }
}

impl JsonUtil for NostrConnectMessage {
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
                    let relay_url: &str = relay_url.as_str_without_trailing_slash();

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
        let public_key =
            PublicKey::parse("b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4")
                .unwrap();

        let res: ResponseResult =
            ResponseResult::parse(NostrConnectMethod::Connect, "ack").unwrap();
        assert_eq!(res, ResponseResult::Ack);

        let res = ResponseResult::parse(NostrConnectMethod::Ping, "ack");
        assert_eq!(res.unwrap_err(), Error::UnexpectedResult);

        let res: ResponseResult = ResponseResult::parse(
            NostrConnectMethod::GetPublicKey,
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )
        .unwrap();
        assert_eq!(res, ResponseResult::GetPublicKey(public_key));

        let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let event = Event::from_json(json).unwrap();
        let res: ResponseResult =
            ResponseResult::parse(NostrConnectMethod::SignEvent, json).unwrap();
        assert_eq!(res, ResponseResult::SignEvent(Box::new(event)));

        let res: ResponseResult = ResponseResult::parse(NostrConnectMethod::Ping, "pong").unwrap();
        assert_eq!(res, ResponseResult::Pong);
    }

    #[test]
    fn test_message_serialization() {
        // Error
        let message = NostrConnectMessage::response(
            "2581081643",
            NostrConnectResponse::new(
                Some(ResponseResult::Error),
                Some(String::from("Empty response")),
            ),
        );
        let json = r#"{"id":"2581081643","result":"error","error":"Empty response"}"#;
        assert_eq!(message.as_json(), json);

        // Sign event
        let unsigned = UnsignedEvent::from_json(r#"{"created_at":1710854115,"content":"Testing rust-nostr NIP46 signer [bunker]","tags":[],"kind":1,"pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","id":"236ad3390704e1bf435f40143fb3de163723aeaa8f25c3bf12a0ac4d9a4b56a7"}"#).unwrap();
        let json = r#"{"id":"3047714669","method":"sign_event","params":["{\"id\":\"236ad3390704e1bf435f40143fb3de163723aeaa8f25c3bf12a0ac4d9a4b56a7\",\"pubkey\":\"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3\",\"created_at\":1710854115,\"kind\":1,\"tags\":[],\"content\":\"Testing rust-nostr NIP46 signer [bunker]\"}"]}"#;
        let message = NostrConnectMessage::Request {
            id: String::from("3047714669"),
            method: NostrConnectMethod::SignEvent,
            params: vec![unsigned.as_json()],
        };
        assert_eq!(message.as_json(), json);

        let req = message.to_request().unwrap();
        assert_eq!(req, NostrConnectRequest::SignEvent(unsigned));
    }

    #[test]
    fn test_message_deserialization() {
        // Connect
        let json = r#"{"id":"2845841889","method":"connect","params":["79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3"]}"#;
        let message = NostrConnectMessage::from_json(json).unwrap();
        let expected_msg = NostrConnectMessage::Request {
            id: String::from("2845841889"),
            method: NostrConnectMethod::Connect,
            params: vec![String::from(
                "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3",
            )],
        };
        assert_eq!(message, expected_msg);
        let req = message.to_request().unwrap();
        assert_eq!(
            req,
            NostrConnectRequest::Connect {
                public_key: PublicKey::from_hex(
                    "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3"
                )
                .unwrap(),
                secret: None,
            }
        );

        // Connect ACK
        let json = r#"{"id":"2581081643","result":"ack","error":null}"#;
        let message = NostrConnectMessage::from_json(json).unwrap();
        assert_eq!(
            message,
            NostrConnectMessage::response(
                "2581081643",
                NostrConnectResponse::new(Some(ResponseResult::Ack), None)
            )
        );

        // Error
        let json = r#"{"id":"2581081643","result":"error","error":"Empty response"}"#;
        let message = NostrConnectMessage::from_json(json).unwrap();
        assert_eq!(
            message,
            NostrConnectMessage::response(
                "2581081643",
                NostrConnectResponse::new(
                    Some(ResponseResult::Error),
                    Some(String::from("Empty response"))
                )
            )
        );

        // Sign event
        let event = Event::from_json(r#"{"created_at":1710854115,"content":"Testing rust-nostr NIP46 signer [bunker]","tags":[],"kind":1,"pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","id":"236ad3390704e1bf435f40143fb3de163723aeaa8f25c3bf12a0ac4d9a4b56a7","sig":"509b8fe51c1e4c4cc55a0b2032b70bfb683f1da6c62e4e5b0da7175eab99b18c67862deaaea80cf31acedb9ad3022ebf54fd0cb6c9d1297a96541848d2035d92"}"#).unwrap();
        let json = r#"{"id":"3047714669","result":"{\"id\":\"236ad3390704e1bf435f40143fb3de163723aeaa8f25c3bf12a0ac4d9a4b56a7\",\"pubkey\":\"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3\",\"created_at\":1710854115,\"kind\":1,\"tags\":[],\"content\":\"Testing rust-nostr NIP46 signer [bunker]\",\"sig\":\"509b8fe51c1e4c4cc55a0b2032b70bfb683f1da6c62e4e5b0da7175eab99b18c67862deaaea80cf31acedb9ad3022ebf54fd0cb6c9d1297a96541848d2035d92\"}","error":null}"#;
        let message = NostrConnectMessage::from_json(json).unwrap();
        assert_eq!(
            message,
            NostrConnectMessage::response(
                "3047714669",
                NostrConnectResponse::new(Some(ResponseResult::SignEvent(Box::new(event))), None),
            )
        );

        // Encryption
        let ciphertext = "ArY1I2xC2yDwIbuNHN/1ynXdGgzHLqdCrXUPMwELJPc7s7JqlCMJBAIIjfkpHReBPXeoMCyuClwgbT419jUWU1PwaNl4FEQYKCDKVJz+97Mp3K+Q2YGa77B6gpxB/lr1QgoqpDf7wDVrDmOqGoiPjWDqy8KzLueKDcm9BVP8xeTJIxs=";
        let json = r#"{"id":"3047714669","result":"ArY1I2xC2yDwIbuNHN/1ynXdGgzHLqdCrXUPMwELJPc7s7JqlCMJBAIIjfkpHReBPXeoMCyuClwgbT419jUWU1PwaNl4FEQYKCDKVJz+97Mp3K+Q2YGa77B6gpxB/lr1QgoqpDf7wDVrDmOqGoiPjWDqy8KzLueKDcm9BVP8xeTJIxs=","error":null}"#;
        let message = NostrConnectMessage::from_json(json).unwrap();
        assert_eq!(
            message,
            NostrConnectMessage::response(
                "3047714669",
                NostrConnectResponse::new(
                    Some(ResponseResult::Nip44Encrypt {
                        ciphertext: ciphertext.to_string()
                    }),
                    None
                )
            )
        );

        // Decryption
        let plaintext = "Hello world!";
        let json = r#"{"id":"3047714669","result":"Hello world!","error":null}"#;
        let message = NostrConnectMessage::from_json(json).unwrap();
        assert_eq!(
            message,
            NostrConnectMessage::response(
                "3047714669",
                NostrConnectResponse::new(
                    Some(ResponseResult::Nip44Decrypt {
                        plaintext: plaintext.to_string()
                    }),
                    None
                )
            )
        );
    }
}
