// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP05: Mapping Nostr keys to DNS-based internet identifiers
//!
//! <https://github.com/nostr-protocol/nips/blob/master/05.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use std::net::SocketAddr;

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;
use reqwest::{Client, Response};
use serde_json::Value;

use crate::{key, PublicKey, RelayUrl};

/// `NIP05` error
#[derive(Debug)]
pub enum Error {
    /// Invalid format
    InvalidFormat,
    /// Impossible to verify
    ImpossibleToVerify,
    /// Reqwest error
    Reqwest(reqwest::Error),
    /// Error deserializing JSON data
    Json(serde_json::Error),
    /// Keys error
    Keys(key::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid format"),
            Self::ImpossibleToVerify => write!(f, "impossible to verify"),
            Self::Reqwest(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "impossible to deserialize NIP05 data: {e}"),
            Self::Keys(e) => write!(f, "{e}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
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

async fn make_req(nip05: &str, _proxy: Option<SocketAddr>) -> Result<(Value, &str), Error> {
    let (url, name) = compose_url(nip05)?;

    #[cfg(not(target_arch = "wasm32"))]
    let client: Client = {
        let mut builder = Client::builder();
        if let Some(proxy) = _proxy {
            let proxy = format!("socks5h://{proxy}");
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        builder.build()?
    };

    #[cfg(target_arch = "wasm32")]
    let client: Client = Client::new();

    let res: Response = client.get(url).send().await?;
    let json: Value = res.json().await?;

    Ok((json, name))
}

/// Verify NIP05
///
/// **Proxy is ignored for WASM targets!**
///
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
pub async fn verify<S>(
    public_key: &PublicKey,
    nip05: S,
    _proxy: Option<SocketAddr>,
) -> Result<bool, Error>
where
    S: AsRef<str>,
{
    let (json, name) = make_req(nip05.as_ref(), _proxy).await?;
    Ok(verify_from_json(public_key, &json, name))
}

/// Get NIP05 profile
///
/// **Proxy is ignored for WASM targets!**
///
/// <https://github.com/nostr-protocol/nips/blob/master/05.md>
pub async fn profile<S>(nip05: S, _proxy: Option<SocketAddr>) -> Result<Nip05Profile, Error>
where
    S: AsRef<str>,
{
    let (json, name) = make_req(nip05.as_ref(), _proxy).await?;

    let public_key: PublicKey = get_key_from_json(&json, name).ok_or(Error::ImpossibleToVerify)?;
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
