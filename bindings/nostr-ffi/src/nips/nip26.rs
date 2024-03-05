// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::nips::nip26::{self, Conditions, DelegationTag, EventProperties};
use nostr::secp256k1::schnorr::Signature;

use crate::error::Result;
use crate::{Keys, Kind, PublicKey};

/// Create a NIP-26 delegation tag (including the signature).
/// See also validate_delegation_tag().
#[uniffi::export]
pub fn create_delegation_tag(
    delegator_keys: &Keys,
    delegatee_pubkey: &PublicKey,
    conditions: String,
) -> Result<String> {
    let conditions = Conditions::from_str(&conditions)?;
    let tag = DelegationTag::new(delegator_keys.deref(), **delegatee_pubkey, conditions)?;
    Ok(tag.to_string())
}

/// Validate a NIP-26 delegation tag, check signature and conditions.
#[uniffi::export]
pub fn validate_delegation_tag(
    delegation_tag: String,
    delegatee_pubkey: &PublicKey,
    event_kind: &Kind,
    created_at: u64,
) -> bool {
    match DelegationTag::from_str(&delegation_tag) {
        Ok(tag) => {
            let event_properties = EventProperties::new(event_kind.as_u64(), created_at);
            tag.validate(**delegatee_pubkey, &event_properties).is_ok()
        }
        Err(_) => false,
    }
}

/// Sign delegation.
/// See `create_delegation_tag` for more complete functionality.
#[uniffi::export]
pub fn sign_delegation(
    delegator_keys: &Keys,
    delegatee_pk: &PublicKey,
    conditions: String,
) -> Result<String> {
    let conditions = Conditions::from_str(&conditions)?;
    Ok(nip26::sign_delegation(delegator_keys.deref(), **delegatee_pk, conditions)?.to_string())
}

/// Verify delegation signature (NIP26)
#[uniffi::export]
pub fn verify_delegation_signature(
    delegator_public_key: &PublicKey,
    delegatee_public_key: &PublicKey,
    conditions: String,
    signature: String,
) -> Result<bool> {
    let conditions = Conditions::from_str(&conditions)?;
    let signature_struct = Signature::from_str(&signature)?;
    Ok(nip26::verify_delegation_signature(
        **delegator_public_key,
        signature_struct,
        **delegatee_public_key,
        conditions,
    )
    .is_ok())
}
