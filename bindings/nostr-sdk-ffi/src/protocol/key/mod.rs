// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::key;
use nostr::nips::nip06::FromMnemonic;
use nostr::secp256k1::Message;
use uniffi::Object;

mod public_key;
mod secret_key;

pub use self::public_key::PublicKey;
pub use self::secret_key::SecretKey;
use crate::error::Result;

/// Nostr keys
#[derive(Debug, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
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
    /// Initialize nostr keys from secret key.
    #[uniffi::constructor]
    pub fn new(secret_key: &SecretKey) -> Self {
        Self {
            inner: key::Keys::new(secret_key.deref().clone()),
        }
    }

    /// Parse secret key from `hex` or `bech32` and compose keys
    #[uniffi::constructor]
    pub fn parse(secret_key: &str) -> Result<Self> {
        Ok(Self {
            inner: key::Keys::parse(secret_key)?,
        })
    }

    /// Generate random keys
    ///
    /// This constructor use a random number generator that retrieves randomness from the operating system.
    ///
    /// Generate random keys **without** construct the `Keypair`.
    /// This allows faster keys generation (i.e. for vanity pubkey mining).
    /// The `Keypair` will be automatically created when needed and stored in a cell.
    #[uniffi::constructor]
    pub fn generate() -> Self {
        Self {
            inner: key::Keys::generate(),
        }
    }

    /// Derive keys from BIP-39 mnemonics (ENGLISH wordlist).
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/06.md>
    #[uniffi::constructor(default(passphrase = None, account = None, typ = None, index = None))]
    pub fn from_mnemonic(
        mnemonic: String,
        passphrase: Option<String>,
        account: Option<u32>,
        typ: Option<u32>,
        index: Option<u32>,
    ) -> Result<Self> {
        Ok(Self {
            inner: key::Keys::from_mnemonic_advanced(mnemonic, passphrase, account, typ, index)?,
        })
    }

    /// Get public key
    pub fn public_key(&self) -> PublicKey {
        self.inner.public_key().into()
    }

    /// Get secret key
    pub fn secret_key(&self) -> SecretKey {
        self.inner.secret_key().clone().into()
    }

    /// Creates a schnorr signature of a message.
    ///
    /// This method use a random number generator that retrieves randomness from the operating system.
    pub fn sign_schnorr(&self, message: &[u8]) -> Result<String> {
        let message: Message = Message::from_digest_slice(message)?;
        Ok(self.inner.sign_schnorr(&message).to_string())
    }
}
