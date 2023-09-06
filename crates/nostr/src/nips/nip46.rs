// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP46
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use alloc::borrow::{Cow, ToOwned};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand;
use bitcoin::secp256k1::rand::{CryptoRng, Rng, RngCore};
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Message as Secp256k1Message, Secp256k1, Signing, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url_fork::form_urlencoded::byte_serialize;
use url_fork::{ParseError, Url};

use super::nip04;
use super::nip26::{self, sign_delegation_with_ctx, Conditions};
use crate::event::unsigned::{self, UnsignedEvent};
use crate::key::{self, Keys};
use crate::Event;
#[cfg(feature = "std")]
use crate::SECP256K1;

/// NIP46 error
#[derive(Debug)]
pub enum Error {
    /// Key error
    Key(key::Error),
    /// JSON error
    Json(serde_json::Error),
    /// Url parse error
    Url(ParseError),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// NIP04 error
    NIP04(nip04::Error),
    /// NIP26 error
    NIP26(nip26::Error),
    /// Unsigned event error
    UnsignedEvent(unsigned::Error),
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
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "Key: {e}"),
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::Url(e) => write!(f, "Url: {e}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::NIP04(e) => write!(f, "NIP04: {e}"),
            Self::NIP26(e) => write!(f, "NIP26: {e}"),
            Self::UnsignedEvent(e) => write!(f, "{e}"),
            Self::InvalidRequest => write!(f, "Invalid request"),
            Self::InvalidParamsLength => write!(f, "Too many/few params"),
            Self::UnsupportedMethod(name) => write!(f, "Unsupported method: {name}"),
            Self::InvalidURI => write!(f, "Invalid uri"),
            Self::InvalidURIScheme => write!(f, "Invalid uri scheme"),
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

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<nip04::Error> for Error {
    fn from(e: nip04::Error) -> Self {
        Self::NIP04(e)
    }
}

impl From<nip26::Error> for Error {
    fn from(e: nip26::Error) -> Self {
        Self::NIP26(e)
    }
}

impl From<unsigned::Error> for Error {
    fn from(e: unsigned::Error) -> Self {
        Self::UnsignedEvent(e)
    }
}

/// Request
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Request {
    /// Describe
    Describe,
    /// Get public key
    GetPublicKey,
    /// Sign [`UnsignedEvent`]
    SignEvent(UnsignedEvent),
    /// Connect
    Connect(XOnlyPublicKey),
    /// Disconnect
    Disconnect,
    /// Delegate
    Delegate {
        /// Pubkey
        public_key: XOnlyPublicKey,
        /// NIP26 conditions
        conditions: Conditions,
    },
    /// Encrypt text (NIP04)
    Nip04Encrypt {
        /// Pubkey
        public_key: XOnlyPublicKey,
        /// Plain text
        text: String,
    },
    /// Decrypt (NIP04)
    Nip04Decrypt {
        /// Pubkey
        public_key: XOnlyPublicKey,
        /// Ciphertext
        text: String,
    },
    /// Sign Schnorr
    SignSchnorr(String),
}

impl Request {
    /// Get req method
    pub fn method(&self) -> String {
        match self {
            Self::Describe => "describe".to_string(),
            Self::GetPublicKey => "get_public_key".to_string(),
            Self::SignEvent(_) => "sign_event".to_string(),
            Self::Connect(_) => "connect".to_string(),
            Self::Disconnect => "disconnect".to_string(),
            Self::Delegate { .. } => "delegate".to_string(),
            Self::Nip04Encrypt { .. } => "nip04_encrypt".to_string(),
            Self::Nip04Decrypt { .. } => "nip04_decrypt".to_string(),
            Self::SignSchnorr(_) => "sign_schnorr".to_string(),
        }
    }

    /// Get req params
    pub fn params(&self) -> Vec<Value> {
        match self {
            Self::Describe => Vec::new(),
            Self::GetPublicKey => Vec::new(),
            Self::SignEvent(event) => vec![json!(event)],
            Self::Connect(pubkey) => vec![json!(pubkey)],
            Self::Disconnect => Vec::new(),
            Self::Delegate {
                public_key,
                conditions,
            } => vec![json!(public_key), json!(conditions)],
            Self::Nip04Encrypt { public_key, text } => vec![json!(public_key), json!(text)],
            Self::Nip04Decrypt { public_key, text } => vec![json!(public_key), json!(text)],
            Self::SignSchnorr(value) => vec![json!(value)],
        }
    }

    /// Generate [`Response`] message for [`Request`]
    #[cfg(feature = "std")]
    pub fn generate_response(self, keys: &Keys) -> Result<Option<Response>, Error> {
        self.generate_response_with_ctx(&SECP256K1, &mut rand::thread_rng(), keys)
    }

    /// Generate [`Response`] message for [`Request`]
    pub fn generate_response_with_ctx<C, R>(
        self,
        secp: &Secp256k1<C>,
        rng: &mut R,
        keys: &Keys,
    ) -> Result<Option<Response>, Error>
    where
        C: Signing,
        R: Rng + CryptoRng,
    {
        let res: Option<Response> = match self {
            Self::Describe => Some(Response::Describe(vec![
                String::from("describe"),
                String::from("get_public_key"),
                String::from("sign_event"),
                String::from("connect"),
                String::from("disconnect"),
                String::from("delegate"),
                String::from("nip04_encrypt"),
                String::from("nip04_decrypt"),
                String::from("sign_schnorr"),
            ])),
            Self::GetPublicKey => Some(Response::GetPublicKey(keys.public_key())),
            Self::SignEvent(unsigned_event) => {
                let signed_event = unsigned_event.sign_with_ctx(secp, rng, keys)?;
                Some(Response::SignEvent(signed_event))
            }
            Self::Connect(_) => None,
            Self::Disconnect => None,
            Self::Delegate {
                public_key,
                conditions,
            } => {
                let sig =
                    sign_delegation_with_ctx(secp, rng, keys, public_key, conditions.clone())?;
                let delegation_result = DelegationResult {
                    from: keys.public_key(),
                    to: public_key,
                    cond: conditions,
                    sig,
                };

                Some(Response::Delegate(delegation_result))
            }
            Self::Nip04Encrypt { public_key, text } => {
                let encrypted_content =
                    nip04::encrypt_with_rng(rng, &keys.secret_key()?, &public_key, text)?;
                Some(Response::Nip04Encrypt(encrypted_content))
            }
            Self::Nip04Decrypt { public_key, text } => {
                let decrypted_content = nip04::decrypt(&keys.secret_key()?, &public_key, text)?;
                Some(Response::Nip04Decrypt(decrypted_content))
            }
            Self::SignSchnorr(value) => {
                let hash = Sha256Hash::hash(value.as_bytes());
                let message = Secp256k1Message::from(hash);
                let sig: Signature = keys.sign_schnorr_with_ctx(secp, &message, rng)?;
                Some(Response::SignSchnorr(sig))
            }
        };
        Ok(res)
    }
}

/// Delegation Response Result
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DelegationResult {
    /// Pubkey of Delegator
    pub from: XOnlyPublicKey,
    /// Pubkey of Delegatee
    pub to: XOnlyPublicKey,
    /// Conditions of delegation
    pub cond: Conditions,
    /// Signature of Delegation Token
    pub sig: Signature,
}

/// Response
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Response {
    /// Describe
    Describe(Vec<String>),
    /// Get public key
    GetPublicKey(XOnlyPublicKey),
    /// Sign event
    SignEvent(Event),
    /// Delegation
    Delegate(DelegationResult),
    /// Encrypted content (NIP04)
    Nip04Encrypt(String),
    /// Decrypted content (NIP04)
    Nip04Decrypt(String),
    /// Sign Schnorr
    SignSchnorr(Signature),
}

/// Message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    /// Request
    Request {
        /// Request id
        id: String,
        /// Method
        method: String,
        /// params
        params: Vec<Value>,
    },
    /// Response
    Response {
        /// Request id
        id: String,
        /// Result
        result: Option<Value>,
        /// Reason, if failed
        error: Option<String>,
    },
}

impl Message {
    /// Compose [`Request`] message
    #[cfg(feature = "std")]
    pub fn request(req: Request) -> Self {
        Self::request_with_rng(&mut rand::thread_rng(), req)
    }

    /// Compose [`Request`] message
    pub fn request_with_rng<R>(rng: &mut R, req: Request) -> Self
    where
        R: RngCore,
    {
        Self::Request {
            id: rng.next_u32().to_string(),
            method: req.method(),
            params: req.params(),
        }
    }

    /// Compose `Response` message
    pub fn response<S>(req_id: S, res: Option<Response>, error: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self::Response {
            id: req_id.into(),
            result: res.map(|res| match res {
                Response::Describe(v) => json!(v),
                Response::GetPublicKey(pubkey) => json!(pubkey),
                Response::SignEvent(sig) => json!(sig),
                Response::Delegate(delegation_result) => json!(delegation_result),
                Response::Nip04Encrypt(encrypted_content) => json!(encrypted_content),
                Response::Nip04Decrypt(decrypted_content) => json!(decrypted_content),
                Response::SignSchnorr(sig) => json!(sig),
            }),
            error: error.map(|e| e.into()),
        }
    }

    /// check if current [`Message`] is a request
    pub fn is_request(&self) -> bool {
        match self {
            Message::Request { .. } => true,
            Message::Response { .. } => false,
        }
    }

    /// Get [`Message`] id
    pub fn id(&self) -> String {
        match self {
            Self::Request { id, .. } => id.to_owned(),
            Self::Response { id, .. } => id.to_owned(),
        }
    }

    /// Serialize [`Message`] as JSON string
    pub fn as_json(&self) -> String {
        json!(self).to_string()
    }

    /// Deserialize from JSON string
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(serde_json::from_str(&json.into())?)
    }

    /// Convert [`Message`] to [`Request`]
    pub fn to_request(&self) -> Result<Request, Error> {
        if let Message::Request { method, params, .. } = self {
            match method.as_str() {
                "describe" => Ok(Request::Describe),
                "get_public_key" => Ok(Request::GetPublicKey),
                "sign_event" => {
                    if let Some(value) = params.first() {
                        let unsigned_event: UnsignedEvent =
                            serde_json::from_value(value.to_owned())?;
                        Ok(Request::SignEvent(unsigned_event))
                    } else {
                        Err(Error::InvalidRequest)
                    }
                }
                "connect" => {
                    if params.len() != 1 {
                        return Err(Error::InvalidParamsLength);
                    }

                    let pubkey: XOnlyPublicKey = serde_json::from_value(params[0].to_owned())?;
                    Ok(Request::Connect(pubkey))
                }
                "disconnect" => Ok(Request::Disconnect),
                "delegate" => {
                    if params.len() != 2 {
                        return Err(Error::InvalidParamsLength);
                    }

                    Ok(Request::Delegate {
                        public_key: serde_json::from_value(params[0].clone())?,
                        conditions: serde_json::from_value(params[1].clone())?,
                    })
                }
                "nip04_encrypt" => {
                    if params.len() != 2 {
                        return Err(Error::InvalidParamsLength);
                    }

                    Ok(Request::Nip04Encrypt {
                        public_key: serde_json::from_value(params[0].clone())?,
                        text: serde_json::from_value(params[1].clone())?,
                    })
                }
                "nip04_decrypt" => {
                    if params.len() != 2 {
                        return Err(Error::InvalidParamsLength);
                    }

                    Ok(Request::Nip04Decrypt {
                        public_key: serde_json::from_value(params[0].clone())?,
                        text: serde_json::from_value(params[1].clone())?,
                    })
                }
                "sign_schnorr" => {
                    if params.len() != 1 {
                        return Err(Error::InvalidParamsLength);
                    }

                    let value: String = serde_json::from_value(params[0].clone())?;
                    Ok(Request::SignSchnorr(value))
                }
                other => Err(Error::UnsupportedMethod(other.to_string())),
            }
        } else {
            Err(Error::InvalidRequest)
        }
    }

    /// Generate [`Response`] message for [`Request`]
    #[cfg(feature = "std")]
    pub fn generate_response(&self, keys: &Keys) -> Result<Option<Self>, Error> {
        self.generate_response_wit_ctx(&SECP256K1, &mut rand::thread_rng(), keys)
    }

    /// Generate [`Response`] message for [`Request`]
    pub fn generate_response_wit_ctx<C, R>(
        &self,
        secp: &Secp256k1<C>,
        rng: &mut R,
        keys: &Keys,
    ) -> Result<Option<Self>, Error>
    where
        C: Signing,
        R: Rng + CryptoRng,
    {
        let req = self.to_request()?;
        // TODO: remove if let SOme(res) = ...
        if let Some(res) = req.generate_response_with_ctx(secp, rng, keys)? {
            Ok(Some(Self::response(self.id(), Some(res), None)))
        } else {
            Ok(None)
        }
    }

    /// Generate error [`Response`] message for [`Request`]
    pub fn generate_error_response<S>(&self, error: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        // Check if Message is a Request
        let _req: Request = self.to_request()?;
        Ok(Self::response(self.id(), None, Some(error.into())))
    }
}

fn url_encode<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    byte_serialize(data.as_ref()).collect()
}

/// NIP46 URI Scheme
pub const NOSTR_CONNECT_URI_SCHEME: &str = "nostrconnect";

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

    /// Serialize as JSON string
    pub fn as_json(&self) -> String {
        json!(self).to_string()
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

/// Nostr Connect URI
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NostrConnectURI {
    /// App Pubkey
    pub public_key: XOnlyPublicKey,
    /// URL of the relay of choice where the `App` is connected and the `Signer` must send and listen for messages.
    pub relay_url: Url,
    /// Metadata
    pub metadata: NostrConnectMetadata,
}

impl NostrConnectURI {
    /// Create new [`NostrConnectURI`]
    pub fn new<S>(public_key: XOnlyPublicKey, relay_url: Url, app_name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            public_key,
            relay_url,
            metadata: NostrConnectMetadata {
                name: app_name.into(),
                url: None,
                description: None,
                icons: None,
            },
        }
    }

    /// Set url
    pub fn url(self, url: Url) -> Self {
        Self {
            metadata: self.metadata.url(url),
            ..self
        }
    }

    /// Set description
    pub fn description<S>(self, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            metadata: self.metadata.description(description),
            ..self
        }
    }

    /// Set icons
    pub fn icons(self, icons: Vec<Url>) -> Self {
        Self {
            metadata: self.metadata.icons(icons),
            ..self
        }
    }
}

impl FromStr for NostrConnectURI {
    type Err = Error;
    fn from_str(uri: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(uri)?;

        if url.scheme() != NOSTR_CONNECT_URI_SCHEME {
            return Err(Error::InvalidURIScheme);
        }

        if let Some(pubkey) = url.domain() {
            let public_key = XOnlyPublicKey::from_str(pubkey)?;

            let mut relay_url: Option<Url> = None;
            let mut metadata: Option<NostrConnectMetadata> = None;

            for (key, value) in url.query_pairs() {
                match key {
                    Cow::Borrowed("relay") => {
                        let value = value.to_string();
                        relay_url = Some(Url::parse(&value)?);
                    }
                    Cow::Borrowed("metadata") => {
                        let value = value.to_string();
                        metadata = Some(serde_json::from_str(&value)?);
                    }
                    _ => (),
                }
            }

            if let Some(relay_url) = relay_url {
                if let Some(metadata) = metadata {
                    return Ok(Self {
                        public_key,
                        relay_url,
                        metadata,
                    });
                }
            }
        }

        Err(Error::InvalidURI)
    }
}

impl fmt::Display for NostrConnectURI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{NOSTR_CONNECT_URI_SCHEME}://{}?relay={}&metadata={}",
            self.public_key,
            url_encode(self.relay_url.to_string()),
            url_encode(self.metadata.as_json())
        )
    }
}

#[cfg(test)]
mod test {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_uri() {
        let pubkey = XOnlyPublicKey::from_str(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )
        .unwrap();
        let relay_url = Url::parse("wss://relay.damus.io").unwrap();
        let app_name = "Example";
        let uri = NostrConnectURI::new(pubkey, relay_url, app_name);
        assert_eq!(
            uri.to_string(),
            "nostrconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&metadata=%7B%22name%22%3A%22Example%22%7D".to_string()
        );
    }

    #[test]
    fn test_parse_uri() {
        let uri = "nostrconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&metadata=%7B%22name%22%3A%22Example%22%7D";
        let uri = NostrConnectURI::from_str(uri).unwrap();

        let pubkey = XOnlyPublicKey::from_str(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )
        .unwrap();
        let relay_url = Url::parse("wss://relay.damus.io").unwrap();
        let app_name = "Example";
        assert_eq!(uri, NostrConnectURI::new(pubkey, relay_url, app_name));
    }
}
