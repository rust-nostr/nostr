//! Relay builder

use std::sync::Arc;

use nostr::signer::{IntoNostrSigner, NostrSigner};
use nostr::RelayUrl;
use nostr_database::{IntoNostrDatabase, MemoryDatabase, NostrDatabase};

use super::options::RelayOptions;
use super::{Relay, RelayCapabilities};
use crate::policy::AdmitPolicy;
use crate::transport::websocket::{DefaultWebsocketTransport, WebSocketTransport};

/// Relay builder
#[derive(Debug, Clone)]
pub struct RelayBuilder {
    /// Relay URL
    pub url: RelayUrl,
    /// WebSocket transport
    pub websocket_transport: Arc<dyn WebSocketTransport>,
    /// Nostr Signer
    pub signer: Option<Arc<dyn NostrSigner>>,
    /// Database
    pub database: Arc<dyn NostrDatabase>,
    /// Admission policy
    pub admit_policy: Option<Arc<dyn AdmitPolicy>>,
    /// Capabilities
    pub capabilities: RelayCapabilities,
    /// Relay pool options
    pub opts: RelayOptions,
}

impl RelayBuilder {
    /// New relay builder
    #[inline]
    pub fn new(url: RelayUrl) -> Self {
        Self {
            url,
            websocket_transport: Arc::new(DefaultWebsocketTransport),
            signer: None,
            database: Arc::new(MemoryDatabase::default()),
            admit_policy: None,
            capabilities: RelayCapabilities::default(),
            opts: RelayOptions::default(),
        }
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

    /// Set a signer
    #[inline]
    pub fn signer<T>(mut self, signer: T) -> Self
    where
        T: IntoNostrSigner,
    {
        self.signer = Some(signer.into_nostr_signer());
        self
    }

    /// Set a database
    #[inline]
    pub fn database<T>(mut self, database: T) -> Self
    where
        T: IntoNostrDatabase,
    {
        self.database = database.into_nostr_database();
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

    /// Set capabilities
    #[inline]
    pub fn capabilities(mut self, capabilities: RelayCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Set options
    #[inline]
    pub fn opts(mut self, opts: RelayOptions) -> Self {
        self.opts = opts;
        self
    }

    /// Build relay
    #[inline]
    pub fn build(self) -> Relay {
        Relay::from_builder(self)
    }
}
