// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP26
//!
//! https://github.com/nostr-protocol/nips/blob/master/26.md

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
    secp.verify_schnorr(&signature, &message, &keys.public_key())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::{FromBech32, SecretKey};
    use std::str::FromStr;

    #[test]
    fn test_sign_delegation_verify_delegation_signature() {
        let sk = SecretKey::from_bech32(
            "nsec1ktekw0hr5evjs0n9nyyquz4sue568snypy2rwk5mpv6hl2hq3vtsk0kpae",
        )
        .unwrap();
        let keys = Keys::new(sk);
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1h652adkpv4lr8k66cadg8yg0wl5wcc29z4lyw66m3rrwskcl4v6qr82xez",
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
        let sk = SecretKey::from_bech32(
            "nsec1ktekw0hr5evjs0n9nyyquz4sue568snypy2rwk5mpv6hl2hq3vtsk0kpae",
        )
        .unwrap();
        let keys = Keys::new(sk);
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1h652adkpv4lr8k66cadg8yg0wl5wcc29z4lyw66m3rrwskcl4v6qr82xez",
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
        let sk = SecretKey::from_bech32(
            "nsec1ktekw0hr5evjs0n9nyyquz4sue568snypy2rwk5mpv6hl2hq3vtsk0kpae",
        )
        .unwrap();
        let keys = Keys::new(sk);
        // use one concrete signature
        let signature = Signature::from_str("05deed1262a42c832ae0acb42a2259ca9d82193e15aa6f0817dc9ccce08e107976778c60131783c6a2b4d48acd15b1c5bd2b06107af4a2bc657404a6077223b6").unwrap();
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1h652adkpv4lr8k66cadg8yg0wl5wcc29z4lyw66m3rrwskcl4v6qr82xez",
        )
        .unwrap();
        let conditions = "k=1".to_string();

        let verify_result =
            verify_delegation_signature(&keys, &signature, delegatee_pk, conditions);
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_delegation_token() {
        let delegatee_pk = XOnlyPublicKey::from_bech32(
            "npub1h652adkpv4lr8k66cadg8yg0wl5wcc29z4lyw66m3rrwskcl4v6qr82xez",
        )
        .unwrap();
        let conditions = "kind=1&created_at>1674834236&created_at<1677426236";
        let unhashed_token: String = delegation_token(&delegatee_pk, &conditions);
        assert_eq!(
            unhashed_token,
            "nostr:delegation:bea8aeb6c1657e33db5ac75a83910f77e8ec6145157e476b5b88c6e85b1fab34:kind=1&created_at>1674834236&created_at<1677426236"
        );
    }
}
