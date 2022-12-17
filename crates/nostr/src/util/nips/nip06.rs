// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bitcoin::hashes::{sha512, Hash, HashEngine};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
use bitcoin::Network;

use crate::key::Keys;
use crate::util::time;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// BIP32 error
    BIP32(bitcoin::util::bip32::Error),
    /// BIP39 error
    BIP39(bip39::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BIP32(err) => write!(f, "BIP32 error: {}", err),
            Self::BIP39(err) => write!(f, "BIP39 error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<bitcoin::util::bip32::Error> for Error {
    fn from(err: bitcoin::util::bip32::Error) -> Self {
        Self::BIP32(err)
    }
}

impl From<bip39::Error> for Error {
    fn from(err: bip39::Error) -> Self {
        Self::BIP39(err)
    }
}

pub trait FromMnemonic: Sized {
    type Err;
    fn from_mnemonic<S>(mnemonic: S, passphrase: Option<S>) -> Result<Self, Self::Err>
    where
        S: Into<String>;
}

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
        let secp = Secp256k1::new();
        let child_xprv = root_key.derive_priv(&secp, &path)?;
        Ok(Self::new(child_xprv.private_key))
    }
}

impl GenerateMnemonic for Keys {
    type Err = Error;

    fn generate_mnemonic(word_count: usize) -> Result<Mnemonic, Self::Err> {
        let mut h = HmacEngine::<sha512::Hash>::new(b"nostr");
        let random: [u8; 32] = bitcoin::secp256k1::rand::random();
        h.input(&random);
        h.input(&time::timestamp_nanos().to_be_bytes());
        let entropy: [u8; 64] = Hmac::from_engine(h).into_inner();
        let len: usize = word_count * 4 / 3;
        Ok(Mnemonic::from_entropy(&entropy[0..len])?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::key::ToBech32;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn test_nip06() -> Result<()> {
        let mnemonic: &str = "equal dragon fabric refuse stable cherry smoke allow alley easy never medal attend together lumber movie what sad siege weather matrix buffalo state shoot";
        let keys = Keys::from_mnemonic(mnemonic, None)?;

        assert_eq!(
            keys.secret_key()?.to_bech32()?,
            "nsec1q6vjgxdgl6ppmkx7q02vxqrpf687a7674ymtwmufjaku4n52a0hq9glmaf".to_string()
        );

        Ok(())
    }
}
