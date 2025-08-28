// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP05: Mapping Nostr keys to DNS-based internet identifiers
//!
//! <https://github.com/nostr-protocol/nips/blob/master/05.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::Split;

use serde_json::Value;

use crate::types::url::{ParseError, Url};
use crate::{PublicKey, RelayUrl};

/// `NIP05` error
#[derive(Debug)]
pub enum Error {
    /// Error deserializing JSON data
    Json(serde_json::Error),
    /// Url error
    Url(ParseError),
    /// Invalid format
    InvalidFormat,
    /// Impossible to verify
    ImpossibleToVerify,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => e.fmt(f),
            Self::Url(e) => e.fmt(f),
            Self::InvalidFormat => f.write_str("invalid format"),
            Self::ImpossibleToVerify => f.write_str("impossible to verify"),
        }
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

/// NIP-05 address
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nip05Address {
    name: String,
    domain: String,
    url: Url,
}

impl fmt::Display for Nip05Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.domain)
    }
}

impl Nip05Address {
    /// Parse a NIP-05 address (i.e., `yuki@yukikishimoto.com`).
    pub fn parse(address: &str) -> Result<Self, Error> {
        // Split the address into parts
        let mut split: Split<char> = address.split('@');

        // Get first and second parts
        let (name, domain): (&str, &str) = match (split.next(), split.next()) {
            // First and second parts found
            (Some(name), Some(domain)) => (name, domain),
            // Found only a part, meaning that `@` is missing
            (Some(address), None) => ("_", address),
            // Invalid format
            _ => return Err(Error::InvalidFormat),
        };

        // Compose and parse the URLS
        let url: String = format!("https://{domain}/.well-known/nostr.json?name={name}");
        let url: Url = Url::parse(&url)?;

        Ok(Self {
            name: name.to_string(),
            domain: domain.to_string(),
            url,
        })
    }

    /// Get the name value
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the domain value
    #[inline]
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Get url for NIP05 address
    ///
    /// This can be used to make a `GET` HTTP request and get the NIP-05 JSON.
    #[inline]
    pub fn url(&self) -> &Url {
        &self.url
    }
}

/// NIP-05 profile
///
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nip05Profile {
    /// Public key
    pub public_key: PublicKey,
    /// Relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/05.md>
    pub relays: Vec<RelayUrl>,
    /// NIP46 relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/46.md>
    pub nip46: Vec<RelayUrl>,
}

impl Nip05Profile {
    /// Extract a NIP-05 profile from JSON
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/05.md>
    pub fn from_json(address: &Nip05Address, json: &Value) -> Result<Self, Error> {
        let public_key: PublicKey =
            get_key_from_json(json, address).ok_or(Error::ImpossibleToVerify)?;
        let relays: Vec<RelayUrl> = get_relays_from_json(json, &public_key);
        let nip46: Vec<RelayUrl> = get_nip46_relays_from_json(json, &public_key);

        Ok(Self {
            public_key,
            relays,
            nip46,
        })
    }

    /// Extract a NIP-05 profile from raw JSON
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/05.md>
    #[inline]
    pub fn from_raw_json(address: &Nip05Address, raw_json: &str) -> Result<Self, Error> {
        let json: Value = serde_json::from_str(raw_json)?;
        Self::from_json(address, &json)
    }
}

#[inline]
fn get_key_from_json(json: &Value, address: &Nip05Address) -> Option<PublicKey> {
    json.get("names")
        .and_then(|names| names.get(&address.name))
        .and_then(|value| value.as_str())
        .and_then(|pubkey| PublicKey::from_hex(pubkey).ok())
}

#[inline]
fn get_relays_from_json(json: &Value, pk: &PublicKey) -> Vec<RelayUrl> {
    json.get("relays")
        .and_then(|relays| relays.get(pk.to_hex()))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}

#[inline]
fn get_nip46_relays_from_json(json: &Value, pk: &PublicKey) -> Vec<RelayUrl> {
    json.get("nip46")
        .and_then(|relays| relays.get(pk.to_hex()))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}

/// Verify a NIP-05 from JSON
///
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
pub fn verify_from_json(public_key: &PublicKey, address: &Nip05Address, json: &Value) -> bool {
    if let Some(pubkey) = get_key_from_json(json, address) {
        if &pubkey == public_key {
            return true;
        }
    }

    false
}

/// Verify a NIP-05 from raw JSON
#[inline]
pub fn verify_from_raw_json(
    public_key: &PublicKey,
    address: &Nip05Address,
    raw_json: &str,
) -> Result<bool, Error> {
    let json: Value = serde_json::from_str(raw_json)?;
    Ok(verify_from_json(public_key, address, &json))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_nip05() {
        // nostr.json
        let json: &str = r#"{
            "names": {
              "yuki": "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272",
              "_": "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"
            }
          }"#;
        let json: Value = serde_json::from_str(json).unwrap();

        let address = Nip05Address::parse("_@yukikishimoto.com").unwrap();
        assert_eq!(
            address.url().to_string(),
            "https://yukikishimoto.com/.well-known/nostr.json?name=_"
        );
        assert_eq!(address.name(), "_");

        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();
        assert!(verify_from_json(&public_key, &address, &json));

        let public_key =
            PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
                .unwrap();
        assert!(!verify_from_json(&public_key, &address, &json));
    }

    #[test]
    fn test_verify_nip05_without_local() {
        // nostr.json
        let json: &str = r#"{
            "names": {
              "yuki": "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272",
              "_": "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"
            }
          }"#;
        let json: Value = serde_json::from_str(json).unwrap();

        let address = Nip05Address::parse("yukikishimoto.com").unwrap();
        assert_eq!(
            address.url().to_string(),
            "https://yukikishimoto.com/.well-known/nostr.json?name=_"
        );
        assert_eq!(address.name(), "_");

        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();
        assert!(verify_from_json(&public_key, &address, &json));

        let public_key =
            PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
                .unwrap();
        assert!(!verify_from_json(&public_key, &address, &json));
    }

    #[test]
    fn test_nip05_profile_from_json() {
        // nostr.json
        let json = r#"{
  "names": {
    "nostr-tool-test-user": "94a9eb13c37b3c1519169a426d383c51530f4fe8f693c62f32b321adfdd4ec7f",
    "robosatsob": "3b57518d02e6acfd5eb7198530b2e351e5a52278fb2499d14b66db2b5791c512",
    "0xtr": "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
    "_": "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a"
  },
  "relays": {
    "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a": [ "wss://nostr.oxtr.dev", "wss://relay.damus.io", "wss://relay.nostr.band" ]
  }
}"#;
        let json: Value = serde_json::from_str(json).unwrap();

        let pubkey =
            PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
                .unwrap();
        let address = Nip05Address::parse("0xtr@oxtr.dev").unwrap();

        let profile = Nip05Profile::from_json(&address, &json).unwrap();

        assert_eq!(profile.public_key, pubkey);
        assert_eq!(
            profile.relays,
            vec![
                RelayUrl::parse("wss://nostr.oxtr.dev").unwrap(),
                RelayUrl::parse("wss://relay.damus.io").unwrap(),
                RelayUrl::parse("wss://relay.nostr.band").unwrap()
            ]
        );
        assert!(profile.nip46.is_empty());
    }
}
