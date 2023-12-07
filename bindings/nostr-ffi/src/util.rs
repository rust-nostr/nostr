// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::util;

use crate::error::Result;
use crate::{PublicKey, SecretKey};

#[uniffi::export]
pub fn generate_shared_key(
    secret_key: Arc<SecretKey>,
    public_key: Arc<PublicKey>,
) -> Result<Vec<u8>> {
    let shared_key: [u8; 32] =
        util::generate_shared_key(secret_key.as_ref().deref(), public_key.as_ref().deref())?;
    Ok(shared_key.to_vec())
}
