// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

//! Nostr Keyring

// * Save in OS keyring?
// * Save in ~/.nostr/accounts.dat or ~/.nostr/keys.dat

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use nostr::prelude::*;
use thiserror::Error;

mod constants;
mod dat;
mod dir;

use crate::dat::AccountKey;

/// Nostr Keyring error
#[derive(Debug, Error)]
pub enum Error {
    /// Dir error
    #[error(transparent)]
    Dir(#[from] dir::Error),
    /// Can't get home directory
    #[error("Can't get home directory")]
    CantGetHomeDir,
}

/// Nostr Keyring version
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    #[default]
    V1,
}

/// Nostr Keyring
pub struct NostrKeyring {
    path: PathBuf,
    version: Version,
    accounts: BTreeMap<String, AccountKey>,
}

impl NostrKeyring {
    /// Get list of available accounts
    #[cfg(not(all(target_os = "android", target_os = "ios")))]
    pub fn open() -> Result<Self, Error> {
        let home_dir: PathBuf = dirs::home_dir().ok_or(Error::CantGetHomeDir)?;
        todo!()
    }

    /// Open Nostr Keyring from custom path
    ///
    /// Automatically create it if not exists.
    pub fn open_in<P>(base_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        todo!()
    }

    /// Get keyring version
    #[inline(always)]
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get list of available accounts
    #[inline(always)]
    pub fn accounts(&self) -> &BTreeMap<String, AccountKey> {
        &self.accounts
    }

    /// Add account
    #[inline(always)]
    pub fn add_account(&mut self, name: String, key: AccountKey) {
        self.accounts.insert(name, key);
    }

    /// Remove account from keyring
    #[inline(always)]
    pub fn remove_account(&mut self, name: &str) {
        self.accounts.remove(name);
    }

    /// Write keyring to file
    pub fn save(&self) -> Result<(), Error> {
        todo!()
    }
}
