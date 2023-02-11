// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP05
//!
//! https://github.com/nostr-protocol/nips/blob/master/05.md

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::str::FromStr;

use bitcoin::secp256k1::XOnlyPublicKey;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;
use serde_json::Value;

/// `NIP05` error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid format
    #[error("invalid format")]
    InvalidFormat,
    /// Impossible to verify
    #[error("impossible to verify")]
    ImpossibleToVerify,
    /// Reqwest error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Error serializing or deserializing JSON data
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Secp256k1 error
    #[error(transparent)]
    Secp256k1(#[from] bitcoin::secp256k1::Error),
}

fn compose_url(nip05: &str) -> Result<(String, &str), Error> {
    let data: Vec<&str> = nip05.split('@').collect();
    if data.len() != 2 {
        return Err(Error::InvalidFormat);
    }
    let name: &str = data[0];
    let domain: &str = data[1];
    let url = format!("https://{domain}/.well-known/nostr.json?name={name}");
    Ok((url, name))
}

fn get_key_from_json(json: Value, name: &str) -> Option<XOnlyPublicKey> {
    json.get("names")
        .and_then(|names| names.get(name))
        .and_then(|value| value.as_str())
        .and_then(|pubkey| XOnlyPublicKey::from_str(pubkey).ok())
}

fn verify_json(public_key: XOnlyPublicKey, json: Value, name: &str) -> Result<(), Error> {
    if let Some(pubkey) = get_key_from_json(json, name) {
        if pubkey == public_key {
            return Ok(());
        }
    }

    Err(Error::ImpossibleToVerify)
}

/// Verify NIP05
#[cfg(not(target_arch = "wasm32"))]
pub async fn verify(
    public_key: XOnlyPublicKey,
    nip05: &str,
    proxy: Option<SocketAddr>,
) -> Result<(), Error> {
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
pub fn verify_blocking(
    public_key: XOnlyPublicKey,
    nip05: &str,
    proxy: Option<SocketAddr>,
) -> Result<(), Error> {
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
pub async fn verify(public_key: XOnlyPublicKey, nip05: &str) -> Result<(), Error> {
    use reqwest::Client;

    let (url, name) = compose_url(nip05)?;
    let client: Client = Client::new();
    let res = client.get(url).send().await?;
    let json: Value = serde_json::from_str(&res.text().await?)?;
    verify_json(public_key, json, name)
}

/// Get public key from NIP05
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_key(nip05: &str, proxy: Option<SocketAddr>) -> Result<XOnlyPublicKey, Error> {
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

    match get_key_from_json(json, name) {
        Some(key) => Ok(key),
        None => Err(Error::ImpossibleToVerify),
    }
}

/// Get public key from NIP05
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "blocking")]
pub fn get_key_blocking(nip05: &str, proxy: Option<SocketAddr>) -> Result<XOnlyPublicKey, Error> {
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

    match get_key_from_json(json, name) {
        Some(key) => Ok(key),
        None => Err(Error::ImpossibleToVerify),
    }
}

/// Get public key from NIP05
#[cfg(target_arch = "wasm32")]
pub async fn get_key(nip05: &str) -> Result<XOnlyPublicKey, Error> {
    use reqwest::Client;

    let (url, name) = compose_url(nip05)?;
    let client: Client = Client::new();
    let res = client.get(url).send().await?;
    let json: Value = serde_json::from_str(&res.text().await?)?;

    match get_key_from_json(json, name) {
        Some(key) => Ok(key),
        None => Err(Error::ImpossibleToVerify),
    }
}
