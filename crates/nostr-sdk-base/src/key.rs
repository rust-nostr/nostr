// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use bech32::{self, FromBase32, Variant};
use secp256k1::rand::rngs::OsRng;
pub use secp256k1::{KeyPair, Secp256k1, SecretKey, XOnlyPublicKey};
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum KeyError {
    #[error("Invalid secret key string")]
    SkParseError,
    #[error("Invalid public key string")]
    PkParseError,
    #[error("Invalid bech32 secret key string")]
    Bech32SkParseError,
    #[error("Invalid bech32 public key string")]
    Bech32PkParseError,
    #[error("Secrete key missing")]
    SkMissing,
    #[error("Key pair missing")]
    KeyPairMissing,
    #[error("Failed to generate new keys")]
    KeyGenerationFailure,
}

pub trait FromBech32: Sized {
    fn from_bech32(secret_key: &str) -> Result<Self, KeyError>;
    fn from_bech32_public_key(publicc_key: &str) -> Result<Self, KeyError>;
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
    pub fn secret_key(&self) -> Result<SecretKey, KeyError> {
        if let Some(secret_key) = self.secret_key {
            Ok(secret_key)
        } else {
            Err(KeyError::SkMissing)
        }
    }

    /// Get keypair
    pub fn key_pair(&self) -> Result<KeyPair, KeyError> {
        if let Some(key_pair) = self.key_pair {
            Ok(key_pair)
        } else {
            Err(KeyError::KeyPairMissing)
        }
    }

    /// Get secret key as string
    pub fn secret_key_as_str(&self) -> Result<String, KeyError> {
        Ok(self.secret_key()?.display_secret().to_string())
    }

    /// Get public key as string
    pub fn public_key_as_str(&self) -> String {
        self.public_key.to_string()
    }
}

impl FromStr for Keys {
    type Err = anyhow::Error;

    fn from_str(secret_key: &str) -> Result<Self, Self::Err> {
        let secret_key = SecretKey::from_str(secret_key)?;
        Ok(Self::new(secret_key))
    }
}

impl FromBech32 for Keys {
    fn from_bech32(secret_key: &str) -> Result<Self, KeyError> {
        let (hrp, data, checksum) =
            bech32::decode(secret_key).map_err(|_| KeyError::Bech32SkParseError)?;

        if hrp != "nsec" || checksum != Variant::Bech32 {
            return Err(KeyError::Bech32SkParseError);
        }

        let data = Vec::<u8>::from_base32(&data).map_err(|_| KeyError::Bech32SkParseError)?;

        let secret_key =
            SecretKey::from_slice(data.as_slice()).map_err(|_| KeyError::Bech32SkParseError)?;

        let secp = Secp256k1::new();
        let key_pair = KeyPair::from_secret_key(&secp, &secret_key);
        let public_key = XOnlyPublicKey::from_keypair(&key_pair).0;

        Ok(Self {
            public_key,
            key_pair: Some(key_pair),
            secret_key: Some(secret_key),
        })
    }

    fn from_bech32_public_key(public_key: &str) -> Result<Self, KeyError> {
        let (hrp, data, checksum) =
            bech32::decode(public_key).map_err(|_| KeyError::Bech32PkParseError)?;

        if hrp != "npub" || checksum != Variant::Bech32 {
            return Err(KeyError::Bech32PkParseError);
        }

        let data = Vec::<u8>::from_base32(&data).map_err(|_| KeyError::Bech32PkParseError)?;

        let public_key =
            XOnlyPublicKey::from_slice(data.as_slice()).map_err(|_| KeyError::PkParseError)?;

        Ok(Keys {
            public_key,
            key_pair: None,
            secret_key: None,
        })
    }
}
