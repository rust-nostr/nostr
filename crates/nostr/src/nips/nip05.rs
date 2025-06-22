// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP05: Mapping Nostr keys to DNS-based internet identifiers
//!
//! <https://github.com/nostr-protocol/nips/blob/master/05.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use serde_json::Value;
use crate::{key, PublicKey, RelayUrl};

/// `NIP05` error
#[derive(Debug)]
pub enum Error {
    /// Error deserializing JSON data
    Json(serde_json::Error),
    /// Keys error
    Keys(key::Error),
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
            Self::Json(e) => write!(f, "impossible to deserialize NIP05 data: {e}"),
            Self::Keys(e) => write!(f, "{e}"),
            Self::InvalidFormat => write!(f, "invalid format"),
            Self::ImpossibleToVerify => write!(f, "impossible to verify"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

/// NIP05 profile
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

/// NIP05 request information
///
/// Contains the URL and name needed to verify a NIP05 identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nip05Request {
    /// The URL to fetch
    pub url: String,
    /// The name to verify
    pub name: String,
}

impl Nip05Request {
    /// Create a new NIP05 request from identifier
    pub fn new<S>(nip05: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let (url, name) = compose_url(nip05.as_ref())?;
        Ok(Self {
            url,
            name: name.to_string(),
        })
    }

    /// Get the URL to fetch
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the name to verify
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Get the NIP05 URL for a given identifier
/// (just an option if you do not want to use the full verification logic)
///
/// Returns the URL that should be fetched for NIP05 verification
pub fn get_nip05_url<S>(nip05: S) -> Result<String, Error>
where
    S: AsRef<str>,
{
    let (url, _) = compose_url(nip05.as_ref())?;
    Ok(url)
}

fn compose_url(nip05: &str) -> Result<(String, &str), Error> {
    let mut split = nip05.split('@');
    if let (Some(name), Some(domain)) = (split.next(), split.next()) {
        let url = format!("https://{domain}/.well-known/nostr.json?name={name}");
        return Ok((url, name));
    }
    Err(Error::InvalidFormat)
}

#[inline]
fn get_key_from_json(json: &Value, name: &str) -> Option<PublicKey> {
    json.get("names")
        .and_then(|names| names.get(name))
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

fn verify_from_json(public_key: &PublicKey, json: &Value, name: &str) -> bool {
    if let Some(pubkey) = get_key_from_json(json, name) {
        if &pubkey == public_key {
            return true;
        }
    }

    false
}

/// Parse JSON data and extract name for NIP05 operations
// fn parse_json_data(data: &[u8], nip05: &str) -> Result<(Value, &str), Error> {
//     let (_url, name) = compose_url(nip05)?;
//     let json: Value = serde_json::from_slice(data)?;
//     Ok((json, name))
// }

/// Verify NIP05 using pre-fetched JSON data
///
/// Takes the JSON response from the .well-known/nostr.json endpoint
/// and verifies if public key matches the NIP05 identifier.
/// 
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
pub fn verify_from_response(
    public_key: &PublicKey,
    json_response: &str,
    request: &Nip05Request,
) -> Result<bool, Error> {
    let json: Value = serde_json::from_str(json_response)?;
    Ok(verify_from_json(public_key, &json, &request.name))
}

/// Get NIP05 profile using pre-fetched JSON data (I/O free)
///
/// Takes the JSON response from the .well-known/nostr.json endpoint 
/// and extracts profile information
/// 
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
pub fn profile_from_response(
    json_response: &str,
    request: &Nip05Request,
) -> Result<Nip05Profile, Error> {
    let json: Value = serde_json::from_str(json_response)?;

    let public_key: PublicKey = get_key_from_json(&json, &request.name).ok_or(Error::ImpossibleToVerify)?;
    let relays: Vec<RelayUrl> = get_relays_from_json(&json, &public_key);
    let nip46: Vec<RelayUrl> = get_nip46_relays_from_json(&json, &public_key);

    Ok(Nip05Profile {
        public_key,
        relays,
        nip46,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nip05_request() {
        let request = Nip05Request::new("_@yukikishimoto.com").unwrap();
        assert_eq!(
            request.url(),
            "https://yukikishimoto.com/.well-known/nostr.json?name=_"
        );
        assert_eq!(request.name(), "_");

        // Test invalid format
        assert!(Nip05Request::new("invalid").is_err());
    }

    #[test]
    fn test_verify_from_response() {
        let json_response = r#"{
            "names": {
              "yuki": "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272",
              "_": "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"
            }
          }"#;

        let request = Nip05Request::new("_@yukikishimoto.com").unwrap();
        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();
        
        assert!(verify_from_response(&public_key, json_response, &request).unwrap());

        let yuki_request = Nip05Request::new("yuki@yukikishimoto.com").unwrap();
        assert!(verify_from_response(&public_key, json_response, &yuki_request).unwrap());

        let wrong_key =
            PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
                .unwrap();
        assert!(!verify_from_response(&wrong_key, json_response, &yuki_request).unwrap());
    }
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

        let (url, name) = compose_url("_@yukikishimoto.com").unwrap();
        assert_eq!(
            url,
            "https://yukikishimoto.com/.well-known/nostr.json?name=_"
        );
        assert_eq!(name, "_");

        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();
        assert!(verify_from_json(&public_key, &json, name));
        assert!(verify_from_json(&public_key, &json, "yuki"));

        let public_key =
            PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
                .unwrap();
        assert!(!verify_from_json(&public_key, &json, "yuki"));
    }
}
