// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use anyhow::Result;
use flutter_rust_bridge::frb;
use nostr_sdk::prelude::*;

pub mod public_key;
pub mod secret_key;

use self::public_key::_PublicKey;
use self::secret_key::_SecretKey;

#[frb(name = "Keys")]
pub struct _Keys {
    inner: Keys,
}

#[frb(sync)]
impl _Keys {
    pub fn new(secret_key: _SecretKey) -> Self {
        Self {
            inner: Keys::new(secret_key.into()),
        }
    }

    /// Generate random keys
    ///
    /// This constructor use a random number generator that retrieves randomness from the operating system.
    pub fn generate() -> Self {
        Self {
            inner: Keys::generate(),
        }
    }

    /// Parse secret key from `hex` or `bech32`
    pub fn parse(secret_key: &str) -> Result<Self> {
        Ok(Self {
            inner: Keys::parse(secret_key)?,
        })
    }

    pub fn public_key(&self) -> _PublicKey {
        self.inner.public_key().into()
    }

    pub fn secret_key(&self) -> _SecretKey {
        self.inner.secret_key().clone().into()
    }
}
