// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip04;

use crate::error::Result;
use crate::{PublicKey, SecretKey};

pub fn nip04_encrypt(
    secret_key: Arc<SecretKey>,
    public_key: Arc<PublicKey>,
    content: String,
) -> Result<String> {
    Ok(nip04::encrypt(
        secret_key.as_ref().deref(),
        public_key.as_ref().deref(),
        content,
    )?)
}

pub fn nip04_decrypt(
    secret_key: Arc<SecretKey>,
    public_key: Arc<PublicKey>,
    encrypted_content: String,
) -> Result<String> {
    Ok(nip04::decrypt(
        secret_key.as_ref().deref(),
        public_key.as_ref().deref(),
        encrypted_content,
    )?)
}

pub fn generate_shared_key(
    secret_key: Arc<SecretKey>,
    public_key: Arc<PublicKey>,
) -> Result<Vec<u8>> {
    let shared_key: [u8; 32] =
        nip04::generate_shared_key(secret_key.as_ref().deref(), public_key.as_ref().deref())?;
    Ok(shared_key.to_vec())
}
