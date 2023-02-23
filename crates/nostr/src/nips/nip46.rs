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

/// Compose URI
pub fn uri<S>(
    public_key: XOnlyPublicKey,
    relay_url: Url,
    app_name: S,
    url: Option<Url>,
    description: Option<S>,
    icons: Option<Vec<Url>>,
) -> String
where
    S: Into<String>,
{
    let mut metadata = json!({
        "name": app_name.into(),
    });
    if let Some(url) = url {
        metadata["url"] = json!(url);
    }
    if let Some(description) = description.map(|d| d.into()) {
        metadata["description"] = json!(description);
    }
    if let Some(icons) = icons {
        metadata["icons"] = json!(icons);
    }
    format!(
        "nostrconnect://{public_key}?relay={}&metadata={}",
        url_encode(relay_url.to_string()),
        url_encode(metadata.to_string())
    )
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
        let uri = uri(pubkey, relay_url, app_name, None, None, None);
        assert_eq!(uri, "nostrconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io%2F&metadata=%7B%22name%22%3A%22Example%22%7D".to_string());
        Ok(())
    }
}
