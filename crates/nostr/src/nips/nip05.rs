// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP05
//!
//! <https://github.com/nostr-protocol/nips/blob/master/05.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;

use bitcoin::secp256k1::{self, XOnlyPublicKey};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;
use serde_json::Value;

use crate::Profile;

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
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
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
            Self::Secp256k1(e) => write!(f, "{e}"),
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

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

fn compose_url<S>(nip05: S) -> Result<(String, String), Error>
where
    S: Into<String>,
{
    let nip05: String = nip05.into();
    let data: Vec<&str> = nip05.split('@').collect();
    if data.len() != 2 {
        return Err(Error::InvalidFormat);
    }
    let name: &str = data[0];
    let domain: &str = data[1];
    let url = format!("https://{domain}/.well-known/nostr.json?name={name}");
    Ok((url, name.to_string()))
}

fn get_key_from_json<S>(json: Value, name: S) -> Option<XOnlyPublicKey>
where
    S: Into<String>,
{
    let name: String = name.into();
    json.get("names")
        .and_then(|names| names.get(name))
        .and_then(|value| value.as_str())
        .and_then(|pubkey| XOnlyPublicKey::from_str(pubkey).ok())
}

fn get_relays_from_json(json: Value, pk: XOnlyPublicKey) -> Vec<String> {
    let relays_list: Option<Vec<String>> = json
        .get("relays")
        .and_then(|relays| relays.get(pk.to_string()))
        .and_then(|value| serde_json::from_value(value.clone()).ok());

    match relays_list {
        None => vec![],
        Some(v) => v,
    }
}

fn verify_json<S>(public_key: XOnlyPublicKey, json: Value, name: S) -> Result<(), Error>
where
    S: Into<String>,
{
    if let Some(pubkey) = get_key_from_json(json, name) {
        if pubkey == public_key {
            return Ok(());
        }
    }

    Err(Error::ImpossibleToVerify)
}

/// Verify NIP05
#[cfg(not(target_arch = "wasm32"))]
pub async fn verify<S>(
    public_key: XOnlyPublicKey,
    nip05: S,
    proxy: Option<SocketAddr>,
) -> Result<(), Error>
where
    S: Into<String>,
{
    use reqwest::Client;

    let (url, name) = compose_url(nip05)?;
    let mut builder = Client::builder();
    if let Some(proxy) = proxy {
        let proxy = format!("socks5h://{proxy}");
        builder = builder.proxy(Proxy::all(proxy)?);
    }
    let client: Client = builder.build()?;
    let res = client.get(url).send().await?;
    let json: Value = serde_json::from_str(&res.text().await?)?;
    verify_json(public_key, json, name)
}

/// Verify NIP05
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "blocking")]
pub fn verify_blocking<S>(
    public_key: XOnlyPublicKey,
    nip05: S,
    proxy: Option<SocketAddr>,
) -> Result<(), Error>
where
    S: Into<String>,
{
    use reqwest::blocking::Client;

    let (url, name) = compose_url(nip05)?;
    let mut builder = Client::builder();
    if let Some(proxy) = proxy {
        let proxy = format!("socks5h://{proxy}");
        builder = builder.proxy(Proxy::all(proxy)?);
    }
    let client: Client = builder.build()?;
    let res = client.get(url).send()?;
    let json: Value = serde_json::from_str(&res.text()?)?;
    verify_json(public_key, json, name)
}

/// Verify NIP05
#[cfg(target_arch = "wasm32")]
pub async fn verify<S>(public_key: XOnlyPublicKey, nip05: S) -> Result<(), Error>
where
    S: Into<String>,
{
    use reqwest::Client;

    let (url, name) = compose_url(nip05)?;
    let client: Client = Client::new();
    let res = client.get(url).send().await?;
    let json: Value = serde_json::from_str(&res.text().await?)?;
    verify_json(public_key, json, name)
}

/// Get [Profile] from NIP05 (public key and list of advertised relays)
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_profile<S>(nip05: S, proxy: Option<SocketAddr>) -> Result<Profile, Error>
where
    S: Into<String>,
{
    use reqwest::Client;

    let (url, name) = compose_url(nip05)?;
    let mut builder = Client::builder();
    if let Some(proxy) = proxy {
        let proxy = format!("socks5h://{proxy}");
        builder = builder.proxy(Proxy::all(proxy)?);
    }
    let client: Client = builder.build()?;
    let res = client.get(url).send().await?;
    let json: Value = serde_json::from_str(&res.text().await?)?;

    let public_key = get_key_from_json(json.clone(), name).ok_or(Error::ImpossibleToVerify)?;
    let relays = get_relays_from_json(json, public_key);

    Ok(Profile { public_key, relays })
}

/// Get [Profile] from NIP05 (public key and list of advertised relays)
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "blocking")]
pub fn get_profile_blocking<S>(nip05: S, proxy: Option<SocketAddr>) -> Result<Profile, Error>
where
    S: Into<String>,
{
    use reqwest::blocking::Client;

    let (url, name) = compose_url(nip05)?;
    let mut builder = Client::builder();
    if let Some(proxy) = proxy {
        let proxy = format!("socks5h://{proxy}");
        builder = builder.proxy(Proxy::all(proxy)?);
    }
    let client: Client = builder.build()?;
    let res = client.get(url).send()?;
    let json: Value = serde_json::from_str(&res.text()?)?;

    let public_key = get_key_from_json(json.clone(), name).ok_or(Error::ImpossibleToVerify)?;
    let relays = get_relays_from_json(json, public_key);

    Ok(Profile { public_key, relays })
}

/// Get [Profile] from NIP05 (public key and list of advertised relays)
#[cfg(target_arch = "wasm32")]
pub async fn get_profile<S>(nip05: S) -> Result<Profile, Error>
where
    S: Into<String>,
{
    use reqwest::Client;

    let (url, name) = compose_url(nip05)?;
    let client: Client = Client::new();
    let res = client.get(url).send().await?;
    let json: Value = serde_json::from_str(&res.text().await?)?;

    let public_key = get_key_from_json(json.clone(), name).ok_or(Error::ImpossibleToVerify)?;
    let relays = get_relays_from_json(json, public_key);

    Ok(Profile { public_key, relays })
}
