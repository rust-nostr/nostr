// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip44::{self, Version};
use uniffi::Enum;

use crate::error::Result;
use crate::protocol::key::{PublicKey, SecretKey};

/// NIP44 Version
#[derive(Enum)]
pub enum Nip44Version {
    /// V2 - Secp256k1 ECDH, HKDF, padding, ChaCha20, HMAC-SHA256 and base64
    V2,
}

impl From<Nip44Version> for Version {
    fn from(version: Nip44Version) -> Self {
        match version {
            Nip44Version::V2 => Self::V2,
        }
    }
}

#[uniffi::export]
pub fn nip44_encrypt(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    content: String,
    version: Nip44Version,
) -> Result<String> {
    Ok(nip44::encrypt(
        secret_key.deref(),
        public_key.deref(),
        content,
        version.into(),
    )?)
}

#[uniffi::export]
pub fn nip44_decrypt(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    payload: String,
) -> Result<String> {
    Ok(nip44::decrypt(
        secret_key.deref(),
        public_key.deref(),
        payload,
    )?)
}
