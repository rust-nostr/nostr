// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr_sdk::prelude::*;
use nostr_sdk::secp256k1::schnorr::Signature;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::event::JsKind;
use crate::protocol::key::{JsKeys, JsPublicKey};
use crate::protocol::types::JsTimestamp;

/// Create a NIP26 delegation tag (including the signature).
/// See also validate_delegation_tag().
///
/// <https://github.com/nostr-protocol/nips/blob/master/26.md>
#[wasm_bindgen(js_name = createDelegationTag)]
pub fn create_delegation_tag(
    delegator_keys: &JsKeys,
    delegatee_pubkey: &JsPublicKey,
    conditions: &str,
) -> Result<String> {
    let conditions = Conditions::from_str(conditions).map_err(into_err)?;
    let tag = DelegationTag::new(delegator_keys.deref(), delegatee_pubkey.deref(), conditions);
    Ok(tag.to_string())
}

/// Validate a NIP26 delegation tag, check signature and conditions.
///
/// <https://github.com/nostr-protocol/nips/blob/master/26.md>
#[wasm_bindgen(js_name = validateDelegationTag)]
pub fn validate_delegation_tag(
    delegation_tag: &str,
    delegatee_pubkey: &JsPublicKey,
    kind: &JsKind,
    created_at: &JsTimestamp,
) -> bool {
    match DelegationTag::from_str(delegation_tag) {
        Ok(tag) => {
            let event_properties = EventProperties::new(kind.as_u16(), created_at.as_u64());
            tag.validate(delegatee_pubkey.deref(), &event_properties)
                .is_ok()
        }
        Err(_) => false,
    }
}

/// Sign delegation
///
/// <https://github.com/nostr-protocol/nips/blob/master/26.md>
#[wasm_bindgen(js_name = signDelegation)]
pub fn sign_delegation(
    keys: &JsKeys,
    delegatee_pk: &JsPublicKey,
    conditions: &str,
) -> Result<String> {
    let conditions = Conditions::from_str(conditions).map_err(into_err)?;
    let signature: Signature =
        nip26::sign_delegation(keys.deref(), delegatee_pk.deref(), &conditions);
    Ok(signature.to_string())
}

/// Verify delegation signature
///
/// <https://github.com/nostr-protocol/nips/blob/master/26.md>
#[wasm_bindgen(js_name = verifyDelegationSignature)]
pub fn verify_delegation_signature(
    delegator_public_key: &JsPublicKey,
    delegatee_public_key: &JsPublicKey,
    conditions: &str,
    signature: &str,
) -> Result<bool> {
    let conditions = Conditions::from_str(conditions).map_err(into_err)?;
    let signature_struct = Signature::from_str(signature).map_err(into_err)?;
    match nip26::verify_delegation_signature(
        delegator_public_key.deref(),
        signature_struct,
        delegatee_public_key.deref(),
        &conditions,
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
