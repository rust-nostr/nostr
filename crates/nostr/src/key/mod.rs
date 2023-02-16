// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Keys
//!
//! This module defines the [`Keys`] structure.

#[cfg(feature = "nip19")]
use std::str::FromStr;

use secp256k1::rand::rngs::OsRng;
pub use secp256k1::{KeyPair, SecretKey, XOnlyPublicKey};

use crate::SECP256K1;

#[cfg(feature = "vanity")]
pub mod vanity;

#[cfg(feature = "nip19")]
use crate::nips::nip19::FromBech32;

/// [`Keys`] error
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    /// Invalid secret key
    #[error("Invalid secret key")]
    InvalidSecretKey,
    /// Invalid public key
    #[error("Invalid public key")]
    InvalidPublicKey,
    /// Secrete key missing
    #[error("Secrete key missing")]
    SkMissing,
    /// Key pair missing
    #[error("Key pair missing")]
    KeyPairMissing,
    /// Failed to generate new keys
    #[error("Failed to generate new keys")]
    KeyGenerationFailure,
    /// Unsupported char
    #[error("Unsupported char: {0}")]
    InvalidChar(char),
    /// Secp256k1 error
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
}

/// Trait for [`Keys`]
pub trait FromSkStr: Sized {
    /// Error
    type Err;
    /// Init [`Keys`] from `hex` or `bech32` secret key string
    fn from_sk_str(secret_key: &str) -> Result<Self, Self::Err>;
}

/// Trait for [`Keys`]
pub trait FromPkStr: Sized {
    /// Error
    type Err;
    /// Init [`Keys`] from `hex` or `bech32` public key string
    fn from_pk_str(public_key: &str) -> Result<Self, Self::Err>;
}

/// Keys
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Keys {
    public_key: XOnlyPublicKey,
    key_pair: Option<KeyPair>,
    secret_key: Option<SecretKey>,
}

impl Keys {
    /// Initialize from secret key.
    pub fn new(secret_key: SecretKey) -> Self {
        let key_pair = KeyPair::from_secret_key(SECP256K1, &secret_key);
        let public_key = XOnlyPublicKey::from_keypair(&key_pair).0;

        Self {
            public_key,
            key_pair: Some(key_pair),
            secret_key: Some(secret_key),
        }
    }

    /// Initialize with public key only (no secret key).
    pub fn from_public_key(public_key: XOnlyPublicKey) -> Self {
        Self {
            public_key,
            key_pair: None,
            secret_key: None,
        }
    }

    /// Generate new random keys
    pub fn generate() -> Self {
        let mut rng = OsRng::default();
        let (secret_key, _) = SECP256K1.generate_keypair(&mut rng);
        Self::new(secret_key)
    }

    /// Get public key
    pub fn public_key(&self) -> XOnlyPublicKey {
        self.public_key
    }

    /// Get secret key
    pub fn secret_key(&self) -> Result<SecretKey, Error> {
        if let Some(secret_key) = self.secret_key {
            Ok(secret_key)
        } else {
            Err(Error::SkMissing)
        }
    }

    /// Get keypair
    pub fn key_pair(&self) -> Result<KeyPair, Error> {
        if let Some(key_pair) = self.key_pair {
            Ok(key_pair)
        } else {
            Err(Error::KeyPairMissing)
        }
    }
}

#[cfg(feature = "nip19")]
impl FromSkStr for Keys {
    type Err = Error;

    /// Init [`Keys`] from `hex` or `bech32` secret key
    fn from_sk_str(secret_key: &str) -> Result<Self, Self::Err> {
        match SecretKey::from_str(secret_key) {
            Ok(secret_key) => Ok(Self::new(secret_key)),
            Err(_) => match SecretKey::from_bech32(secret_key) {
                Ok(secret_key) => Ok(Self::new(secret_key)),
                Err(_) => Err(Error::InvalidSecretKey),
            },
        }
    }
}

#[cfg(feature = "nip19")]
impl FromPkStr for Keys {
    type Err = Error;

    /// Init [`Keys`] from `hex` or `bech32` public key
    fn from_pk_str(public_key: &str) -> Result<Self, Self::Err> {
        match XOnlyPublicKey::from_str(public_key) {
            Ok(public_key) => Ok(Self::from_public_key(public_key)),
            Err(_) => match XOnlyPublicKey::from_bech32(public_key) {
                Ok(public_key) => Ok(Self::from_public_key(public_key)),
                Err(_) => Err(Error::InvalidSecretKey),
            },
        }
    }
}
