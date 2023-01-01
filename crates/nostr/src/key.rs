// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use bitcoin::secp256k1::rand::rngs::OsRng;
pub use bitcoin::secp256k1::{KeyPair, Secp256k1, SecretKey, XOnlyPublicKey};

use crate::util::nips::nip19::FromBech32;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Invalid secret key")]
    InvalidSecretKey,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Secrete key missing")]
    SkMissing,
    #[error("Key pair missing")]
    KeyPairMissing,
    #[error("Failed to generate new keys")]
    KeyGenerationFailure,
    /// Secp256k1 error
    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] bitcoin::secp256k1::Error),
}

pub trait FromSkStr: Sized {
    type Err;
    fn from_sk_str(secret_key: &str) -> Result<Self, Self::Err>;
}

pub trait FromPkStr: Sized {
    type Err;
    fn from_pk_str(public_key: &str) -> Result<Self, Self::Err>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Keys {
    public_key: XOnlyPublicKey,
    key_pair: Option<KeyPair>,
    secret_key: Option<SecretKey>,
}

impl Keys {
    /// Initialize from secret key.
    pub fn new(secret_key: SecretKey) -> Self {
        let secp = Secp256k1::new();
        let key_pair = KeyPair::from_secret_key(&secp, &secret_key);
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

    #[deprecated(since = "0.11.0")]
    pub fn from_bech32<S>(secret_key: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let secret_key = SecretKey::from_bech32(secret_key).map_err(|_| Error::InvalidSecretKey)?;
        Ok(Self::new(secret_key))
    }

    #[deprecated(since = "0.11.0")]
    pub fn from_bech32_public_key<S>(public_key: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let public_key =
            XOnlyPublicKey::from_bech32(public_key).map_err(|_| Error::InvalidPublicKey)?;
        Ok(Self::from_public_key(public_key))
    }

    /// Generate a new random keys
    pub fn generate_from_os_random() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng::default();
        let (secret_key, _) = secp.generate_keypair(&mut rng);
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
