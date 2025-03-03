// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool builder

use std::sync::Arc;

use nostr::NostrSigner;
use nostr_database::{MemoryDatabase, NostrDatabase};

use super::options::RelayPoolOptions;
use super::RelayPool;
use crate::middleware::{AdmitPolicy, AuthenticationLayer};
use crate::transport::websocket::{DefaultWebsocketTransport, WebSocketTransport};

/// Relay Pool builder
#[derive(Debug, Clone)]
pub struct RelayPoolBuilder {
    /// WebSocket transport
    pub websocket_transport: Arc<dyn WebSocketTransport>,
    /// Admission policy
    pub admit_policy: Option<Arc<dyn AdmitPolicy>>,
    /// Authentication layer
    pub authentication_layer: Option<Arc<dyn AuthenticationLayer>>,
    /// Relay pool options
    pub opts: RelayPoolOptions,
    // Private stuff
    #[doc(hidden)]
    pub __database: Arc<dyn NostrDatabase>,
    #[doc(hidden)]
    pub __signer: Option<Arc<dyn NostrSigner>>,
}

impl Default for RelayPoolBuilder {
    fn default() -> Self {
        Self {
            websocket_transport: Arc::new(DefaultWebsocketTransport),
            admit_policy: None,
            authentication_layer: None,
            opts: RelayPoolOptions::default(),
            __database: Arc::new(MemoryDatabase::default()),
            __signer: None,
        }
    }
}

impl RelayPoolBuilder {
    /// New default builder
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a WebSocket transport
    #[inline]
    pub fn websocket_transport<T>(mut self, transport: T) -> Self
    where
        T: WebSocketTransport + 'static,
    {
        self.websocket_transport = Arc::new(transport);
        self
    }

    /// Admission policy
    #[inline]
    pub fn admit_policy<T>(mut self, policy: T) -> Self
    where
        T: AdmitPolicy + 'static,
    {
        self.admit_policy = Some(Arc::new(policy));
        self
    }

    /// Authentication layer
    #[inline]
    pub fn authentication_layer<T>(mut self, layer: T) -> Self
    where
        T: AuthenticationLayer + 'static,
    {
        self.authentication_layer = Some(Arc::new(layer));
        self
    }

    /// Set options
    #[inline]
    pub fn opts(mut self, opts: RelayPoolOptions) -> Self {
        self.opts = opts;
        self
    }

    /// Build relay pool
    #[inline]
    pub fn build(self) -> RelayPool {
        RelayPool::from_builder(self)
    }
}
