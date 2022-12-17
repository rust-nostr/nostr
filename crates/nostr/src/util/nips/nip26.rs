// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;

use bitcoin::hashes::Hash;
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{KeyPair, Message, Secp256k1, XOnlyPublicKey};

use crate::key::{self, Keys};
use crate::Sha256Hash;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Key error
    Key(key::Error),
    Secp256k1(bitcoin::secp256k1::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(err) => write!(f, "key error: {}", err),
            Self::Secp256k1(err) => write!(f, "secp256k1 error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<key::Error> for Error {
    fn from(err: key::Error) -> Self {
        Self::Key(err)
    }
}

impl From<bitcoin::secp256k1::Error> for Error {
    fn from(err: bitcoin::secp256k1::Error) -> Self {
        Self::Secp256k1(err)
    }
}

pub fn sign_delegation(
    keys: &Keys,
    delegatee_pk: XOnlyPublicKey,
    conditions: String,
) -> Result<Signature, Error> {
    let secp = Secp256k1::new();
    let keypair: &KeyPair = &keys.key_pair()?;
    let unhashed_token: String = format!("nostr:delegation:{}:{}", delegatee_pk, conditions);
    let hashed_token = Sha256Hash::hash(unhashed_token.as_bytes());
    let message = Message::from_slice(&hashed_token)?;
    Ok(secp.sign_schnorr(&message, keypair))
}
