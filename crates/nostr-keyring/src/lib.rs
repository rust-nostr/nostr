// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Keyring

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

use std::fmt;

use keyring::Entry;
use nostr::{key, Keys, SecretKey};

pub mod prelude;

/// Keyring error
#[derive(Debug)]
pub enum Error {
    /// Keyring error
    Keyring(keyring::Error),
    /// Nostr keys error
    Keys(key::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyring(e) => write!(f, "{e}"),
            Self::Keys(e) => write!(f, "{e}"),
        }
    }
}

impl From<keyring::Error> for Error {
    fn from(e: keyring::Error) -> Self {
        Self::Keyring(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

/// Nostr keyring
#[derive(Debug, Clone)]
pub struct NostrKeyring {
    service: String,
}

impl NostrKeyring {
    /// Construct keyring for service
    pub fn new<S>(service: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            service: service.into(),
        }
    }

    /// Save a [`Keys`] into the keyring
    pub fn set(&self, name: &str, keys: &Keys) -> Result<(), Error> {
        let entry: Entry = Entry::new(&self.service, name)?;
        entry.set_secret(keys.secret_key().as_secret_bytes())?;
        Ok(())
    }

    /// Get the [`Keys`] from the keyring
    pub fn get(&self, name: &str) -> Result<Keys, Error> {
        let entry: Entry = Entry::new(&self.service, name)?;
        let secret: Vec<u8> = entry.get_secret()?;
        let secret_key: SecretKey = SecretKey::from_slice(&secret)?;
        Ok(Keys::new(secret_key))
    }

    /// Delete the [`Keys`] from the keyring
    pub fn delete(&self, name: &str) -> Result<(), Error> {
        let entry: Entry = Entry::new(&self.service, name)?;
        entry.delete_credential()?;
        Ok(())
    }
}
