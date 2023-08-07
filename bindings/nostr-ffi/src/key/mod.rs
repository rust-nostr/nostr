// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::key::{FromPkStr, FromSkStr, Keys as KeysSdk};
use nostr::nips::nip06::FromMnemonic;
use uniffi::Object;

mod public_key;
mod secret_key;

pub use self::public_key::PublicKey;
pub use self::secret_key::SecretKey;
use crate::error::{NostrError, Result};

#[derive(Clone, Object)]
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

#[uniffi::export]
impl Keys {
    #[uniffi::constructor]
    pub fn new(sk: Arc<SecretKey>) -> Arc<Self> {
        Arc::new(Self {
            keys: KeysSdk::new(**sk),
        })
    }

    #[uniffi::constructor]
    pub fn from_public_key(pk: Arc<PublicKey>) -> Arc<Self> {
        Arc::new(Self {
            keys: KeysSdk::from_public_key(*pk.as_ref().deref()),
        })
    }

    #[uniffi::constructor]
    pub fn from_sk_str(sk: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            keys: KeysSdk::from_sk_str(&sk)?,
        }))
    }

    #[uniffi::constructor]
    pub fn from_pk_str(pk: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            keys: KeysSdk::from_pk_str(&pk)?,
        }))
    }

    #[uniffi::constructor]
    pub fn generate() -> Arc<Self> {
        Arc::new(Self {
            keys: KeysSdk::generate(),
        })
    }

    #[uniffi::constructor]
    pub fn vanity(prefixes: Vec<String>, bech32: bool, num_cores: u8) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            keys: KeysSdk::vanity(prefixes, bech32, num_cores as usize)?,
        }))
    }

    #[uniffi::constructor]
    pub fn from_mnemonic(mnemonic: String, passphrase: Option<String>) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            keys: KeysSdk::from_mnemonic(mnemonic, passphrase)
                .map_err(|e| NostrError::Generic { err: e.to_string() })?,
        }))
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.keys.public_key().into())
    }

    pub fn secret_key(&self) -> Result<Arc<SecretKey>> {
        Ok(Arc::new(self.keys.secret_key()?.into()))
    }
}
