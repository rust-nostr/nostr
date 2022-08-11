use secp256k1::{KeyPair, rand::rngs::OsRng, Secp256k1, SecretKey, XOnlyPublicKey};

use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum KeyError {
    #[error("Invalid secret key string")]
    SkParseError,
    #[error("Invalid public key string")]
    PkParseError,
    #[error("Secrete key missing")]
    SkMissing,
    #[error("Key pair missing")]
    KeyPairMissing,
    #[error("Failed to generate new keys")]
    KeyGenerationFailure,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keys {
    pub public_key: XOnlyPublicKey,
    key_pair: Option<KeyPair>,
    secret_key: Option<SecretKey>,
}

impl Keys {
    pub fn generate_from_os_random() -> Result<Self, KeyError> {
        let secp = Secp256k1::new();
        let mut rng = OsRng::default();
        let (sk, _pk) = secp.generate_keypair(&mut rng);
        Self::new(sk)
    }

    pub fn new_pub_only(pk: &str) -> Result<Self, KeyError> {
        let pk = XOnlyPublicKey::from_str(pk).map_err(|_| KeyError::PkParseError)?;

        Ok(Keys {
            public_key: pk,
            key_pair: None,
            secret_key: None,
        })
    }

    pub fn new(sk: SecretKey) -> Result<Self, KeyError> {
        let secp = Secp256k1::new();
        let key_pair = KeyPair::from_secret_key(&secp, &sk);
        let pk = XOnlyPublicKey::from_keypair(&key_pair).0;

        Ok(Self {
            public_key: pk,
            key_pair: Some(key_pair),
            secret_key: Some(sk),
        })
    }

    pub fn public_key_as_str(&self) -> String {
        self.public_key.to_string()
    }

    pub fn secret_key(&self) -> Result<SecretKey, KeyError> {
        if let Some(secret_key) = self.secret_key {
            Ok(secret_key)
        } else {
            Err(KeyError::SkMissing)
        }
    }

    pub fn key_pair(&self) -> Result<KeyPair, KeyError> {
        if let Some(key_pair) = self.key_pair {
            Ok(key_pair)
        } else {
            Err(KeyError::KeyPairMissing)
        }
    }
}
