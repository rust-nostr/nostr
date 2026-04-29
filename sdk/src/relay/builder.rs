//! Relay builder

use std::sync::Arc;

use nostr::RelayUrl;
use nostr_database::{IntoNostrDatabase, NostrDatabase};

use super::options::RelayOptions;
use super::{Relay, RelayCapabilities};
use crate::authenticator::Authenticator;
use crate::events_tracker::MemoryEventsTracker;
use crate::policy::AdmitPolicy;
use crate::transport::websocket::{DefaultWebsocketTransport, WebSocketTransport};

/// Relay builder
#[derive(Debug, Clone)]
pub struct RelayBuilder {
    /// Relay URL
    pub url: RelayUrl,
    /// WebSocket transport
    pub websocket_transport: Arc<dyn WebSocketTransport>,
    /// Database
    pub database: Arc<dyn NostrDatabase>,
    /// Admission policy
    pub admit_policy: Option<Arc<dyn AdmitPolicy>>,
    /// Authenticator
    pub authenticator: Option<Arc<dyn Authenticator>>,
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
            database: Arc::new(MemoryEventsTracker::default()),
            admit_policy: None,
            authenticator: None,
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

    /// Set a NIP-42 authenticator.
    ///
    /// The authenticator is used when a relay requires authentication and the
    /// client needs to build an `AUTH` event.
    ///
    /// If you already have a signer that implements
    /// [`AsyncGetPublicKey`](nostr::signer::AsyncGetPublicKey) and
    /// [`AsyncSignEvent`](nostr::signer::AsyncSignEvent), you can wrap it with
    /// [`SignerAuthenticator`](crate::authenticator::SignerAuthenticator).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nostr_sdk::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let keys = Keys::generate();
    /// let authenticator = SignerAuthenticator::new(keys);
    /// let url = RelayUrl::parse("wss://relay.damus.io")?;
    /// let client = Relay::builder(url).authenticator(authenticator).build();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn authenticator<T>(mut self, authenticator: T) -> Self
    where
        T: Authenticator + 'static,
    {
        self.authenticator = Some(Arc::new(authenticator));
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
