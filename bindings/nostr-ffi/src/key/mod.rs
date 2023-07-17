// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::key::{FromPkStr, FromSkStr, Keys as KeysSdk};
use nostr::nips::nip06::FromMnemonic;

mod public_key;
mod secret_key;

pub use self::public_key::PublicKey;
pub use self::secret_key::SecretKey;
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
    pub fn new(sk: Arc<SecretKey>) -> Self {
        Self {
            keys: KeysSdk::new(**sk),
        }
    }

    pub fn from_public_key(pk: Arc<PublicKey>) -> Self {
        Self {
            keys: KeysSdk::from_public_key(*pk.as_ref().deref()),
        }
    }

    pub fn from_sk_str(sk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::from_sk_str(&sk)?,
        })
    }

    pub fn from_pk_str(pk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::from_pk_str(&pk)?,
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

    pub fn secret_key(&self) -> Result<Arc<SecretKey>> {
        Ok(Arc::new(self.keys.secret_key()?.into()))
    }
}
