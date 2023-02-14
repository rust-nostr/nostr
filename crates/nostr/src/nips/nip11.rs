// Copyright (c) 2022 Thomas (0xtlt)
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP11
//!
//! <https://github.com/nostr-protocol/nips/blob/master/11.md>

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use url::Url;

/// `NIP11` error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reqwest error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// The relay information document is invalid
    #[error("The relay information document is invalid")]
    InvalidInformationDocument,
    /// The relay information document is not accessible
    #[error("The relay information document is not accessible")]
    InaccessibleInformationDocument,
    /// Provided URL scheme is not valid
    #[error("Provided URL scheme is not valid")]
    InvalidScheme,
}

/// Relay information document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelayInformationDocument {
    /// Name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Owner public key
    pub pubkey: Option<String>,
    /// Owner contact
    pub contact: Option<String>,
    /// Supported NIPs
    pub supported_nips: Option<Vec<u16>>,
    /// Software
    pub software: Option<String>,
    /// Software version
    pub version: Option<String>,
}

impl RelayInformationDocument {
    /// Create new empty [`RelayInformationDocument`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get Relay Information Document
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get(url: Url, proxy: Option<SocketAddr>) -> Result<Self, Error> {
        use reqwest::Client;

        let mut builder = Client::builder();
        if let Some(proxy) = proxy {
            let proxy = format!("socks5h://{proxy}");
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        let client: Client = builder.build()?;
        let url = Self::with_http_scheme(url)?;
        let req = client.get(url).header("Accept", "application/nostr+json");
        match req.send().await {
            Ok(response) => match response.json().await {
                Ok(json) => Ok(json),
                Err(_) => Err(Error::InvalidInformationDocument),
            },
            Err(_) => Err(Error::InaccessibleInformationDocument),
        }
    }

    /// Get Relay Information Document
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "blocking")]
    pub fn get_blocking(url: Url, proxy: Option<SocketAddr>) -> Result<Self, Error> {
        use reqwest::blocking::Client;

        let mut builder = Client::builder();
        if let Some(proxy) = proxy {
            let proxy = format!("socks5h://{proxy}");
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        let client: Client = builder.build()?;
        let url = Self::with_http_scheme(url)?;
        let req = client.get(url).header("Accept", "application/nostr+json");
        match req.send() {
            Ok(response) => match response.json() {
                Ok(json) => Ok(json),
                Err(_) => Err(Error::InvalidInformationDocument),
            },
            Err(_) => Err(Error::InaccessibleInformationDocument),
        }
    }

    /// Get Relay Information Document
    #[cfg(target_arch = "wasm32")]
    pub async fn get(url: Url) -> Result<Self, Error> {
        use reqwest::Client;

        let client: Client = Client::new();
        let url = Self::with_http_scheme(url)?;
        let req = client.get(url).header("Accept", "application/nostr+json");
        match req.send().await {
            Ok(response) => match response.json().await {
                Ok(json) => Ok(json),
                Err(_) => Err(Error::InvalidInformationDocument),
            },
            Err(_) => Err(Error::InaccessibleInformationDocument),
        }
    }

    /// Returns new URL with scheme substituted to HTTP(S) if WS(S) was provided,
    /// other schemes leaves untouched.
    fn with_http_scheme(url: Url) -> Result<Url, Error> {
        let mut url = url;
        match url.scheme() {
            "wss" => url.set_scheme("https").map_err(|_| Error::InvalidScheme)?,
            "ws" => url.set_scheme("http").map_err(|_| Error::InvalidScheme)?,
            _ => {}
        }
        Ok(url)
    }
}
