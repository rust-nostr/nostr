// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP05
//!
//! <https://github.com/nostr-protocol/nips/blob/master/05.md>

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::str::FromStr;

#[cfg(feature = "base")]
use bitcoin_hashes::hex::ToHex;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;
use secp256k1::XOnlyPublicKey;
use serde_json::Value;

#[cfg(feature = "base")]
use crate::Profile;

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
    Secp256k1(#[from] secp256k1::Error),
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

#[cfg(feature = "base")]
fn get_relays_from_json(json: Value, pk: XOnlyPublicKey) -> Vec<String> {
    let relays_list: Option<Vec<String>> = json
        .get("relays")
        .and_then(|relays| relays.get(pk.to_hex()))
        .and_then(|value| serde_json::from_value(value.clone()).ok());

    match relays_list {
        None => vec![],
        Some(v) => v,
    }
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

/// Get [Profile] from NIP05 (public key and list of advertised relays)
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "base")]
pub async fn get_profile(nip05: &str, proxy: Option<SocketAddr>) -> Result<Profile, Error> {
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
#[cfg(all(feature = "blocking", feature = "base"))]
pub fn get_profile_blocking(nip05: &str, proxy: Option<SocketAddr>) -> Result<Profile, Error> {
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
#[cfg(feature = "base")]
pub async fn get_profile(nip05: &str) -> Result<Profile, Error> {
    use reqwest::Client;

    let (url, name) = compose_url(nip05)?;
    let client: Client = Client::new();
    let res = client.get(url).send().await?;
    let json: Value = serde_json::from_str(&res.text().await?)?;

    let public_key = get_key_from_json(json.clone(), name).ok_or(Error::ImpossibleToVerify)?;
    let relays = get_relays_from_json(json, public_key);

    Ok(Profile { public_key, relays })
}
