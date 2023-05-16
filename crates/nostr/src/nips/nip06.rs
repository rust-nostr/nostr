// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP06
//!
//! <https://github.com/nostr-protocol/nips/blob/master/06.md>

#[cfg(all(feature = "alloc", not(feature = "std")))]
use core::error::Error as StdError;
use core::fmt;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::error::Error as StdError;

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, ExtendedPrivKey};
use bitcoin::Network;
use bitcoin_hashes::hmac::{Hmac, HmacEngine};
use bitcoin_hashes::{sha512, Hash, HashEngine};
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::RngCore;

use crate::{Keys, SECP256K1};

/// `NIP06` error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// BIP32 error
    BIP32(bitcoin::bip32::Error),
    /// BIP39 error
    BIP39(bip39::Error),
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BIP32(e) => write!(f, "{e}"),
            Self::BIP39(e) => write!(f, "{e}"),
        }
    }
}

impl From<bitcoin::bip32::Error> for Error {
    fn from(e: bitcoin::bip32::Error) -> Self {
        Self::BIP32(e)
    }
}

impl From<bip39::Error> for Error {
    fn from(e: bip39::Error) -> Self {
        Self::BIP39(e)
    }
}

#[allow(missing_docs)]
pub trait FromMnemonic: Sized {
    type Err;
    fn from_mnemonic<S>(mnemonic: S, passphrase: Option<S>) -> Result<Self, Self::Err>
    where
        S: Into<String>;
}

#[allow(missing_docs)]
pub trait GenerateMnemonic {
    type Err;
    fn generate_mnemonic(word_count: usize) -> Result<Mnemonic, Self::Err>;
}

impl FromMnemonic for Keys {
    type Err = Error;

    /// Derive keys from BIP-39 mnemonics (ENGLISH wordlist).
    fn from_mnemonic<S>(mnemonic: S, passphrase: Option<S>) -> Result<Self, Self::Err>
    where
        S: Into<String>,
    {
        let mnemonic = Mnemonic::from_str(&mnemonic.into())?;
        let seed = mnemonic.to_seed(passphrase.map(|p| p.into()).unwrap_or_default());
        let root_key = ExtendedPrivKey::new_master(Network::Bitcoin, &seed)?;
        let path = DerivationPath::from_str("m/44'/1237'/0'/0/0")?;
        let child_xprv = root_key.derive_priv(&SECP256K1, &path)?;
        Ok(Self::new(child_xprv.private_key))
    }
}

impl GenerateMnemonic for Keys {
    type Err = Error;

    /// Generate new `mnemonic`
    fn generate_mnemonic(word_count: usize) -> Result<Mnemonic, Self::Err> {
        let mut h = HmacEngine::<sha512::Hash>::new(b"nostr");
        let mut os_random = [0u8; 32];
        OsRng.fill_bytes(&mut os_random);
        h.input(&os_random);
        let entropy: [u8; 64] = Hmac::from_engine(h).to_byte_array();
        let len: usize = word_count * 4 / 3;
        Ok(Mnemonic::from_entropy(&entropy[0..len])?)
    }
}

#[cfg(test)]
mod tests {
    use secp256k1::SecretKey;

    use super::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn test_nip06() -> Result<()> {
        let mnemonic: &str = "equal dragon fabric refuse stable cherry smoke allow alley easy never medal attend together lumber movie what sad siege weather matrix buffalo state shoot";
        let keys = Keys::from_mnemonic(mnemonic, None)?;

        assert_eq!(
            keys.secret_key()?,
            SecretKey::from_str(
                "06992419a8fe821dd8de03d4c300614e8feefb5ea936b76f89976dcace8aebee"
            )?
        );

        Ok(())
    }
}
