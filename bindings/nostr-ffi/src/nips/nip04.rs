// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip04;
use nostr::secp256k1::SecretKey;

use crate::error::Result;
use crate::PublicKey;

pub fn nip04_encrypt(
    secret_key: String,
    public_key: Arc<PublicKey>,
    content: String,
) -> Result<String> {
    let sk = SecretKey::from_str(&secret_key)?;
    Ok(nip04::encrypt(&sk, public_key.as_ref().deref(), content)?)
}

pub fn nip04_decrypt(
    secret_key: String,
    public_key: Arc<PublicKey>,
    encrypted_content: String,
) -> Result<String> {
    let sk = SecretKey::from_str(&secret_key)?;
    Ok(nip04::decrypt(
        &sk,
        public_key.as_ref().deref(),
        encrypted_content,
    )?)
}
