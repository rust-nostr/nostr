// SPDX-License-Identifier: CC0-1.0

//! BIP32 implementation.
//!
//! Implementation of BIP32 hierarchical deterministic wallets, as defined
//! at <https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki>.

use alloc::vec::Vec;
use core::fmt;

use hashes::{hash160, sha512, Hash, HashEngine, Hmac, HmacEngine};
use secp256k1::{self, PublicKey, Secp256k1, SecretKey, Signing};

/// A BIP32 error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A secp256k1 error occurred
    Secp256k1(secp256k1::Error),
    /// A child number was provided that was out of range
    InvalidChildNumber(u32),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Secp256k1(ref e) => write!(f, "{e}"),
            Self::InvalidChildNumber(ref n) => write!(
                f,
                "child number {} is invalid (not within [0, 2^31 - 1])",
                n
            ),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Error {
        Error::Secp256k1(e)
    }
}

/// A chain code
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainCode([u8; 32]);

impl ChainCode {
    fn from_hmac(hmac: Hmac<sha512::Hash>) -> Self {
        let bytes = hmac.as_byte_array()[32..]
            .try_into()
            .expect("half of hmac is guaranteed to be 32 bytes");
        Self(bytes)
    }
}

/// A fingerprint
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Fingerprint([u8; 4]);

/// Extended key identifier as defined in BIP-32.
pub struct XKeyIdentifier(hash160::Hash);

/// A child number for a derived key
#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub enum ChildNumber {
    /// Non-hardened key
    Normal {
        /// Key index, within [0, 2^31 - 1]
        index: u32,
    },
    /// Hardened key
    Hardened {
        /// Key index, within [0, 2^31 - 1]
        index: u32,
    },
}
impl ChildNumber {
    /// Normal child number with index 0.
    pub const ZERO_NORMAL: Self = ChildNumber::Normal { index: 0 };

    /// Constructs a new [`Normal`] from an index, returns an error if the index is not within
    /// [0, 2^31 - 1].
    ///
    /// [`Normal`]: #variant.Normal
    pub fn from_normal_idx(index: u32) -> Result<Self, Error> {
        if index & (1 << 31) == 0 {
            Ok(ChildNumber::Normal { index })
        } else {
            Err(Error::InvalidChildNumber(index))
        }
    }

    /// Constructs a new [`Hardened`] from an index, returns an error if the index is not within
    /// [0, 2^31 - 1].
    ///
    /// [`Hardened`]: #variant.Hardened
    pub fn from_hardened_idx(index: u32) -> Result<Self, Error> {
        if index & (1 << 31) == 0 {
            Ok(ChildNumber::Hardened { index })
        } else {
            Err(Error::InvalidChildNumber(index))
        }
    }

    fn as_u32(&self) -> u32 {
        match self {
            ChildNumber::Normal { index } => *index,
            ChildNumber::Hardened { index } => index | (1 << 31),
        }
    }
}

/// Extended private key
pub struct Xpriv {
    /// How many derivations this key is from the master (which is 0)
    pub depth: u8,
    /// Fingerprint of the parent key (0 for master)
    pub parent_fingerprint: Fingerprint,
    /// Child number of the key used to derive from parent (0 for master)
    pub child_number: ChildNumber,
    /// Private key
    pub private_key: SecretKey,
    /// Chain code
    pub chain_code: ChainCode,
}

impl Xpriv {
    /// Constructs a new master key from a seed value
    pub fn new_master(seed: &[u8]) -> Result<Xpriv, Error> {
        let mut hmac_engine: HmacEngine<sha512::Hash> = HmacEngine::new(b"Bitcoin seed");
        hmac_engine.input(seed);
        let hmac_result: Hmac<sha512::Hash> = Hmac::from_engine(hmac_engine);

        Ok(Xpriv {
            depth: 0,
            parent_fingerprint: Default::default(),
            child_number: ChildNumber::ZERO_NORMAL,
            private_key: SecretKey::from_slice(&hmac_result.as_byte_array()[..32])?,
            chain_code: ChainCode::from_hmac(hmac_result),
        })
    }

    /// Derives an extended private key from a path.
    ///
    /// The `path` argument can be both of type `DerivationPath` or `Vec<ChildNumber>`.
    pub fn derive_xpriv<C: Signing>(self, secp: &Secp256k1<C>, path: Vec<ChildNumber>) -> Xpriv {
        let mut sk: Xpriv = self;
        for cnum in path.into_iter() {
            sk = sk.ckd_priv(secp, cnum)
        }
        sk
    }

    /// Private->Private child key derivation
    fn ckd_priv<C: Signing>(&self, secp: &Secp256k1<C>, i: ChildNumber) -> Xpriv {
        let mut hmac_engine: HmacEngine<sha512::Hash> = HmacEngine::new(&self.chain_code.0);
        match i {
            ChildNumber::Normal { .. } => {
                // Non-hardened key: compute public data and use that
                hmac_engine
                    .input(&PublicKey::from_secret_key(secp, &self.private_key).serialize()[..]);
            }
            ChildNumber::Hardened { .. } => {
                // Hardened key: use only secret data to prevent public derivation
                hmac_engine.input(&[0u8]);
                hmac_engine.input(&self.private_key[..]);
            }
        }

        hmac_engine.input(&i.as_u32().to_be_bytes());
        let hmac_result: Hmac<sha512::Hash> = Hmac::from_engine(hmac_engine);
        let sk = SecretKey::from_slice(&hmac_result.as_byte_array()[..32])
            .expect("statistically impossible to hit");
        let tweaked = sk
            .add_tweak(&self.private_key.into())
            .expect("statistically impossible to hit");

        Xpriv {
            depth: self.depth + 1,
            parent_fingerprint: self.fingerprint(secp),
            child_number: i,
            private_key: tweaked,
            chain_code: ChainCode::from_hmac(hmac_result),
        }
    }

    /// Returns the HASH160 of the public key belonging to the xpriv
    pub fn identifier<C: Signing>(&self, secp: &Secp256k1<C>) -> XKeyIdentifier {
        Xpub::from_xpriv(secp, self).identifier()
    }

    /// Returns the first four bytes of the identifier
    pub fn fingerprint<C: Signing>(&self, secp: &Secp256k1<C>) -> Fingerprint {
        let bytes = self.identifier(secp).0.as_byte_array()[0..4]
            .try_into()
            .expect("4 is the fingerprint length");
        Fingerprint(bytes)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Xpub {
    /// How many derivations this key is from the master (which is 0)
    pub depth: u8,
    /// Fingerprint of the parent key
    pub parent_fingerprint: Fingerprint,
    /// Child number of the key used to derive from parent (0 for master)
    pub child_number: ChildNumber,
    /// Public key
    pub public_key: PublicKey,
    /// Chain code
    pub chain_code: ChainCode,
}

impl Xpub {
    /// Constructs a new extended public key from an extended private key.
    pub fn from_xpriv<C>(secp: &Secp256k1<C>, xpriv: &Xpriv) -> Self
    where
        C: Signing,
    {
        Self {
            depth: xpriv.depth,
            parent_fingerprint: xpriv.parent_fingerprint,
            child_number: xpriv.child_number,
            public_key: PublicKey::from_secret_key(secp, &xpriv.private_key),
            chain_code: xpriv.chain_code,
        }
    }

    /// Returns the HASH160 of the public key component of the xpub
    pub fn identifier(&self) -> XKeyIdentifier {
        XKeyIdentifier(hash160::Hash::hash(&self.public_key.serialize()))
    }
}
