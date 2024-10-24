// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use anyhow::Result;
use flutter_rust_bridge::frb;
use nostr_sdk::prelude::*;

#[frb(name = "Keys")]
pub struct _Keys {
    inner: Keys,
}

impl _Keys {
    /// Generate random keys
    ///
    /// This constructor use a random number generator that retrieves randomness from the operating system.
    #[frb(sync)]
    pub fn generate() -> Self {
        Self {
            inner: Keys::generate(),
        }
    }

    /// Parse secret key from `hex` or `bech32`
    #[frb(sync)]
    pub fn parse(secret_key: &str) -> Result<Self> {
        Ok(Self {
            inner: Keys::parse(secret_key)?,
        })
    }

    // TODO: add PublicKey struct
    pub fn public_key(&self) -> String {
        self.inner.public_key().to_string()
    }

    // TODO: add SecretKey struct
    pub fn secret_key(&self) -> String {
        self.inner.secret_key().to_secret_hex()
    }
}
