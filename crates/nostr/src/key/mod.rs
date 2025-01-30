// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Keys

#[cfg(not(feature = "std"))]
use core::cell::OnceCell;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
#[cfg(feature = "std")]
use core::str::FromStr;
#[cfg(feature = "std")]
use std::sync::OnceLock as OnceCell;

#[cfg(feature = "std")]
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::{CryptoRng, Rng};
use secp256k1::schnorr::Signature;
use secp256k1::{self, Keypair, Message, Secp256k1, Signing, XOnlyPublicKey};

pub mod public_key;
pub mod secret_key;
#[cfg(feature = "std")]
pub mod vanity;

pub use self::public_key::PublicKey;
pub use self::secret_key::SecretKey;
#[cfg(feature = "std")]
use crate::signer::{NostrSigner, SignerBackend, SignerError};
use crate::util::hex;
#[cfg(feature = "std")]
use crate::util::BoxedFuture;
#[cfg(feature = "std")]
use crate::{Event, UnsignedEvent, SECP256K1};

/// [`Keys`] error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Hex decode error
    Hex(hex::Error),
    /// Invalid secret key
    InvalidSecretKey,
    /// Invalid public key
    InvalidPublicKey,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::Hex(e) => write!(f, "{e}"),
            Self::InvalidSecretKey => write!(f, "Invalid secret key"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
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

/// Nostr keys
#[derive(Clone)]
pub struct Keys {
    /// Public key
    pub public_key: PublicKey,
    secret_key: SecretKey,
    key_pair: OnceCell<Keypair>,
}

impl fmt::Debug for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keys")
            .field("public_key", &self.public_key)
            .finish()
    }
}

impl PartialEq for Keys {
    fn eq(&self, other: &Self) -> bool {
        self.public_key == other.public_key
    }
}

impl Eq for Keys {}

impl PartialOrd for Keys {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Keys {
    fn cmp(&self, other: &Self) -> Ordering {
        self.public_key.cmp(&other.public_key)
    }
}

impl Hash for Keys {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.public_key.hash(state)
    }
}

impl Keys {
    /// Construct from a secret key.
    ///
    /// This method internally constructs the [`Keypair`] and derives the [`PublicKey`].
    #[inline]
    #[cfg(feature = "std")]
    pub fn new(secret_key: SecretKey) -> Self {
        Self::new_with_ctx(SECP256K1, secret_key)
    }

    /// Construct from a secret key.
    ///
    /// This method internally constructs the [`Keypair`] and derives the [`PublicKey`].
    pub fn new_with_ctx<C>(secp: &Secp256k1<C>, secret_key: SecretKey) -> Self
    where
        C: Signing,
    {
        let key_pair: Keypair = Keypair::from_secret_key(secp, &secret_key);
        let public_key: XOnlyPublicKey = XOnlyPublicKey::from_keypair(&key_pair).0;

        Self {
            public_key: PublicKey::from(public_key),
            secret_key,
            key_pair: OnceCell::from(key_pair),
        }
    }

    /// Parse secret key and construct keys.
    ///
    /// Check [`SecretKey::parse`] to learn more about secret key parsing.
    #[inline]
    #[cfg(feature = "std")]
    pub fn parse(secret_key: &str) -> Result<Self, Error> {
        Self::parse_with_ctx(SECP256K1, secret_key)
    }

    /// Parse secret key and construct keys.
    ///
    /// Check [`SecretKey::parse`] to learn more about secret key parsing.
    #[inline]
    pub fn parse_with_ctx<C>(secp: &Secp256k1<C>, secret_key: &str) -> Result<Self, Error>
    where
        C: Signing,
    {
        let secret_key: SecretKey = SecretKey::parse(secret_key)?;
        Ok(Self::new_with_ctx(secp, secret_key))
    }

    /// Generate random keys
    ///
    /// This constructor uses a random number generator that retrieves randomness from the operating system (see [`OsRng`]).
    ///
    /// Use [`Keys::generate_with_rng`] to specify a custom random source.
    ///
    /// Check [`Keys::generate_with_ctx`] to learn more about how this constructor works internally.
    #[inline]
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        Self::generate_with_rng(&mut OsRng)
    }

    /// Generate random keys using a custom random source
    ///
    /// Check [`Keys::generate_with_ctx`] to learn more about how this constructor works internally.
    #[inline]
    #[cfg(feature = "std")]
    pub fn generate_with_rng<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        Self::generate_with_ctx(SECP256K1, rng)
    }

    /// Generate random keys
    ///
    /// Generate random keys **without** construct the [`Keypair`].
    /// This allows faster keys generation (i.e., for vanity pubkey mining).
    /// The [`Keypair`] will be automatically created when needed and stored in a cell.
    #[inline]
    pub fn generate_with_ctx<C, R>(secp: &Secp256k1<C>, rng: &mut R) -> Self
    where
        C: Signing,
        R: Rng + ?Sized,
    {
        let (secret_key, public_key) = secp.generate_keypair(rng);
        let (public_key, _) = public_key.x_only_public_key();
        Self {
            public_key: PublicKey::from(public_key),
            secret_key: SecretKey::from(secret_key),
            key_pair: OnceCell::new(),
        }
    }

    /// Get public key
    #[inline]
    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    /// Get secret key
    #[inline]
    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }

    /// Get keypair
    #[inline]
    pub fn key_pair<C>(&self, secp: &Secp256k1<C>) -> &Keypair
    where
        C: Signing,
    {
        self.key_pair
            .get_or_init(|| Keypair::from_secret_key(secp, &self.secret_key))
    }

    /// Creates a schnorr signature of the [`Message`].
    ///
    /// This method uses a random number generator that retrieves randomness from the operating system (see [`OsRng`]).
    #[inline]
    #[cfg(feature = "std")]
    pub fn sign_schnorr(&self, message: &Message) -> Signature {
        self.sign_schnorr_with_ctx(SECP256K1, message, &mut OsRng)
    }

    /// Creates a schnorr signature of the [`Message`] using a custom random number generation source.
    pub fn sign_schnorr_with_ctx<C, R>(
        &self,
        secp: &Secp256k1<C>,
        message: &Message,
        rng: &mut R,
    ) -> Signature
    where
        C: Signing,
        R: Rng + CryptoRng,
    {
        let keypair: &Keypair = self.key_pair(secp);
        secp.sign_schnorr_with_rng(message, keypair, rng)
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

#[cfg(feature = "std")]
impl NostrSigner for Keys {
    fn backend(&self) -> SignerBackend {
        SignerBackend::Keys
    }

    fn get_public_key(&self) -> BoxedFuture<Result<PublicKey, SignerError>> {
        Box::pin(async { Ok(self.public_key) })
    }

    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<Result<Event, SignerError>> {
        Box::pin(async { unsigned.sign_with_keys(self).map_err(SignerError::backend) })
    }

    fn nip04_encrypt<'a>(
        &'a self,
        _public_key: &'a PublicKey,
        _content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            #[cfg(feature = "nip04")]
            {
                let secret_key: &SecretKey = self.secret_key();
                crate::nips::nip04::encrypt(secret_key, _public_key, _content)
                    .map_err(SignerError::backend)
            }

            #[cfg(not(feature = "nip04"))]
            Err(SignerError::from("NIP04 feature is not enabled"))
        })
    }

    fn nip04_decrypt<'a>(
        &'a self,
        _public_key: &'a PublicKey,
        _content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            #[cfg(feature = "nip04")]
            {
                let secret_key: &SecretKey = self.secret_key();
                crate::nips::nip04::decrypt(secret_key, _public_key, _content)
                    .map_err(SignerError::backend)
            }

            #[cfg(not(feature = "nip04"))]
            Err(SignerError::from("NIP04 feature is not enabled"))
        })
    }

    fn nip44_encrypt<'a>(
        &'a self,
        _public_key: &'a PublicKey,
        _content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            #[cfg(feature = "nip44")]
            {
                use crate::nips::nip44::{self, Version};
                let secret_key: &SecretKey = self.secret_key();
                nip44::encrypt(secret_key, _public_key, _content, Version::default())
                    .map_err(SignerError::backend)
            }

            #[cfg(not(feature = "nip44"))]
            Err(SignerError::from("NIP44 feature is not enabled"))
        })
    }

    fn nip44_decrypt<'a>(
        &'a self,
        _public_key: &'a PublicKey,
        _content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            #[cfg(feature = "nip44")]
            {
                let secret_key: &SecretKey = self.secret_key();
                crate::nips::nip44::decrypt(secret_key, _public_key, _content)
                    .map_err(SignerError::backend)
            }

            #[cfg(not(feature = "nip44"))]
            Err(SignerError::from("NIP44 feature is not enabled"))
        })
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    const SECRET_KEY_BECH32: &str =
        "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";
    const SECRET_KEY_HEX: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";

    #[test]
    fn parse_keys() -> Result<(), Error> {
        Keys::parse(SECRET_KEY_BECH32)?;
        Keys::parse(SECRET_KEY_HEX)?;
        Ok(())
    }

    #[test]
    fn parse_invalid_keys() {
        assert_eq!(Keys::parse("nsec...").unwrap_err(), Error::InvalidSecretKey);
        assert_eq!(
            Keys::parse("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
                .unwrap_err(),
            Error::InvalidSecretKey
        );
        assert_eq!(
            Keys::parse("6b911fd37cdf5c8").unwrap_err(),
            Error::InvalidSecretKey
        );
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn generate_keys(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(Keys::generate());
        });
    }
}
