// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use anyhow::Result;
use nostr::key::{FromBech32, Keys as KeysSdk, XOnlyPublicKey};
use nostr::util::nips::nip06::FromMnemonic;
use secp256k1::SecretKey;

#[derive(Clone)]
pub struct Keys {
    keys: KeysSdk,
}

impl Deref for Keys {
    type Target = KeysSdk;
    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

impl Keys {
    pub fn new(sk: String) -> Result<Self> {
        let sk = SecretKey::from_str(&sk)?;

        Ok(Self {
            keys: KeysSdk::new(sk),
        })
    }

    pub fn from_public_key(pk: String) -> Result<Self> {
        let public_key = XOnlyPublicKey::from_str(&pk)?;

        Ok(Self {
            keys: KeysSdk::from_public_key(public_key),
        })
    }

    pub fn from_bech32_public_key(pk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::from_bech32_public_key(&pk)?,
        })
    }

    pub fn from_bech32(sk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::from_bech32(&sk)?,
        })
    }

    pub fn generate_from_os_random() -> Self {
        Self {
            keys: KeysSdk::generate_from_os_random(),
        }
    }

    pub fn from_mnemonic(mnemonic: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::from_mnemonic(mnemonic.as_str())?,
        })
    }

    pub fn public_key(&self) -> String {
        self.keys.public_key_as_str()
    }

    pub fn secret_key(&self) -> Result<String> {
        Ok(self.keys.secret_key_as_str()?)
    }
}
