// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use nostr::key::{Keys as KeysSdk, XOnlyPublicKey};
use nostr::secp256k1::SecretKey;
use nostr::util::nips::nip06::FromMnemonic;
use nostr::util::nips::nip19::{FromBech32, ToBech32};

use crate::error::{NostrError, Result};

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

    pub fn from_bech32(sk: String) -> Result<Self> {
        let sk = SecretKey::from_bech32(sk)?;
        Ok(Self {
            keys: KeysSdk::new(sk),
        })
    }

    pub fn from_bech32_public_key(pk: String) -> Result<Self> {
        let pk = XOnlyPublicKey::from_bech32(pk)?;
        Ok(Self {
            keys: KeysSdk::from_public_key(pk),
        })
    }

    pub fn generate_from_os_random() -> Self {
        Self {
            keys: KeysSdk::generate_from_os_random(),
        }
    }

    pub fn from_mnemonic(mnemonic: String, passphrase: Option<String>) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::from_mnemonic(mnemonic, passphrase)
                .map_err(|e| NostrError::Generic { err: e.to_string() })?,
        })
    }

    pub fn public_key(&self) -> String {
        self.keys.public_key().to_string()
    }

    pub fn public_key_bech32(&self) -> Result<String> {
        Ok(self.keys.public_key().to_bech32()?)
    }

    pub fn secret_key(&self) -> Result<String> {
        Ok(self.keys.secret_key()?.display_secret().to_string())
    }

    pub fn secret_key_bech32(&self) -> Result<String> {
        Ok(self.keys.secret_key()?.to_bech32()?)
    }
}
