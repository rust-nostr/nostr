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

#[cfg(feature = "async")]
use async_utility::{task, tokio};
pub use keyring::{Entry, Error as KeyringError};
use nostr::{key, Keys, SecretKey};

pub mod prelude;

/// Keyring error
#[derive(Debug)]
pub enum Error {
    /// Join error
    #[cfg(feature = "async")]
    Join(tokio::task::JoinError),
    /// Keyring error
    Keyring(KeyringError),
    /// Nostr keys error
    Keys(key::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "async")]
            Self::Join(e) => write!(f, "{e}"),
            Self::Keyring(e) => write!(f, "{e}"),
            Self::Keys(e) => write!(f, "{e}"),
        }
    }
}

#[cfg(feature = "async")]
impl From<tokio::task::JoinError> for Error {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::Join(e)
    }
}

impl From<KeyringError> for Error {
    fn from(e: KeyringError) -> Self {
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

    #[inline]
    fn entry(&self, name: &str) -> Result<Entry, Error> {
        // This internally uses a std RwLock, which should never block since always just reads.
        // The write lock is acquired only in case you are setting a custom store.
        Ok(Entry::new(&self.service, name)?)
    }

    /// Save a [`Keys`] into the keyring
    pub fn set(&self, name: &str, keys: &Keys) -> Result<(), Error> {
        let entry: Entry = self.entry(name)?;
        let secret_bytes: &[u8] = keys.secret_key().as_secret_bytes();
        entry.set_secret(secret_bytes)?;
        Ok(())
    }

    /// Asynchronously save a [`Keys`] into the keyring
    #[cfg(feature = "async")]
    pub async fn set_async(&self, name: &str, keys: &Keys) -> Result<(), Error> {
        let entry: Entry = self.entry(name)?;
        let secret_bytes: [u8; 32] = keys.secret_key().to_secret_bytes();
        task::spawn_blocking(move || entry.set_secret(&secret_bytes)).await??;
        Ok(())
    }

    /// Get the [`Keys`] from the keyring
    pub fn get(&self, name: &str) -> Result<Keys, Error> {
        let entry: Entry = self.entry(name)?;
        let secret: Vec<u8> = entry.get_secret()?;
        let secret_key: SecretKey = SecretKey::from_slice(&secret)?;
        Ok(Keys::new(secret_key))
    }

    /// Asynchronously get the [`Keys`] from the keyring
    #[cfg(feature = "async")]
    pub async fn get_async(&self, name: &str) -> Result<Keys, Error> {
        let entry: Entry = self.entry(name)?;
        let secret: Vec<u8> = task::spawn_blocking(move || entry.get_secret()).await??;
        let secret_key: SecretKey = SecretKey::from_slice(&secret)?;
        Ok(Keys::new(secret_key))
    }

    /// Delete the [`Keys`] from the keyring
    pub fn delete(&self, name: &str) -> Result<(), Error> {
        let entry: Entry = self.entry(name)?;
        entry.delete_credential()?;
        Ok(())
    }

    /// Asynchronously delete the [`Keys`] from the keyring
    #[cfg(feature = "async")]
    pub async fn delete_async(&self, name: &str) -> Result<(), Error> {
        let entry: Entry = self.entry(name)?;
        task::spawn_blocking(move || entry.delete_credential()).await??;
        Ok(())
    }
}
