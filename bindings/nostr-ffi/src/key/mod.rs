// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::key;
use nostr::nips::nip06::FromMnemonic;
use nostr::secp256k1::Message;
use uniffi::Object;

mod public_key;
mod secret_key;

pub use self::public_key::PublicKey;
pub use self::secret_key::SecretKey;
use crate::error::{NostrError, Result};

#[derive(Clone, Object)]
pub struct Keys {
    inner: key::Keys,
}

impl Deref for Keys {
    type Target = key::Keys;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<key::Keys> for Keys {
    fn from(inner: key::Keys) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Keys {
    #[uniffi::constructor]
    pub fn new(sk: Arc<SecretKey>) -> Self {
        Self {
            inner: key::Keys::new(sk.as_ref().deref().clone()),
        }
    }

    /// Try to parse keys from **secret key** `hex` or `bech32`
    #[uniffi::constructor]
    pub fn parse(secret_key: String) -> Result<Self> {
        Ok(Self {
            inner: key::Keys::parse(secret_key)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_public_key(pk: Arc<PublicKey>) -> Self {
        Self {
            inner: key::Keys::from_public_key(**pk),
        }
    }

    #[uniffi::constructor]
    pub fn generate() -> Self {
        Self {
            inner: key::Keys::generate(),
        }
    }

    #[uniffi::constructor]
    pub fn vanity(prefixes: Vec<String>, bech32: bool, num_cores: u8) -> Result<Self> {
        Ok(Self {
            inner: key::Keys::vanity(prefixes, bech32, num_cores as usize)?,
        })
    }

    /// Derive `Keys` from BIP-39 mnemonics (ENGLISH wordlist).
    ///
    /// By default no passphrase is used and account is set to `0`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/06.md>
    #[uniffi::constructor]
    pub fn from_mnemonic(
        mnemonic: String,
        passphrase: Option<String>,
        account: Option<u32>,
    ) -> Result<Self> {
        Ok(Self {
            inner: key::Keys::from_mnemonic_with_account(mnemonic, passphrase, account)
                .map_err(|e| NostrError::Generic(e.to_string()))?,
        })
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.public_key().into())
    }

    pub fn secret_key(&self) -> Result<Arc<SecretKey>> {
        Ok(Arc::new(self.inner.secret_key()?.clone().into()))
    }

    pub fn sign_schnorr(&self, message: Vec<u8>) -> Result<String> {
        let message = Message::from_slice(&message)?;
        Ok(self.inner.sign_schnorr(&message)?.to_string())
    }
}
