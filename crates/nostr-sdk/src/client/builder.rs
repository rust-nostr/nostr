// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client builder

use std::sync::Arc;

use nostr::signer::{IntoNostrSigner, NostrSigner};
use nostr_database::memory::MemoryDatabase;
use nostr_database::{IntoNostrDatabase, NostrDatabase};
#[cfg(feature = "nip57")]
use nostr_zapper::{DynNostrZapper, IntoNostrZapper};

use crate::{Client, Options};

/// Client builder
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    /// Nostr Signer
    pub signer: Option<Arc<dyn NostrSigner>>,
    /// Nostr Zapper
    #[cfg(feature = "nip57")]
    pub zapper: Option<Arc<DynNostrZapper>>,
    /// Database
    pub database: Arc<dyn NostrDatabase>,
    /// Client options
    pub opts: Options,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            signer: None,
            #[cfg(feature = "nip57")]
            zapper: None,
            database: Arc::new(MemoryDatabase::default()),
            opts: Options::default(),
        }
    }
}

impl ClientBuilder {
    /// New default client builder
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set signer
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// // Signer with private keys
    /// let keys = Keys::generate();
    /// let client = ClientBuilder::new().signer(keys).build();
    /// ```
    #[inline]
    pub fn signer<T>(mut self, signer: T) -> Self
    where
        T: IntoNostrSigner,
    {
        self.signer = Some(signer.into_nostr_signer());
        self
    }

    /// Set zapper
    #[inline]
    #[cfg(feature = "nip57")]
    pub fn zapper<Z>(mut self, zapper: Z) -> Self
    where
        Z: IntoNostrZapper,
    {
        self.zapper = Some(zapper.into_nostr_zapper());
        self
    }

    /// Set database
    #[inline]
    pub fn database<D>(mut self, database: D) -> Self
    where
        D: IntoNostrDatabase,
    {
        self.database = database.into_nostr_database();
        self
    }

    /// Set opts
    #[inline]
    pub fn opts(mut self, opts: Options) -> Self {
        self.opts = opts;
        self
    }

    /// Build [`Client`]
    #[inline]
    pub fn build(self) -> Client {
        Client::from_builder(self)
    }
}
