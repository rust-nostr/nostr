// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use anyhow::anyhow;
use bech32::{self, FromBase32, ToBase32, Variant};
use bip32::{DerivationPath, Language, Mnemonic, XPrv};
use secp256k1::rand::rngs::OsRng;
pub use secp256k1::{KeyPair, Secp256k1, SecretKey, XOnlyPublicKey};
use thiserror::Error;

const PREFIX_BECH32_SECRET_KEY: &str = "nsec";
const PREFIX_BECH32_PUBLIC_KEY: &str = "npub";

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
    fn from_bech32_public_key(public_key: &str) -> Result<Self, KeyError>;
}

pub trait ToBech32 {
    type Err;
    fn to_bech32(&self) -> Result<String, Self::Err>;
}

pub trait FromSeedPhrase: Sized {
    type Err;
    fn from_seed(seed: &str) -> Result<Self, Self::Err>;
}

impl ToBech32 for XOnlyPublicKey {
    type Err = anyhow::Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.serialize().to_base32();
        let encoded = bech32::encode(PREFIX_BECH32_PUBLIC_KEY, data, Variant::Bech32)?;
        Ok(encoded)
    }
}

impl ToBech32 for SecretKey {
    type Err = anyhow::Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.secret_bytes().to_base32();
        let encoded = bech32::encode(PREFIX_BECH32_SECRET_KEY, data, Variant::Bech32)?;
        Ok(encoded)
    }
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

        if hrp != PREFIX_BECH32_SECRET_KEY || checksum != Variant::Bech32 {
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

        if hrp != PREFIX_BECH32_PUBLIC_KEY || checksum != Variant::Bech32 {
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

impl FromSeedPhrase for Keys {
    type Err = anyhow::Error;

    /// Derive keys from BIP-39 mnemonics (ENGLISH wordlist).
    /// ONLY 24-WORD BIP-39 MNEMONICS ARE SUPPORTED!
    fn from_seed(phrase: &str) -> Result<Self, Self::Err> {
        if phrase.split(' ').count() != 24 {
            return Err(anyhow!(
                "Invalid mnemonic length: only 24-word BIP-39 mnemonics are supported."
            ));
        }

        let mnemonic = Mnemonic::new(phrase, Language::English)?;
        let seed = mnemonic.to_seed("");
        let child_path = DerivationPath::from_str("m/44'/1237'/0'/0/0")?;
        let child_xprv = XPrv::derive_from_path(seed, &child_path)?;

        let secret_key = SecretKey::from_slice(child_xprv.private_key().to_bytes().as_slice())?;

        Ok(Self::new(secret_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    #[test]
    fn to_bech32_public_key() -> Result<()> {
        let bech32_pubkey_str: &str =
            "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy";
        let keys = Keys::from_bech32_public_key(bech32_pubkey_str)?;
        let public_key: XOnlyPublicKey = keys.public_key();

        assert_eq!(bech32_pubkey_str.to_string(), public_key.to_bech32()?);

        Ok(())
    }

    #[test]
    fn to_bech32_secret_key() -> Result<()> {
        let bech32_secret_key_str: &str =
            "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";
        let keys = Keys::from_bech32(bech32_secret_key_str)?;
        let secret_key: SecretKey = keys.secret_key()?;

        assert_eq!(bech32_secret_key_str.to_string(), secret_key.to_bech32()?);

        Ok(())
    }
}
