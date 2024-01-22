// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client builder

use std::sync::Arc;

use nostr_database::memory::MemoryDatabase;
use nostr_database::{DynNostrDatabase, IntoNostrDatabase};

use super::signer::ClientSigner;
#[cfg(feature = "nip57")]
use super::zapper::ClientZapper;
use crate::{Client, Options};

/// Client builder
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    /// Client Signer
    pub signer: Option<ClientSigner>,
    /// Client Zapper
    #[cfg(feature = "nip57")]
    pub zapper: Option<ClientZapper>,
    /// Database
    pub database: Arc<DynNostrDatabase>,
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
    /// let builder = ClientBuilder::new().signer(keys);
    ///
    /// let _client: Client = builder.build();
    /// ```
    pub fn signer<S>(mut self, signer: S) -> Self
    where
        S: Into<ClientSigner>,
    {
        self.signer = Some(signer.into());
        self
    }

    /// Set zapper
    #[cfg(feature = "nip57")]
    #[allow(unused_mut, unreachable_code)]
    pub fn zapper<S>(mut self, zapper: S) -> Self
    where
        S: Into<ClientZapper>,
    {
        self.zapper = Some(zapper.into());
        self
    }

    /// Set database
    pub fn database<D>(mut self, database: D) -> Self
    where
        D: IntoNostrDatabase,
    {
        self.database = database.into_nostr_database();
        self
    }

    /// Set opts
    pub fn opts(mut self, opts: Options) -> Self {
        self.opts = opts;
        self
    }

    /// Build [`Client`]
    pub fn build(self) -> Client {
        Client::from_builder(self)
    }
}
