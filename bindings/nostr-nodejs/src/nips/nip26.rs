// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use napi::Result;
use nostr::nips::nip26;
use nostr::secp256k1::schnorr::Signature;

use crate::error::into_err;
use crate::{JsKeys, JsPublicKey};

/// Create a NIP-26 delegation tag (including the signature).
/// See also validate_delegation_tag().
#[napi]
pub fn create_delegation_tag(
    delegator_keys: &JsKeys,
    delegatee_pubkey: &JsPublicKey,
    conditions: String,
) -> Result<String> {
    match nip26::create_delegation_tag(delegator_keys.deref(), delegatee_pubkey.into(), &conditions)
    {
        Ok(tag) => Ok(tag.to_string()),
        Err(_) => Ok("".to_string()),
    }
}

/// Validate a NIP-26 delegation tag, check signature and conditions.
pub fn validate_delegation_tag(
    delegation_tag: String,
    delegatee_pubkey: &JsPublicKey,
    event_kind: u64,
    created_at: u64,
) -> Result<bool> {
    match nip26::DelegationTag::from_str(&delegation_tag) {
        Err(_) => Ok(false),
        Ok(tag) => {
            let event_properties = nip26::EventProperties::new(event_kind, created_at);
            match nip26::validate_delegation_tag(&tag, delegatee_pubkey.into(), &event_properties) {
                Err(_) => Ok(false),
                Ok(_) => Ok(true),
            }
        }
    }
}

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
    delegator_public_key: &JsPublicKey,
    delegatee_public_key: &JsPublicKey,
    conditions: String,
    signature: String,
) -> Result<bool> {
    let signature_struct = Signature::from_str(&signature).map_err(into_err)?;
    match nip26::verify_delegation_signature(
        delegator_public_key.deref(),
        &signature_struct,
        delegatee_public_key.into(),
        conditions,
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
