use secp256k1::{
    rand::rngs::OsRng,
    schnorrsig,
    schnorrsig::{KeyPair, PublicKey},
    Secp256k1, SecretKey,
};

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
    pub public_key: PublicKey,
    key_pair: Option<KeyPair>,
    secret_key: Option<SecretKey>,
}

impl Keys {
    pub fn generate_from_os_random() -> Result<Self, KeyError> {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().unwrap();
        let (sk, _pk) = secp.generate_keypair(&mut rng);
        Self::new(&sk.to_string())
    }

    pub fn new_pub_only(pk: &str) -> Result<Self, KeyError> {
        let pk = schnorrsig::PublicKey::from_str(pk).map_err(|_| KeyError::PkParseError)?;

        Ok(Keys {
            public_key: pk,
            key_pair: None,
            secret_key: None,
        })
    }

    pub fn new(sk: &str) -> Result<Self, KeyError> {
        let secp = Secp256k1::new();
        let sk = SecretKey::from_str(sk).map_err(|_| KeyError::SkParseError)?;
        let key_pair = schnorrsig::KeyPair::from_secret_key(&secp, sk);
        let pk = schnorrsig::PublicKey::from_keypair(&secp, &key_pair);

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
        // TODO learn the better rust way of writing this
        if self.secret_key.is_none() {
            Err(KeyError::SkMissing)
        } else {
            Ok(self.secret_key.unwrap())
        }
    }

    pub fn secret_key_as_str(&self) -> Result<String, KeyError> {
        // TODO learn the better rust way of writing this
        if self.secret_key.is_none() {
            Err(KeyError::SkMissing)
        } else {
            Ok(self.secret_key.unwrap().to_string())
        }
    }

    pub fn key_pair(&self) -> Result<KeyPair, KeyError> {
        // TODO learn the better rust way of writing this
        if self.key_pair.is_none() {
            Err(KeyError::KeyPairMissing)
        } else {
            Ok(self.key_pair.unwrap())
        }
    }
}

// Given a secret key return all the scep256k1 primitives
// pub fn gen_keys(sk: &str) -> (KeyPair, PublicKey, SecretKey) {
//     let secp = Secp256k1::new();
//     let sk = SecretKey::from_str(sk).unwrap();
//     let key_pair = schnorrsig::KeyPair::from_secret_key(&secp, sk);
//     let pk = schnorrsig::PublicKey::from_keypair(&secp, &key_pair);

//     return (key_pair, pk, sk);
// }
