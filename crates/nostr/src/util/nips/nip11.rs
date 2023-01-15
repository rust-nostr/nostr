// Copyright (c) 2022 Thomas (0xtlt)
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

#[cfg(feature = "blocking")]
use reqwest::blocking::Client;
#[cfg(not(feature = "blocking"))]
use reqwest::Client;
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// The relay information document is invalid
    #[error("The relay information document is invalid")]
    InvalidInformationDocument,
    /// The relay information document is not accessible
    #[error("The relay information document is not accessible")]
    InaccessibleInformationDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayInformationDocument {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pubkey: String,
    pub contact: String,
    pub supported_nips: Vec<u16>,
    pub software: String,
    pub version: String,
}

/// Get Relay Information Document
#[cfg(not(feature = "blocking"))]
pub async fn get_relay_information_document(
    url: Url,
    proxy: Option<SocketAddr>,
) -> Result<RelayInformationDocument, Error> {
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
pub fn get_relay_information_document(
    url: Url,
    proxy: Option<SocketAddr>,
) -> Result<RelayInformationDocument, Error> {
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
