// Copyright (c) 2022 Thomas (0xtlt)
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::SocketAddr;

use reqwest::blocking::Client;
use reqwest::Proxy;
use url::Url;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    /// The relay information document is invalid
    InvalidInformationDocument,
    /// The relay information document is not accessible
    InaccessibleInformationDocument,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reqwest(err) => write!(f, "reqwest error: {}", err),
            Self::InvalidInformationDocument => {
                write!(f, "The relay information document is invalid")
            }
            Self::InaccessibleInformationDocument => {
                write!(f, "The relay information document is not accessible")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
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
