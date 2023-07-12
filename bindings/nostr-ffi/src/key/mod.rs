// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::key::Keys as KeysSdk;
use nostr::nips::nip06::FromMnemonic;
use nostr::nips::nip19::{FromBech32, ToBech32};
use nostr::secp256k1::SecretKey;

mod public_key;

pub use self::public_key::PublicKey;
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

impl From<KeysSdk> for Keys {
    fn from(keys: KeysSdk) -> Self {
        Self { keys }
    }
}

impl Keys {
    pub fn new(sk: String) -> Result<Self> {
        let sk = SecretKey::from_str(&sk)?;

        Ok(Self {
            keys: KeysSdk::new(sk),
        })
    }

    pub fn from_public_key(pk: Arc<PublicKey>) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::from_public_key(*pk.as_ref().deref()),
        })
    }

    pub fn from_bech32(sk: String) -> Result<Self> {
        let sk = SecretKey::from_bech32(sk)?;
        Ok(Self {
            keys: KeysSdk::new(sk),
        })
    }

    pub fn generate() -> Self {
        Self {
            keys: KeysSdk::generate(),
        }
    }

    pub fn from_mnemonic(mnemonic: String, passphrase: Option<String>) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::from_mnemonic(mnemonic, passphrase)
                .map_err(|e| NostrError::Generic { err: e.to_string() })?,
        })
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.keys.public_key().into())
    }

    pub fn secret_key(&self) -> Result<String> {
        Ok(self.keys.secret_key()?.display_secret().to_string())
    }

    pub fn secret_key_bech32(&self) -> Result<String> {
        Ok(self.keys.secret_key()?.to_bech32()?)
    }
}
