// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP05
//!
//! <https://github.com/nostr-protocol/nips/blob/master/05.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use std::net::SocketAddr;

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;
use reqwest::{Client, Response};
use serde_json::Value;
use url::Url;

use crate::nips::nip19::Nip19Profile;
use crate::{key, PublicKey};

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

fn compose_url<S>(nip05: S) -> Result<(String, String), Error>
where
    S: AsRef<str>,
{
    let nip05: &str = nip05.as_ref();
    let data: Vec<&str> = nip05.split('@').collect();
    if data.len() != 2 {
        return Err(Error::InvalidFormat);
    }
    let name: &str = data[0];
    let domain: &str = data[1];
    let url = format!("https://{domain}/.well-known/nostr.json?name={name}");
    Ok((url, name.to_string()))
}

fn get_key_from_json<S>(json: &Value, name: S) -> Option<PublicKey>
where
    S: AsRef<str>,
{
    let name: &str = name.as_ref();
    json.get("names")
        .and_then(|names| names.get(name))
        .and_then(|value| value.as_str())
        .and_then(|pubkey| PublicKey::from_hex(pubkey).ok())
}

#[inline]
fn get_relays_from_json(json: Value, pk: PublicKey) -> Vec<Url> {
    json.get("relays")
        .and_then(|relays| relays.get(pk.to_string()))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}

#[inline]
fn get_nip46_relays_from_json(json: Value, pk: PublicKey) -> Vec<Url> {
    json.get("nip46")
        .and_then(|relays| relays.get(pk.to_string()))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}

fn verify_json<S>(public_key: &PublicKey, json: &Value, name: S) -> bool
where
    S: AsRef<str>,
{
    if let Some(pubkey) = get_key_from_json(json, name) {
        if &pubkey == public_key {
            return true;
        }
    }

    false
}

async fn make_req<S>(nip05: S, _proxy: Option<SocketAddr>) -> Result<(Value, String), Error>
where
    S: AsRef<str>,
{
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
    let json: Value = serde_json::from_str(&res.text().await?)?;

    Ok((json, name))
}

/// Verify NIP05
///
/// **Proxy is ignored for WASM targets!**
pub async fn verify<S>(
    public_key: &PublicKey,
    nip05: S,
    _proxy: Option<SocketAddr>,
) -> Result<bool, Error>
where
    S: AsRef<str>,
{
    let (json, name) = make_req(nip05, _proxy).await?;
    Ok(verify_json(public_key, &json, name))
}

/// Get [Nip19Profile] from NIP05 (public key and list of advertised relays)
///
/// **Proxy is ignored for WASM targets!**
pub async fn get_profile<S>(nip05: S, _proxy: Option<SocketAddr>) -> Result<Nip19Profile, Error>
where
    S: AsRef<str>,
{
    let (json, name) = make_req(nip05, _proxy).await?;

    let public_key: PublicKey = get_key_from_json(&json, name).ok_or(Error::ImpossibleToVerify)?;
    let relays: Vec<Url> = get_relays_from_json(json, public_key);

    Ok(Nip19Profile { public_key, relays })
}

/// Get [Nip19Profile] from NIP05 (public key and list of advertised relays)
///
/// **Proxy is ignored for WASM targets!**
pub async fn get_nip46<S>(
    nip05: S,
    _proxy: Option<SocketAddr>,
) -> Result<(PublicKey, Vec<Url>), Error>
where
    S: AsRef<str>,
{
    let (json, name) = make_req(nip05, _proxy).await?;

    let public_key: PublicKey = get_key_from_json(&json, name).ok_or(Error::ImpossibleToVerify)?;
    let relays: Vec<Url> = get_nip46_relays_from_json(json, public_key);

    Ok((public_key, relays))
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
        assert!(verify_json(&public_key, &json, name));
        assert!(verify_json(&public_key, &json, "yuki"));

        let public_key =
            PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
                .unwrap();
        assert!(!verify_json(&public_key, &json, "yuki"));
    }
}
