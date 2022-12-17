// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::secp256k1::Secp256k1;

use crate::key::Keys;

pub trait FromMnemonic: Sized {
    type Err;
    fn from_mnemonic(mnemonic: &str) -> Result<Self, Self::Err>;
}

pub trait GenerateMnemonic {
    type Err;
    fn generate_mnemonic(word_count: usize) -> Result<Mnemonic, Self::Err>;
}

impl FromMnemonic for Keys {
    type Err = anyhow::Error;

    /// Derive keys from BIP-39 mnemonics (ENGLISH wordlist).
    fn from_mnemonic(mnemonic: &str) -> Result<Self, Self::Err> {
        use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
        use bitcoin::Network;

        let mnemonic = Mnemonic::from_str(mnemonic)?;
        let seed = mnemonic.to_seed("");
        let root_key = ExtendedPrivKey::new_master(Network::Bitcoin, &seed)?;
        let path = DerivationPath::from_str("m/44'/1237'/0'/0/0")?;
        let secp = Secp256k1::new();
        let child_xprv = root_key.derive_priv(&secp, &path)?;
        // Convert from
        // secp256k1::SecretKey (from the secp256k1 version in the bitcoin dependency) into
        // secp256k1::SecretKey (from the secp256k1 version in the direct secp256k1 dependency)
        let sk = secp256k1::SecretKey::from_slice(&child_xprv.private_key.secret_bytes())?;
        Ok(Self::new(sk))
    }
}

impl GenerateMnemonic for Keys {
    type Err = anyhow::Error;

    fn generate_mnemonic(word_count: usize) -> Result<Mnemonic, Self::Err> {
        use crate::util::time;
        use bitcoin::hashes::hmac::{Hmac, HmacEngine};
        use bitcoin::hashes::{sha512, Hash, HashEngine};

        let mut h = HmacEngine::<sha512::Hash>::new(b"nostr");
        let random: [u8; 32] = secp256k1::rand::random();
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

    use anyhow::Result;

    use crate::key::ToBech32;

    #[test]
    fn test_nip06() -> Result<()> {
        let mnemonic: &str = "equal dragon fabric refuse stable cherry smoke allow alley easy never medal attend together lumber movie what sad siege weather matrix buffalo state shoot";
        let keys = Keys::from_mnemonic(mnemonic)?;

        assert_eq!(
            keys.secret_key()?.to_bech32()?,
            "nsec1q6vjgxdgl6ppmkx7q02vxqrpf687a7674ymtwmufjaku4n52a0hq9glmaf".to_string()
        );

        Ok(())
    }
}
