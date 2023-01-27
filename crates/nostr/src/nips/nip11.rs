// Copyright (c) 2022 Thomas (0xtlt)
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP11
//!
//! https://github.com/nostr-protocol/nips/blob/master/11.md

use std::net::SocketAddr;

#[cfg(feature = "blocking")]
use reqwest::blocking::Client;
#[cfg(not(feature = "blocking"))]
use reqwest::Client;
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
}

/// Relay information document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelayInformationDocument {
    ///
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Owner public key
    pub pubkey: String,
    /// Owner contact
    pub contact: String,
    /// Supported NIPs
    pub supported_nips: Vec<u16>,
    /// Software
    pub software: String,
    /// Software version
    pub version: String,
}

impl RelayInformationDocument {
    /// Create new empty [`RelayInformationDocument`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get Relay Information Document
    #[cfg(not(feature = "blocking"))]
    pub async fn get(url: Url, proxy: Option<SocketAddr>) -> Result<Self, Error> {
        let mut builder = Client::builder();
        if let Some(proxy) = proxy {
            let proxy = format!("socks5h://{}", proxy);
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        let client: Client = builder.build()?;
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
    #[cfg(feature = "blocking")]
    pub fn get(url: Url, proxy: Option<SocketAddr>) -> Result<Self, Error> {
        let mut builder = Client::builder();
        if let Some(proxy) = proxy {
            let proxy = format!("socks5h://{}", proxy);
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        let client: Client = builder.build()?;
        let req = client.get(url).header("Accept", "application/nostr+json");
        match req.send() {
            Ok(response) => match response.json() {
                Ok(json) => Ok(json),
                Err(_) => Err(Error::InvalidInformationDocument),
            },
            Err(_) => Err(Error::InaccessibleInformationDocument),
        }
    }
}

/// Get Relay Information Document
#[deprecated]
#[cfg(not(feature = "blocking"))]
pub async fn get_relay_information_document(
    url: Url,
    proxy: Option<SocketAddr>,
) -> Result<RelayInformationDocument, Error> {
    RelayInformationDocument::get(url, proxy).await
}

/// Get Relay Information Document
#[cfg(feature = "blocking")]
#[deprecated]
pub fn get_relay_information_document(
    url: Url,
    proxy: Option<SocketAddr>,
) -> Result<RelayInformationDocument, Error> {
    RelayInformationDocument::get(url, proxy)
}
