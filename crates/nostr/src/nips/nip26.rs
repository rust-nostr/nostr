// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP26
//!
//! <https://github.com/nostr-protocol/nips/blob/master/26.md>

use bitcoin_hashes::sha256::Hash as Sha256Hash;
use bitcoin_hashes::Hash;
use secp256k1::schnorr::Signature;
use secp256k1::{KeyPair, Message, XOnlyPublicKey};

use crate::key::{self, Keys};
use crate::nips::nip19::ToBech32;
use crate::prelude::kind::Kind;
use crate::SECP256K1;

use core::fmt;

/// `NIP26` error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Key error
    #[error(transparent)]
    Key(#[from] key::Error),
    #[error(transparent)]
    /// Secp256k1 error
    Secp256k1(#[from] secp256k1::Error),
    #[error(transparent)]
    /// Signature error (NIP-19)
    SignatureError(#[from] crate::nips::nip19::Error),
}

fn delegation_token(delegatee_pk: &XOnlyPublicKey, conditions: &str) -> String {
    format!("nostr:delegation:{delegatee_pk}:{conditions}")
}

/// Sign delegation.
/// See `create_delegation_tag` for more complete functionality.
pub fn sign_delegation(
    delegator_keys: &Keys,
    delegatee_pk: XOnlyPublicKey,
    conditions: String,
) -> Result<Signature, Error> {
    let keypair: &KeyPair = &delegator_keys.key_pair()?;
    let unhashed_token: String = delegation_token(&delegatee_pk, &conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message = Message::from_slice(&hashed_token)?;
    Ok(SECP256K1.sign_schnorr(&message, keypair))
}

/// Verify delegation signature
pub fn verify_delegation_signature(
    delegator_pk: &XOnlyPublicKey,
    signature: &Signature,
    delegatee_pk: XOnlyPublicKey,
    conditions: String,
) -> Result<(), Error> {
    let unhashed_token: String = delegation_token(&delegatee_pk, &conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message = Message::from_slice(&hashed_token)?;
    SECP256K1.verify_schnorr(signature, &message, delegator_pk)?;
    Ok(())
}

/// Delegation tag, as defined in NIP-26
pub struct DelegationTag {
    delegator_pubkey: XOnlyPublicKey,
    conditions: String,
    signature: Signature,
}

impl DelegationTag {
    /// Accessor for delegator public key
    pub fn get_delegator_pubkey(&self) -> XOnlyPublicKey {
        self.delegator_pubkey
    }

    /// Accessor for conditions
    pub fn get_conditions(&self) -> String {
        self.conditions.clone()
    }

    /// Accessor for signature
    pub fn get_signature(&self) -> Signature {
        self.signature
    }

    // TODO from_string()

    /// Convert to JSON string
    pub(crate) fn to_json(&self, multiline: bool) -> Result<String, Error> {
        let delegator_npub = self.delegator_pubkey.to_bech32()?;
        let separator = if multiline { "\n" } else { " " };
        let tabulator = if multiline { "\t" } else { "" };
        Ok(format!(
            "[{}{}\"delegation\",{}{}\"{}\",{}{}\"{}\",{}{}\"{}\"{}]",
            separator,
            tabulator,
            separator,
            tabulator,
            delegator_npub,
            separator,
            tabulator,
            self.conditions,
            separator,
            tabulator,
            self.signature,
            separator
        ))
    }
}

impl fmt::Display for DelegationTag {
    /// Return tag in JSON string format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_json(false) {
            Err(e) => write!(f, "(error {e})"),
            Ok(s) => write!(f, "{s}"),
        }
    }
}

/// Create a delegation tag (including the signature)
pub fn create_delegation_tag(
    delegator_keys: &Keys,
    delegatee_pubkey: XOnlyPublicKey,
    conditions_string: &str,
) -> Result<DelegationTag, Error> {
    let signature = sign_delegation(
        delegator_keys,
        delegatee_pubkey,
        conditions_string.to_string(),
    )?;
    Ok(DelegationTag {
        delegator_pubkey: delegator_keys.public_key(),
        conditions: conditions_string.to_string(),
        signature,
    })
}

/// Verify a delegation tag, check signature and conditions.
pub fn verify_delegation_tag(
    delegation_tag: &DelegationTag,
    delegatee_pubkey: XOnlyPublicKey,
    _create_time: u64,
    _event_kind: Kind,
) -> Result<(), Error> {
    // verify signature
    verify_delegation_signature(
        &delegation_tag.get_delegator_pubkey(),
        &delegation_tag.get_signature(),
        delegatee_pubkey,
        delegation_tag.get_signature().to_string(),
    )?;

    // verify conditions
    // TODO verify conditions kind, created_at

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::{FromBech32, SecretKey};
    use std::str::FromStr;

    #[test]
    fn test_sign_delegation_verify_delegation_signature() {
        let delegator_secret_key =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        let delegatee_public_key = XOnlyPublicKey::from_bech32(
            "npub1gae33na4gfaeelrx48arwc2sc8wmccs3tt38emmjg9ltjktfzwtqtl4l6u",
        )
        .unwrap();
        let conditions = "kind=1&created_at>1674834236&created_at<1677426236".to_string();

        let signature =
            sign_delegation(&delegator_keys, delegatee_public_key, conditions.clone()).unwrap();

        // signature is changing, validate by verify method
        let verify_result = verify_delegation_signature(
            &delegator_keys.public_key(),
            &signature,
            delegatee_public_key,
            conditions,
        );
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_sign_delegation_verify_lowlevel() {
        let delegator_secret_key =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        let delegatee_public_key = XOnlyPublicKey::from_bech32(
            "npub1gae33na4gfaeelrx48arwc2sc8wmccs3tt38emmjg9ltjktfzwtqtl4l6u",
        )
        .unwrap();
        let conditions = "kind=1&created_at>1674834236&created_at<1677426236";

        let signature = sign_delegation(
            &delegator_keys,
            delegatee_public_key,
            conditions.to_string(),
        )
        .unwrap();

        // signature is changing, validate by lowlevel verify
        let unhashed_token: String =
            format!("nostr:delegation:{delegatee_public_key}:{conditions}");
        let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
        let message = Message::from_slice(&hashed_token).unwrap();

        let verify_result =
            SECP256K1.verify_schnorr(&signature, &message, &delegator_keys.public_key());
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_verify_delegation_signature() {
        let delegator_secret_key =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        // use one concrete signature
        let signature = Signature::from_str("f9f00fcf8480686d9da6dfde1187d4ba19c54f6ace4c73361a14db429c4b96eb30b29283d6ea1f06ba9e18e06e408244c689039ddadbacffc56060f3da5b04b8").unwrap();
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1gae33na4gfaeelrx48arwc2sc8wmccs3tt38emmjg9ltjktfzwtqtl4l6u",
        )
        .unwrap();
        let conditions = "kind=1&created_at>1674834236&created_at<1677426236".to_string();

        let verify_result = verify_delegation_signature(
            &delegator_keys.public_key(),
            &signature,
            delegatee_pk,
            conditions,
        );
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_delegation_token() {
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1gae33na4gfaeelrx48arwc2sc8wmccs3tt38emmjg9ltjktfzwtqtl4l6u",
        )
        .unwrap();
        let conditions = "kind=1&created_at>1674834236&created_at<1677426236";
        let unhashed_token: String = delegation_token(&delegatee_pk, &conditions);
        assert_eq!(
            unhashed_token,
            "nostr:delegation:477318cfb5427b9cfc66a9fa376150c1ddbc62115ae27cef72417eb959691396:kind=1&created_at>1674834236&created_at<1677426236"
        );
    }

    #[test]
    fn test_delegation_tag_to_json() {
        let delegator_sk = SecretKey::from_bech32(
            "nsec1ktekw0hr5evjs0n9nyyquz4sue568snypy2rwk5mpv6hl2hq3vtsk0kpae",
        )
        .unwrap();
        let delegator_pubkey = Keys::new(delegator_sk).public_key();
        let conditions = "k=1&reated_at<1678659553".to_string();
        let signature = Signature::from_str("435091ab4c4a11e594b1a05e0fa6c2f6e3b6eaa87c53f2981a3d6980858c40fdcaffde9a4c461f352a109402a4278ff4dbf90f9ebd05f96dac5ae36a6364a976").unwrap();
        let d = DelegationTag {
            delegator_pubkey,
            conditions,
            signature,
        };
        let tag = d.to_json(false).unwrap();
        assert_eq!(tag, "[ \"delegation\", \"npub1rfze4zn25ezp6jqt5ejlhrajrfx0az72ed7cwvq0spr22k9rlnjq93lmd4\", \"k=1&reated_at<1678659553\", \"435091ab4c4a11e594b1a05e0fa6c2f6e3b6eaa87c53f2981a3d6980858c40fdcaffde9a4c461f352a109402a4278ff4dbf90f9ebd05f96dac5ae36a6364a976\" ]");
        let tag2 = d.to_json(true).unwrap();
        assert_eq!(tag2, "[\n\t\"delegation\",\n\t\"npub1rfze4zn25ezp6jqt5ejlhrajrfx0az72ed7cwvq0spr22k9rlnjq93lmd4\",\n\t\"k=1&reated_at<1678659553\",\n\t\"435091ab4c4a11e594b1a05e0fa6c2f6e3b6eaa87c53f2981a3d6980858c40fdcaffde9a4c461f352a109402a4278ff4dbf90f9ebd05f96dac5ae36a6364a976\"\n]");
    }

    #[test]
    fn test_create_delegation_tag() {
        let delegator_secret_key = SecretKey::from_bech32(
            "nsec1ktekw0hr5evjs0n9nyyquz4sue568snypy2rwk5mpv6hl2hq3vtsk0kpae",
        )
        .unwrap();
        let delegator_keys = Keys::new(delegator_secret_key);
        let delegatee_pubkey = XOnlyPublicKey::from_bech32(
            "npub1h652adkpv4lr8k66cadg8yg0wl5wcc29z4lyw66m3rrwskcl4v6qr82xez",
        )
        .unwrap();
        let conditions = "k=1&created_at>1676067553&created_at<1678659553".to_string();

        let tag = create_delegation_tag(&delegator_keys, delegatee_pubkey, &conditions).unwrap();

        // verify signature (it's variable)
        let verify_result = verify_delegation_signature(
            &delegator_keys.public_key(),
            &tag.get_signature(),
            delegatee_pubkey,
            conditions,
        );
        assert!(verify_result.is_ok());

        // signature changes, cannot compare to expected constant, use signature from result
        let expected = format!(
            "[ \"delegation\", \"npub1rfze4zn25ezp6jqt5ejlhrajrfx0az72ed7cwvq0spr22k9rlnjq93lmd4\", \"k=1&created_at>1676067553&created_at<1678659553\", \"{}\" ]",
            &tag.signature.to_string());
        assert_eq!(tag.to_string(), expected);

        assert_eq!(tag.to_json(false).unwrap(), expected);
        let expected_multiline = format!(
            "[\n\t\"delegation\",\n\t\"npub1rfze4zn25ezp6jqt5ejlhrajrfx0az72ed7cwvq0spr22k9rlnjq93lmd4\",\n\t\"k=1&created_at>1676067553&created_at<1678659553\",\n\t\"{}\"\n]",
            &tag.signature.to_string());
        assert_eq!(tag.to_json(true).unwrap(), expected_multiline);
    }
}
