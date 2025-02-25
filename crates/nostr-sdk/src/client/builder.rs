// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Client builder

use std::sync::Arc;

use nostr::signer::{IntoNostrSigner, NostrSigner};
use nostr_database::memory::MemoryDatabase;
use nostr_database::{IntoNostrDatabase, NostrDatabase};
use nostr_relay_pool::policy::AdmitPolicy;
use nostr_relay_pool::transport::websocket::{
    DefaultWebsocketTransport, IntoWebSocketTransport, WebSocketTransport,
};

use crate::{Client, Options};

/// Client builder
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    /// Nostr Signer
    pub signer: Option<Arc<dyn NostrSigner>>,
    /// WebSocket transport
    pub websocket_transport: Arc<dyn WebSocketTransport>,
    /// Admission policy
    pub admit_policy: Option<Arc<dyn AdmitPolicy>>,
    /// Database
    pub database: Arc<dyn NostrDatabase>,
    /// Client options
    pub opts: Options,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            signer: None,
            websocket_transport: Arc::new(DefaultWebsocketTransport),
            admit_policy: None,
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

    /// Set custom WebSocket transport
    ///
    /// By default [`DefaultWebsocketTransport`] is used.
    #[inline]
    pub fn websocket_transport<T>(mut self, transport: T) -> Self
    where
        T: IntoWebSocketTransport,
    {
        self.websocket_transport = transport.into_transport();
        self
    }

    /// Set an admission policy
    #[inline]
    pub fn admit_policy<T>(mut self, policy: T) -> Self
    where
        T: AdmitPolicy + 'static,
    {
        self.admit_policy = Some(Arc::new(policy));
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
