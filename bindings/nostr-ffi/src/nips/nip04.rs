// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::nips::nip04;
use nostr::secp256k1::{SecretKey, XOnlyPublicKey};

use crate::error::Result;

pub fn nip04_encrypt(secret_key: String, public_key: String, content: String) -> Result<String> {
    let sk = SecretKey::from_str(&secret_key)?;
    let pk = XOnlyPublicKey::from_str(&public_key)?;
    Ok(nip04::encrypt(&sk, &pk, content)?)
}

pub fn nip04_decrypt(
    secret_key: String,
    public_key: String,
    encrypted_content: String,
) -> Result<String> {
    let sk = SecretKey::from_str(&secret_key)?;
    let pk = XOnlyPublicKey::from_str(&public_key)?;
    Ok(nip04::decrypt(&sk, &pk, encrypted_content)?)
}
