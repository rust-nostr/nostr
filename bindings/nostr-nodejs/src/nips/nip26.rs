// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use napi::Result;
use nostr::nips::nip26;
use nostr::secp256k1::schnorr::Signature;

use crate::error::into_err;
use crate::{JsKeys, JsPublicKey};

/// Sign delegation (NIP26)
#[napi]
pub fn sign_delegation(
    keys: &JsKeys,
    delegatee_pk: &JsPublicKey,
    conditions: String,
) -> Result<String> {
    Ok(
        nip26::sign_delegation(keys.deref(), delegatee_pk.into(), conditions)
            .map_err(into_err)?
            .to_string(),
    )
}

/// Verify delegation signature (NIP26)
#[napi]
pub fn verify_delegation_signature(
    keys: &JsKeys,
    delegatee_pk: &JsPublicKey,
    conditions: String,
    signature: String,
) -> Result<bool> {
    let signature = Signature::from_str(&signature).map_err(into_err)?;
    match nip26::verify_delegation_signature(
        keys.deref(),
        &signature,
        delegatee_pk.into(),
        conditions,
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
