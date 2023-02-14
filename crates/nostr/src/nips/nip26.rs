// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP26
//!
//! <https://github.com/nostr-protocol/nips/blob/master/26.md>

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{KeyPair, Message, Secp256k1, XOnlyPublicKey};

use crate::key::{self, Keys};

/// `NIP26` error
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    /// Key error
    #[error(transparent)]
    Key(#[from] key::Error),
    #[error(transparent)]
    /// Secp256k1 error
    Secp256k1(#[from] bitcoin::secp256k1::Error),
}

fn delegation_token(delegatee_pk: &XOnlyPublicKey, conditions: &str) -> String {
    format!("nostr:delegation:{delegatee_pk}:{conditions}")
}

/// Sign delegation
pub fn sign_delegation(
    keys: &Keys,
    delegatee_pk: XOnlyPublicKey,
    conditions: String,
) -> Result<Signature, Error> {
    let secp = Secp256k1::new();
    let keypair: &KeyPair = &keys.key_pair()?;
    let unhashed_token: String = delegation_token(&delegatee_pk, &conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message = Message::from_slice(&hashed_token)?;
    Ok(secp.sign_schnorr(&message, keypair))
}

/// Verify delegation signature
pub fn verify_delegation_signature(
    keys: &Keys,
    signature: &Signature,
    delegatee_pk: XOnlyPublicKey,
    conditions: String,
) -> Result<(), Error> {
    let secp = Secp256k1::new();
    let unhashed_token: String = delegation_token(&delegatee_pk, &conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message = Message::from_slice(&hashed_token)?;
    secp.verify_schnorr(signature, &message, &keys.public_key())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::{FromBech32, SecretKey};
    use std::str::FromStr;

    #[test]
    fn test_sign_delegation_verify_delegation_signature() {
        let sk =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let keys = Keys::new(sk);
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1gae33na4gfaeelrx48arwc2sc8wmccs3tt38emmjg9ltjktfzwtqtl4l6u",
        )
        .unwrap();
        let conditions = "kind=1&created_at>1674834236&created_at<1677426236".to_string();

        let signature = sign_delegation(&keys, delegatee_pk, conditions.clone()).unwrap();

        // signature is changing, validate by verify method
        let verify_result =
            verify_delegation_signature(&keys, &signature, delegatee_pk, conditions);
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_sign_delegation_verify_lowlevel() {
        let sk =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let keys = Keys::new(sk);
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1gae33na4gfaeelrx48arwc2sc8wmccs3tt38emmjg9ltjktfzwtqtl4l6u",
        )
        .unwrap();
        let conditions = "kind=1&created_at>1674834236&created_at<1677426236";

        let signature = sign_delegation(&keys, delegatee_pk, conditions.to_string()).unwrap();

        // signature is changing, validate by lowlevel verify
        let unhashed_token: String = format!("nostr:delegation:{delegatee_pk}:{conditions}");
        let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
        let message = Message::from_slice(&hashed_token).unwrap();
        let secp = Secp256k1::new();
        let verify_result = secp.verify_schnorr(&signature, &message, &keys.public_key());
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_verify_delegation_signature() {
        let sk =
            SecretKey::from_str("ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c")
                .unwrap();
        let keys = Keys::new(sk);
        // use one concrete signature
        let signature = Signature::from_str("f9f00fcf8480686d9da6dfde1187d4ba19c54f6ace4c73361a14db429c4b96eb30b29283d6ea1f06ba9e18e06e408244c689039ddadbacffc56060f3da5b04b8").unwrap();
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1gae33na4gfaeelrx48arwc2sc8wmccs3tt38emmjg9ltjktfzwtqtl4l6u",
        )
        .unwrap();
        let conditions = "kind=1&created_at>1674834236&created_at<1677426236".to_string();

        let verify_result =
            verify_delegation_signature(&keys, &signature, delegatee_pk, conditions);
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
}
