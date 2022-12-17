// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin::secp256k1::XOnlyPublicKey;
use reqwest::blocking::Client;
use serde_json::Value;

/// Verify NIP-05
pub fn verify(public_key: XOnlyPublicKey, nip05: &str) -> Result<bool> {
    let data: Vec<&str> = nip05.split('@').collect();
    if data.len() != 2 {
        return Err(anyhow!("Invalid NIP-05"));
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
                return Ok(XOnlyPublicKey::from_str(pubkey)? == public_key);
            }
        }
    }

    Ok(false)
}
