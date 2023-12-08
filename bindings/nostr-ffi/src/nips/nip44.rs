// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip44::{self, Version};

use crate::error::Result;
use crate::{PublicKey, SecretKey};

#[uniffi::export]
pub fn nip44_encrypt(
    secret_key: Arc<SecretKey>,
    public_key: Arc<PublicKey>,
    content: String,
    version: Version,
) -> Result<String> {
    Ok(nip44::encrypt(
        secret_key.as_ref().deref(),
        public_key.as_ref().deref(),
        content,
        version,
    )?)
}

#[uniffi::export]
pub fn nip44_decrypt(
    secret_key: Arc<SecretKey>,
    public_key: Arc<PublicKey>,
    payload: String,
) -> Result<String> {
    Ok(nip44::decrypt(
        secret_key.as_ref().deref(),
        public_key.as_ref().deref(),
        payload,
    )?)
}
