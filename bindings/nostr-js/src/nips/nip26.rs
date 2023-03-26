// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::nips::nip26::{self, Conditions, DelegationTag, EventProperties};
use nostr::secp256k1::schnorr::Signature;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::{JsKeys, JsPublicKey};

/// Create a NIP-26 delegation tag (including the signature).
/// See also validate_delegation_tag().
#[wasm_bindgen(js_name = createDelegationTag)]
pub fn create_delegation_tag(
    delegator_keys: &JsKeys,
    delegatee_pubkey: &JsPublicKey,
    conditions: String,
) -> Result<String> {
    let conditions = Conditions::from_str(&conditions).map_err(into_err)?;
    let tag = DelegationTag::new(delegator_keys.deref(), delegatee_pubkey.into(), conditions)
        .map_err(into_err)?;
    Ok(tag.to_string())
}

/// Validate a NIP-26 delegation tag, check signature and conditions.
#[wasm_bindgen(js_name = validateDelegationTag)]
pub fn validate_delegation_tag(
    delegation_tag: String,
    delegatee_pubkey: &JsPublicKey,
    event_kind: u64,
    created_at: u64,
) -> bool {
    match DelegationTag::from_str(&delegation_tag) {
        Ok(tag) => {
            let event_properties = EventProperties::new(event_kind, created_at);
            tag.validate(delegatee_pubkey.into(), &event_properties)
                .is_ok()
        }
        Err(_) => false,
    }
}

/// Sign delegation (NIP26)
#[wasm_bindgen(js_name = signDelegation)]
pub fn sign_delegation(
    keys: &JsKeys,
    delegatee_pk: &JsPublicKey,
    conditions: String,
) -> Result<String> {
    let conditions = Conditions::from_str(&conditions).map_err(into_err)?;
    let signature: Signature =
        nip26::sign_delegation(keys.deref(), delegatee_pk.into(), conditions).map_err(into_err)?;
    Ok(signature.to_string())
}

/// Verify delegation signature (NIP26)
#[wasm_bindgen(js_name = verifyDelegationSignature)]
pub fn verify_delegation_signature(
    delegator_public_key: &JsPublicKey,
    delegatee_public_key: &JsPublicKey,
    conditions: String,
    signature: String,
) -> Result<bool> {
    let conditions = Conditions::from_str(&conditions).map_err(into_err)?;
    let signature_struct = Signature::from_str(&signature).map_err(into_err)?;
    match nip26::verify_delegation_signature(
        delegator_public_key.into(),
        signature_struct,
        delegatee_public_key.into(),
        conditions,
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
