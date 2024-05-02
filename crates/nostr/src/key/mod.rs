// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Keys

use core::fmt;
#[cfg(feature = "std")]
use core::str::FromStr;

#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::{CryptoRng, Rng};
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Keypair, Message, Secp256k1, Signing, XOnlyPublicKey};

pub mod public_key;
pub mod secret_key;
#[cfg(feature = "std")]
pub mod vanity;

pub use self::public_key::PublicKey;
pub use self::secret_key::SecretKey;
use crate::util::hex;
#[cfg(feature = "std")]
use crate::SECP256K1;

/// [`Keys`] error
#[derive(Debug, PartialEq, Eq)]
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
    /// Hex decode error
    Hex(hex::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSecretKey => write!(f, "Invalid secret key"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::SkMissing => write!(f, "Secret key missing"),
            Self::InvalidChar(c) => write!(f, "Unsupported char: {c}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::Hex(e) => write!(f, "Hex: {e}"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<hex::Error> for Error {
    fn from(e: hex::Error) -> Self {
        Self::Hex(e)
    }
}

/// Keys
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    public_key: PublicKey,
    key_pair: Option<Keypair>,
    secret_key: Option<SecretKey>,
}

#[cfg(feature = "std")]
impl Keys {
    /// Initialize from secret key.
    #[inline]
    pub fn new(secret_key: SecretKey) -> Self {
        Self::new_with_ctx(&SECP256K1, secret_key)
    }

    /// Try to parse [Keys] from **secret key** `hex` or `bech32`
    #[inline]
    pub fn parse<S>(secret_key: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Self::parse_with_ctx(&SECP256K1, secret_key)
    }

    /// Generate new random [`Keys`]
    #[inline]
    pub fn generate() -> Self {
        Self::generate_with_rng(&mut OsRng)
    }

    /// Generate random [`Keys`] with custom [`Rng`]
    #[inline]
    pub fn generate_with_rng<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        Self::generate_with_ctx(&SECP256K1, rng)
    }

    /// Generate random [`Keys`] with custom [`Rng`] and without [`Keypair`]
    ///
    /// Useful for faster [`Keys`] generation (ex. vanity pubkey mining)
    #[inline]
    pub fn generate_without_keypair<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        Self::generate_without_keypair_with_ctx(&SECP256K1, rng)
    }

    /// Sign schnorr [`Message`]
    #[inline]
    pub fn sign_schnorr(&self, message: &Message) -> Result<Signature, Error> {
        self.sign_schnorr_with_ctx(&SECP256K1, message, &mut OsRng)
    }
}

impl Keys {
    /// Initialize from secret key.
    pub fn new_with_ctx<C>(secp: &Secp256k1<C>, secret_key: SecretKey) -> Self
    where
        C: Signing,
    {
        let key_pair = Keypair::from_secret_key(secp, &secret_key);
        let public_key = XOnlyPublicKey::from_keypair(&key_pair).0;

        Self {
            public_key: PublicKey::from(public_key),
            key_pair: Some(key_pair),
            secret_key: Some(secret_key),
        }
    }

    /// Try to parse [Keys] from **secret key** `hex` or `bech32`
    #[inline]
    pub fn parse_with_ctx<C, S>(secp: &Secp256k1<C>, secret_key: S) -> Result<Self, Error>
    where
        C: Signing,
        S: AsRef<str>,
    {
        let secret_key: SecretKey = SecretKey::parse(secret_key)?;
        Ok(Self::new_with_ctx(secp, secret_key))
    }

    /// Initialize with public key only (no secret key).
    #[inline]
    pub fn from_public_key(public_key: PublicKey) -> Self {
        Self {
            public_key,
            key_pair: None,
            secret_key: None,
        }
    }

    /// Generate random [`Keys`] with custom [`Rng`]
    #[inline]
    pub fn generate_with_ctx<C, R>(secp: &Secp256k1<C>, rng: &mut R) -> Self
    where
        C: Signing,
        R: Rng + ?Sized,
    {
        let secret_key: SecretKey = SecretKey::generate_with_ctx(secp, rng);
        Self::new_with_ctx(secp, secret_key)
    }

    /// Generate random [`Keys`] with custom [`Rng`] and without [`Keypair`]
    /// Useful for faster [`Keys`] generation (ex. vanity pubkey mining)
    pub fn generate_without_keypair_with_ctx<C, R>(secp: &Secp256k1<C>, rng: &mut R) -> Self
    where
        C: Signing,
        R: Rng + ?Sized,
    {
        let (secret_key, public_key) = secp.generate_keypair(rng);
        let (public_key, _) = public_key.x_only_public_key();
        Self {
            public_key: PublicKey::from(public_key),
            key_pair: None,
            secret_key: Some(SecretKey::from(secret_key)),
        }
    }

    /// Get public key
    #[inline]
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get public key
    #[inline]
    #[deprecated(since = "0.31.0", note = "Use `public_key` instead")]
    pub fn public_key_ref(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get secret key
    #[inline]
    pub fn secret_key(&self) -> Result<&SecretKey, Error> {
        if let Some(secret_key) = &self.secret_key {
            Ok(secret_key)
        } else {
            Err(Error::SkMissing)
        }
    }

    /// Get keypair
    ///
    /// If not exists, will be created
    pub fn key_pair<C>(&self, secp: &Secp256k1<C>) -> Result<Keypair, Error>
    where
        C: Signing,
    {
        if let Some(key_pair) = self.key_pair {
            Ok(key_pair)
        } else {
            let secret_key = self.secret_key()?;
            Ok(Keypair::from_secret_key(secp, secret_key))
        }
    }

    /// Sign schnorr [`Message`]
    pub fn sign_schnorr_with_ctx<C, R>(
        &self,
        secp: &Secp256k1<C>,
        message: &Message,
        rng: &mut R,
    ) -> Result<Signature, Error>
    where
        C: Signing,
        R: Rng + CryptoRng,
    {
        let keypair: &Keypair = &self.key_pair(secp)?;
        Ok(secp.sign_schnorr_with_rng(message, keypair, rng))
    }
}

#[cfg(feature = "std")]
impl FromStr for Keys {
    type Err = Error;

    /// Try to parse [Keys] from **secret key** `hex` or `bech32`
    #[inline]
    fn from_str(secret_key: &str) -> Result<Self, Self::Err> {
        Self::parse(secret_key)
    }
}

impl Drop for Keys {
    fn drop(&mut self) {
        self.secret_key = None;
    }
}
