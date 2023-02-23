// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP46
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

use bitcoin::XOnlyPublicKey;
use serde_json::{json, Value};
use url::form_urlencoded::byte_serialize;
use url::Url;

#[cfg(feature = "base")]
use crate::Event;

/// Method
pub enum Method {
    /// Describe
    Describe,
    /// Get public key
    GetPublicKey,
    /// Sign [`Event`]
    #[cfg(feature = "base")]
    SignEvent(Event),
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

/// Request message
pub struct Request {
    /// Request id
    pub id: String,
    /// Method
    pub method: Method,
    /// params
    pub params: Vec<Value>,
}

/// Response message
pub struct Response {
    /// Request id
    pub id: String,
    /// Result
    pub result: Value,
    /// Reason, if failed
    pub error: String,
}

fn url_encode<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    byte_serialize(data.as_ref()).collect()
}

/// Nostr Connect URI
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
    pub fn new<S>(
        public_key: XOnlyPublicKey,
        relay_url: Url,
        app_name: S,
        url: Option<Url>,
        description: Option<S>,
        icons: Option<Vec<Url>>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            public_key,
            relay_url,
            name: app_name.into(),
            url,
            description: description.map(|d| d.into()),
            icons,
        }
    }
}

impl ToString for NostrConnectURI {
    fn to_string(&self) -> String {
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
        format!(
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
        let uri = NostrConnectURI::new(pubkey, relay_url, app_name, None, None, None);
        assert_eq!(
            uri.to_string(),
            "nostrconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&metadata=%7B%22name%22%3A%22Example%22%7D".to_string()
        );
        Ok(())
    }
}
