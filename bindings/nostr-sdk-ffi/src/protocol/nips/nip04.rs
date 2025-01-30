// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip04;

use crate::error::Result;
use crate::protocol::key::{PublicKey, SecretKey};

#[uniffi::export]
pub fn nip04_encrypt(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    content: String,
) -> Result<String> {
    Ok(nip04::encrypt(
        secret_key.deref(),
        public_key.deref(),
        content,
    )?)
}

#[uniffi::export]
pub fn nip04_decrypt(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    encrypted_content: String,
) -> Result<String> {
    Ok(nip04::decrypt(
        secret_key.deref(),
        public_key.deref(),
        encrypted_content,
    )?)
}
