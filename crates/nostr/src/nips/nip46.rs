// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP46
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

#[cfg(feature = "base")]
use secp256k1::schnorr::Signature;
use secp256k1::{rand, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::form_urlencoded::byte_serialize;
use url::Url;

#[cfg(feature = "base")]
use crate::UnsignedEvent;

/// NIP46 error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JSON error
    #[error(transparent)]
    JSON(#[from] serde_json::Error),
    /// Url parse error
    #[error(transparent)]
    Url(#[from] url::ParseError),
    /// Secp256k1 error
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    /// Invalid request
    #[error("invalid request")]
    InvalidRequest,
    /// Unsupported method
    #[error("unsupported method")]
    UnsupportedMethod,
    /// Invalid URI
    #[error("invalid uri")]
    InvalidURI,
    /// Invalid URI scheme
    #[error("invalid uri scheme")]
    InvalidURIScheme,
}

/// Request
#[derive(Debug, Clone)]
pub enum Request {
    /// Describe
    Describe,
    /// Get public key
    GetPublicKey,
    /// Sign [`UnsignedEvent`]
    #[cfg(feature = "base")]
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
        conditions: String,
    },
    /// Get relays
    GetRelays,
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
}

impl Request {
    /// Get req method
    pub fn method(&self) -> String {
        match self {
            Self::Describe => "describe".to_string(),
            Self::GetPublicKey => "get_public_key".to_string(),
            #[cfg(feature = "base")]
            Self::SignEvent(_) => "sign_event".to_string(),
            Self::Connect(_) => "connect".to_string(),
            Self::Disconnect => "disconnect".to_string(),
            Self::Delegate {
                public_key: _,
                conditions: _,
            } => "delegate".to_string(),
            Self::GetRelays => "get_relays".to_string(),
            Self::Nip04Encrypt {
                public_key: _,
                text: _,
            } => "nip04_encrypt".to_string(),
            Self::Nip04Decrypt {
                public_key: _,
                text: _,
            } => "nip04_decrypt".to_string(),
        }
    }

    /// Get req params
    pub fn params(&self) -> Vec<Value> {
        match self {
            Self::Describe => Vec::new(),
            Self::GetPublicKey => Vec::new(),
            #[cfg(feature = "base")]
            Self::SignEvent(event) => vec![json!(event)],
            Self::Connect(pubkey) => vec![json!(pubkey)],
            Self::Disconnect => Vec::new(),
            Self::Delegate {
                public_key,
                conditions,
            } => vec![json!(public_key), json!(conditions)],
            Self::GetRelays => Vec::new(),
            Self::Nip04Encrypt { public_key, text } => vec![json!(public_key), json!(text)],
            Self::Nip04Decrypt { public_key, text } => vec![json!(public_key), json!(text)],
        }
    }
}

/// Response
#[derive(Debug, Clone)]
pub enum Response {
    /// Describe
    Describe(Value),
    /// Get public key
    GetPublicKey(XOnlyPublicKey),
    /// Sign event
    #[cfg(feature = "base")]
    SignEvent(Signature),
}

/// Message
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Compose `Request` message
    pub fn request(req: Request) -> Self {
        Self::Request {
            id: Self::random_id(),
            method: req.method(),
            params: req.params(),
        }
    }

    /// Compose `Response` message
    pub fn response(req_id: String, res: Response) -> Self {
        Self::Response {
            id: req_id,
            result: Some(match res {
                Response::Describe(value) => value,
                Response::GetPublicKey(pubkey) => json!(pubkey),
                Response::SignEvent(sig) => json!(sig),
            }),
            error: None,
        }
    }

    fn random_id() -> String {
        rand::random::<u32>().to_string()
    }

    /// Get [`Message`] id
    pub fn id(&self) -> String {
        match self {
            Self::Request {
                id,
                method: _,
                params: _,
            } => id.to_owned(),
            Self::Response {
                id,
                result: _,
                error: _,
            } => id.to_owned(),
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
        if let Message::Request {
            id: _,
            method,
            params,
        } = self
        {
            match method.as_str() {
                "describe" => Ok(Request::Describe),
                "get_public_key" => Ok(Request::GetPublicKey),
                #[cfg(feature = "base")]
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
                    if let Some(value) = params.first() {
                        let pubkey: XOnlyPublicKey = serde_json::from_value(value.to_owned())?;
                        Ok(Request::Connect(pubkey))
                    } else {
                        Err(Error::InvalidRequest)
                    }
                }
                "disconnect" => todo!(),
                "delegate" => todo!(),
                "get_relays" => todo!(),
                "nip04_encrypt" => todo!(),
                "nip04_decrypt" => todo!(),
                _ => Err(Error::UnsupportedMethod),
            }
        } else {
            Err(Error::InvalidRequest)
        }
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
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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

// nostrconnect://79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3?relay=ws%3A%2F%2F192.168.7.233%3A7777%2F&metadata=%7B%22name%22%3A%22Nostr+SDK%22%2C%22url%22%3A%22https%3A%2F%2Fnostr.info%2F%22%7D

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
    use std::str::FromStr;

    use super::*;
    use crate::Result;

    #[test]
    fn test_uri() -> Result<()> {
        let pubkey = XOnlyPublicKey::from_str(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )?;
        let relay_url = Url::parse("wss://relay.damus.io")?;
        let app_name = "Example";
        let uri = NostrConnectURI::new(pubkey, relay_url, app_name);
        assert_eq!(
            uri.to_string(),
            "nostrconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&metadata=%7B%22name%22%3A%22Example%22%7D".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_parse_uri() -> Result<()> {
        let uri = "nostrconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&metadata=%7B%22name%22%3A%22Example%22%7D";
        let uri = NostrConnectURI::from_str(uri)?;

        let pubkey = XOnlyPublicKey::from_str(
            "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
        )?;
        let relay_url = Url::parse("wss://relay.damus.io")?;
        let app_name = "Example";
        assert_eq!(uri, NostrConnectURI::new(pubkey, relay_url, app_name));
        Ok(())
    }
}
