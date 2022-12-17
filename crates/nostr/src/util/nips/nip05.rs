// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use bitcoin::secp256k1::XOnlyPublicKey;
use reqwest::blocking::Client;
use serde_json::Value;

#[derive(Debug)]
pub enum Error {
    InvalidFormat,
    ImpossibleToVerify,
    Reqwest(reqwest::Error),
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
    Secp256k1(bitcoin::secp256k1::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid format"),
            Self::ImpossibleToVerify => write!(f, "impossible to verify"),
            Self::Reqwest(err) => write!(f, "reqwest error: {}", err),
            Self::Json(err) => write!(f, "JSON error: {}", err),
            Self::Secp256k1(err) => write!(f, "secp256k1 error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<bitcoin::secp256k1::Error> for Error {
    fn from(err: bitcoin::secp256k1::Error) -> Self {
        Self::Secp256k1(err)
    }
}

/// Verify NIP-05
pub fn verify(public_key: XOnlyPublicKey, nip05: &str) -> Result<(), Error> {
    let data: Vec<&str> = nip05.split('@').collect();
    if data.len() != 2 {
        return Err(Error::InvalidFormat);
    }

    let name: &str = data[0];
    let domain: &str = data[1];

    let url = format!("https://{}/.well-known/nostr.json?name={}", domain, name);

    let req = Client::new().get(url);

    let res = req.send()?;
    let json: Value = serde_json::from_str(&res.text()?)?;

    if let Some(names) = json.get("names") {
        if let Some(value) = names.get(name) {
            if let Some(pubkey) = value.as_str() {
                if XOnlyPublicKey::from_str(pubkey)? == public_key {
                    return Ok(());
                }
            }
        }
    }

    Err(Error::ImpossibleToVerify)
}
