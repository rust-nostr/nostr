// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::nips::nip26::{self, Conditions, DelegationTag, EventProperties};
use nostr::secp256k1::schnorr::Signature;

use crate::error::Result;
use crate::protocol::event::Kind;
use crate::protocol::key::{Keys, PublicKey};

/// Create a NIP26 delegation tag (including the signature).
/// See also validate_delegation_tag().
///
/// <https://github.com/nostr-protocol/nips/blob/master/26.md>
#[uniffi::export]
pub fn create_delegation_tag(
    delegator_keys: &Keys,
    delegatee_pubkey: &PublicKey,
    conditions: &str,
) -> Result<String> {
    let conditions = Conditions::from_str(conditions)?;
    let tag = DelegationTag::new(delegator_keys.deref(), delegatee_pubkey.deref(), conditions);
    Ok(tag.to_string())
}

/// Validate a NIP26 delegation tag, check signature and conditions.
///
/// <https://github.com/nostr-protocol/nips/blob/master/26.md>
#[uniffi::export]
pub fn validate_delegation_tag(
    delegation_tag: &str,
    delegatee_pubkey: &PublicKey,
    event_kind: &Kind,
    created_at: u64,
) -> bool {
    match DelegationTag::from_str(delegation_tag) {
        Ok(tag) => {
            let event_properties = EventProperties::new(event_kind.as_u16(), created_at);
            tag.validate(delegatee_pubkey.deref(), &event_properties)
                .is_ok()
        }
        Err(_) => false,
    }
}

/// Sign delegation.
/// See `create_delegation_tag` for more complete functionality.
///
/// <https://github.com/nostr-protocol/nips/blob/master/26.md>
#[uniffi::export]
pub fn sign_delegation(
    delegator_keys: &Keys,
    delegatee_pk: &PublicKey,
    conditions: &str,
) -> Result<String> {
    let conditions = Conditions::from_str(conditions)?;
    Ok(
        nip26::sign_delegation(delegator_keys.deref(), delegatee_pk.deref(), &conditions)
            .to_string(),
    )
}

/// Verify delegation signature
///
/// <https://github.com/nostr-protocol/nips/blob/master/26.md>
#[uniffi::export]
pub fn verify_delegation_signature(
    delegator_public_key: &PublicKey,
    delegatee_public_key: &PublicKey,
    conditions: &str,
    signature: &str,
) -> Result<bool> {
    let conditions = Conditions::from_str(conditions)?;
    let signature_struct = Signature::from_str(signature)?;
    Ok(nip26::verify_delegation_signature(
        delegator_public_key.deref(),
        signature_struct,
        delegatee_public_key.deref(),
        &conditions,
    )
    .is_ok())
}
