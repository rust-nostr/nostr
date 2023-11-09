// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Client builder

use std::sync::Arc;

use nostr::Keys;
use nostr_database::memory::MemoryDatabase;
use nostr_database::{DatabaseError, DynNostrDatabase, NostrDatabase};

#[cfg(feature = "nip46")]
use super::RemoteSigner;
use crate::{Client, Options};

/// Client builder
pub struct ClientBuilder {
    pub(super) keys: Keys,
    pub(super) database: Arc<DynNostrDatabase>,
    pub(super) opts: Options,
    #[cfg(feature = "nip46")]
    pub(super) remote_signer: Option<RemoteSigner>,
}

impl ClientBuilder {
    /// New client builder
    pub fn new(keys: &Keys) -> Self {
        Self {
            keys: keys.clone(),
            database: Arc::new(MemoryDatabase::default()),
            opts: Options::default(),
            #[cfg(feature = "nip46")]
            remote_signer: None,
        }
    }

    /// Set database
    pub fn database<D>(mut self, database: D) -> Self
    where
        D: NostrDatabase<Err = DatabaseError> + 'static,
    {
        self.database = Arc::new(database);
        self
    }

    /// Set opts
    pub fn opts(mut self, opts: Options) -> Self {
        self.opts = opts;
        self
    }

    /// Set remote signer
    #[cfg(feature = "nip46")]
    pub fn remote_signer(mut self, remote_signer: RemoteSigner) -> Self {
        self.remote_signer = Some(remote_signer);
        self
    }

    /// Build [`Client`]
    pub fn build(self) -> Client {
        Client::from_builder(self)
    }
}
