// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP46
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use std::fmt;

use bitcoin_hashes::sha256::Hash as Sha256Hash;
use bitcoin_hashes::Hash;
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::RngCore;
use secp256k1::XOnlyPublicKey;
use serde_json::{json, Value};
use url::form_urlencoded::byte_serialize;
use url::Url;

#[cfg(feature = "base")]
use crate::UnsignedEvent;

/// Method
pub enum Method {
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

impl Method {
    /// Get method name
    pub fn name(&self) -> String {
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

    /// Get method params
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

/// Message
#[derive(Debug, Clone)]
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
        result: Value,
        /// Reason, if failed
        error: String,
    },
}

impl Message {
    /// Compose `Request` message
    pub fn request(method: Method) -> Self {
        Self::Request {
            id: Self::random_id(),
            method: method.name(),
            params: method.params(),
        }
    }

    fn random_id() -> String {
        let mut os_random = [0u8; 32];
        OsRng.fill_bytes(&mut os_random);
        let hash = Sha256Hash::hash(&os_random).to_string();
        hash[..16].to_string()
    }

    /// Serialize [`Message`] as JSON string
    pub fn as_json(&self) -> String {
        match self {
            Self::Request { id, method, params } => {
                json!({"id": id, "method": method, "params": params}).to_string()
            }
            Self::Response { id, result, error } => {
                json!({"id": id, "result": result, "error": error}).to_string()
            }
        }
    }
}

fn url_encode<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    byte_serialize(data.as_ref()).collect()
}

/// Nostr Connect URI
#[derive(Debug, Clone)]
pub struct NostrConnectURI {
    /// Pubkey
    pub public_key: XOnlyPublicKey,
    /// URL of the relay of choice where the `App` is connected and the `Signer` must send and listen for messages.
    pub relay_url: Url,
    /// Human-readable name of the `App`
    pub name: String,
    /// URL of the website requesting the connection
    pub url: Option<Url>,
    /// Description of the `App`
    pub description: Option<String>,
    /// Array of URLs for icons of the `App`
    pub icons: Option<Vec<Url>>,
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
            name: app_name.into(),
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

impl fmt::Display for NostrConnectURI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut metadata = json!({
            "name": self.name,
        });
        if let Some(url) = &self.url {
            metadata["url"] = json!(url);
        }
        if let Some(description) = &self.description {
            metadata["description"] = json!(description);
        }
        if let Some(icons) = &self.icons {
            metadata["icons"] = json!(icons);
        }
        write!(
            f,
            "nostrconnect://{}?relay={}&metadata={}",
            self.public_key,
            url_encode(self.relay_url.to_string()),
            url_encode(metadata.to_string())
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
}
