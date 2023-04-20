// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Keys
//!
//! This module defines the [`Keys`] structure.

use core::fmt;
#[cfg(feature = "nip19")]
use core::str::FromStr;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use rand_core::OsRng;
#[cfg(feature = "std")]
use secp256k1::rand::rngs::OsRng;

#[cfg(feature = "alloc")]
use secp256k1::Secp256k1;
#[cfg(feature = "alloc")]
use secp256k1::Signing;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use rand::Rng;
#[cfg(feature = "std")]
use secp256k1::rand::Rng;
use secp256k1::schnorr::Signature;
use secp256k1::Message;
pub use secp256k1::{KeyPair, PublicKey, SecretKey, XOnlyPublicKey};

#[cfg(feature = "std")]
use crate::SECP256K1;

#[cfg(feature = "vanity")]
pub mod vanity;

#[cfg(feature = "nip19")]
use crate::nips::nip19::FromBech32;

/// [`Keys`] error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Invalid secret key
    InvalidSecretKey,
    /// Invalid public key
    InvalidPublicKey,
    /// Secret key missing
    SkMissing,
    /// Unsupported char
    InvalidChar(char),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSecretKey => write!(f, "Invalid secret key"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::SkMissing => write!(f, "Secret key missing"),
            Self::InvalidChar(c) => write!(f, "Unsupported char: {c}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/// Trait for [`Keys`]
pub trait FromSkStr: Sized {
    /// Error
    type Err;
    #[cfg(all(feature = "std", not(feature = "alloc")))]
    /// Init [`Keys`] from `hex` or `bech32` secret key string
    fn from_sk_str(secret_key: &str) -> Result<Self, Self::Err>;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    /// Init [`Keys`] from `hex` or `bech32` secret key string
    fn from_sk_str<C: secp256k1::Signing>(
        secret_key: &str,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Self::Err>;
}

/// Trait for [`Keys`]
pub trait FromPkStr: Sized {
    /// Error
    type Err;
    /// Init [`Keys`] from `hex` or `bech32` public key string
    fn from_pk_str(public_key: &str) -> Result<Self, Self::Err>;
}

/// Keys
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    public_key: XOnlyPublicKey,
    key_pair: Option<KeyPair>,
    secret_key: Option<SecretKey>,
}

impl Keys {
    /// Initialize from secret key.
    #[cfg(feature = "std")]
    pub fn new(secret_key: SecretKey) -> Self {
        let key_pair = KeyPair::from_secret_key(SECP256K1, &secret_key);
        let public_key = XOnlyPublicKey::from_keypair(&key_pair).0;

        Self {
            public_key,
            key_pair: Some(key_pair),
            secret_key: Some(secret_key),
        }
    }

    /// Initialize from secret key.
    #[cfg(not(feature = "std"))]
    pub fn new_with_secp<C: secp256k1::Signing>(
        secret_key: SecretKey,
        secp: &Secp256k1<C>,
    ) -> Self {
        let key_pair = KeyPair::from_secret_key(secp, &secret_key);
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

    /// Generate new random [`Keys`]
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        let mut rng = OsRng::default();
        let (secret_key, _) = SECP256K1.generate_keypair(&mut rng);
        Self::new(secret_key)
    }

    /// Generate new random [`Keys`]
    #[cfg(not(feature = "std"))]
    pub fn generate_with_secp<C: Signing>(secp: &Secp256k1<C>) -> Self {
        let mut rng = OsRng::default();
        let (secret_key, _) = secp.generate_keypair(&mut rng);
        Self::new_with_secp(secret_key, secp)
    }

    /// Generate random [`Keys`] with custom [`Rng`]
    #[cfg(feature = "std")]
    pub fn generate_with_rng<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        let (secret_key, _) = SECP256K1.generate_keypair(rng);
        Self::new(secret_key)
    }

    /// Generate random [`Keys`] with custom [`Rng`] and given [`Secp256k1`]
    #[cfg(not(feature = "std"))]
    pub fn generate_with_rng_with_secp<R, C: Signing>(rng: &mut R, secp: &Secp256k1<C>) -> Self
    where
        R: Rng + ?Sized,
    {
        let (secret_key, _) = secp.generate_keypair(rng);
        Self::new_with_secp(secret_key, secp)
    }

    /// Generate random [`Keys`] with custom [`Rng`] and without [`KeyPair`]
    /// Useful for faster [`Keys`] generation (ex. vanity pubkey mining)
    #[cfg(feature = "std")]
    pub fn generate_without_keypair<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        let (secret_key, public_key) = SECP256K1.generate_keypair(rng);
        let (public_key, _) = public_key.x_only_public_key();
        Self {
            public_key,
            key_pair: None,
            secret_key: Some(secret_key),
        }
    }

    /// Generate random [`Keys`] with custom [`Rng`] and without [`KeyPair`]
    /// Useful for faster [`Keys`] generation (ex. vanity pubkey mining)
    #[cfg(not(feature = "std"))]
    pub fn generate_without_keypair_with_secp<R, C: Signing>(
        rng: &mut R,
        secp: &Secp256k1<C>,
    ) -> Self
    where
        R: Rng + ?Sized,
    {
        let (secret_key, public_key) = secp.generate_keypair(rng);
        let (public_key, _) = public_key.x_only_public_key();
        Self {
            public_key,
            key_pair: None,
            secret_key: Some(secret_key),
        }
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

    /// Get [`PublicKey`]
    #[cfg(feature = "std")]
    pub fn normalized_public_key(&self) -> Result<PublicKey, Error> {
        Ok(self.secret_key()?.public_key(SECP256K1))
    }

    /// Get keypair
    ///
    /// If not exists, will be created
    #[cfg(feature = "std")]
    pub fn key_pair(&self) -> Result<KeyPair, Error> {
        if let Some(key_pair) = self.key_pair {
            Ok(key_pair)
        } else {
            let sk = self.secret_key()?;
            Ok(KeyPair::from_secret_key(SECP256K1, &sk))
        }
    }

    /// Get keypair
    ///
    /// If not exists, will be created
    #[cfg(not(feature = "std"))]
    pub fn key_pair_from_secp<C: Signing>(&self, secp: &Secp256k1<C>) -> Result<KeyPair, Error> {
        if let Some(key_pair) = self.key_pair {
            Ok(key_pair)
        } else {
            let sk = self.secret_key()?;
            Ok(KeyPair::from_secret_key(secp, &sk))
        }
    }

    /// Sign schnorr [`Message`]
    #[cfg(feature = "std")]
    pub fn sign_schnorr(&self, message: &Message) -> Result<Signature, Error> {
        let keypair: &KeyPair = &self.key_pair()?;
        Ok(SECP256K1.sign_schnorr(message, keypair))
    }

    /// Sign schnorr [`Message`]
    #[cfg(not(feature = "std"))]
    pub fn sign_schnorr_with_secp<C: Signing>(
        &self,
        message: &Message,
        secp: &Secp256k1<C>,
    ) -> Result<Signature, Error> {
        let keypair: &KeyPair = &self.key_pair_from_secp(&secp)?;
        Ok(secp.sign_schnorr_no_aux_rand(message, keypair))
    }
}

#[cfg(all(feature = "std", feature = "nip19", not(feature = "alloc")))]
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
#[cfg(all(feature = "alloc", feature = "nip19", not(feature = "std")))]
impl FromSkStr for Keys {
    type Err = Error;

    /// Init [`Keys`] from `hex` or `bech32` secret key
    fn from_sk_str<C: secp256k1::Signing>(
        secret_key: &str,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Self::Err> {
        match SecretKey::from_str(secret_key) {
            Ok(secret_key) => Ok(Self::new_with_secp(secret_key, &secp)),
            Err(_) => match SecretKey::from_bech32(secret_key) {
                Ok(secret_key) => Ok(Self::new_with_secp(secret_key, &secp)),
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
