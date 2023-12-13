// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::util;

use crate::{PublicKey, SecretKey};

#[uniffi::export]
pub fn generate_shared_key(secret_key: Arc<SecretKey>, public_key: Arc<PublicKey>) -> Vec<u8> {
    util::generate_shared_key(secret_key.as_ref().deref(), public_key.as_ref().deref()).to_vec()
}
