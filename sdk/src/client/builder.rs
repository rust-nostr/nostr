// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Client builder

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
#[cfg(feature = "tor")]
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use async_wsocket::ConnectionMode;
use nostr::signer::{IntoNostrSigner, NostrSigner};
use nostr_database::memory::MemoryDatabase;
use nostr_database::{IntoNostrDatabase, NostrDatabase};
use nostr_gossip::{GossipAllowedRelays, IntoNostrGossip, NostrGossip};

use crate::client::Client;
use crate::monitor::Monitor;
use crate::policy::AdmitPolicy;
use crate::prelude::RelayLimits;
use crate::transport::websocket::{
    DefaultWebsocketTransport, IntoWebSocketTransport, WebSocketTransport,
};

const DEFAULT_NOTIFICATION_CHANNEL_SIZE: usize = 4096;

/// Max number of relays to use for gossip
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GossipRelayLimits {
    /// Max number of **read** relays per user (default: 3)
    pub read_relays_per_user: u8,
    /// Max number of **write** relays per user (default: 3)
    pub write_relays_per_user: u8,
    /// Max number of **hint** relays per user (default: 1)
    pub hint_relays_per_user: u8,
    /// Max number of **most used** relays per user (default: 1)
    pub most_used_relays_per_user: u8,
    /// Max number of NIP-17 relays per user (default: 3)
    pub nip17_relays: u8,
}

impl Default for GossipRelayLimits {
    fn default() -> Self {
        Self {
            read_relays_per_user: 3,
            write_relays_per_user: 3,
            hint_relays_per_user: 1,
            most_used_relays_per_user: 1,
            nip17_relays: 3,
        }
    }
}

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
    /// Gossip
    pub gossip: Option<Arc<dyn NostrGossip>>,
    /// Max number of gossip relays to use
    pub gossip_limits: GossipRelayLimits,
    /// Allowed relays during gossip selection
    pub gossip_allowed: GossipAllowedRelays,
    /// Relay monitor
    pub monitor: Option<Monitor>,
    /// Connection
    #[cfg(not(target_arch = "wasm32"))]
    pub connection: Connection,
    /// Max relays allowed in the pool
    pub max_relays: Option<usize>,
    /// Automatic authentication to relays (NIP-42)
    pub automatic_authentication: bool,
    /// Notification channel size
    pub notification_channel_size: usize,
    /// Relay limits
    pub relay_limits: RelayLimits,
    /// Max average latency
    pub max_avg_latency: Option<Duration>,
    /// Sleep when idle
    pub sleep_when_idle: SleepWhenIdle,
    /// Verify subscriptions
    pub verify_subscriptions: bool,
    /// Ban relay on mismatch
    pub ban_relay_on_mismatch: bool,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            signer: None,
            websocket_transport: Arc::new(DefaultWebsocketTransport),
            admit_policy: None,
            database: Arc::new(MemoryDatabase::default()),
            gossip: None,
            gossip_limits: GossipRelayLimits::default(),
            gossip_allowed: GossipAllowedRelays::default(),
            monitor: None,
            #[cfg(not(target_arch = "wasm32"))]
            connection: Connection::default(),
            max_relays: None,
            automatic_authentication: true,
            relay_limits: RelayLimits::default(),
            max_avg_latency: None,
            sleep_when_idle: SleepWhenIdle::default(),
            verify_subscriptions: false,
            ban_relay_on_mismatch: false,
            notification_channel_size: DEFAULT_NOTIFICATION_CHANNEL_SIZE,
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

    /// Set a gossip database
    #[inline]
    pub fn gossip<T>(mut self, gossip: T) -> Self
    where
        T: IntoNostrGossip,
    {
        self.gossip = Some(gossip.into_nostr_gossip());
        self
    }

    /// Set gossip limits
    #[inline]
    pub fn gossip_limits(mut self, limits: GossipRelayLimits) -> Self {
        self.gossip_limits = limits;
        self
    }

    /// Set allowed relays during gossip selection
    #[inline]
    pub fn gossip_allowed(mut self, allowed: GossipAllowedRelays) -> Self {
        self.gossip_allowed = allowed;
        self
    }

    /// Set monitor
    #[inline]
    pub fn monitor(mut self, monitor: Monitor) -> Self {
        self.monitor = Some(monitor);
        self
    }

    /// Connection mode and target
    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn connection(mut self, connection: Connection) -> Self {
        self.connection = connection;
        self
    }

    /// Max relays allowed in the pool (default: None)
    ///
    /// `None` means no limit.
    #[inline]
    pub fn max_relays(mut self, num: Option<usize>) -> Self {
        self.max_relays = num;
        self
    }

    /// Auto authenticates to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn automatic_authentication(mut self, enabled: bool) -> Self {
        self.automatic_authentication = enabled;
        self
    }

    /// Set relay limits
    #[inline]
    pub fn relay_limits(mut self, limits: RelayLimits) -> Self {
        self.relay_limits = limits;
        self
    }

    /// Set max latency (default: None)
    ///
    /// Relays with an avg. latency greater that this value will be skipped.
    #[inline]
    pub fn max_avg_latency(mut self, max: Duration) -> Self {
        self.max_avg_latency = Some(max);
        self
    }

    /// Set sleep when idle config
    #[inline]
    pub fn sleep_when_idle(mut self, config: SleepWhenIdle) -> Self {
        self.sleep_when_idle = config;
        self
    }

    /// Verify that received events belong to a subscription and match the filter.
    pub fn verify_subscriptions(mut self, enable: bool) -> Self {
        self.verify_subscriptions = enable;
        self
    }

    /// If true, ban a relay when it sends an event that doesn't match the subscription filter.
    pub fn ban_relay_on_mismatch(mut self, ban_relay: bool) -> Self {
        self.ban_relay_on_mismatch = ban_relay;
        self
    }

    /// Notification channel size (default: 4096)
    #[inline]
    pub fn notification_channel_size(mut self, size: usize) -> Self {
        self.notification_channel_size = size;
        self
    }

    /// Build [`Client`]
    #[inline]
    pub fn build(self) -> Client {
        Client::from_builder(self)
    }
}

/// Put relays to sleep when idle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SleepWhenIdle {
    /// Disabled
    #[default]
    Disabled,
    /// Enabled for all relays
    Enabled {
        /// Idle timeout
        ///
        /// After how much time of inactivity put the relay to sleep.
        timeout: Duration,
    },
}

/// Connection target
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ConnectionTarget {
    /// All relays
    #[default]
    All,
    /// Only `.onion` relays
    Onion,
}

/// Connection
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Connection {
    /// Mode
    pub mode: ConnectionMode,
    /// Target
    pub target: ConnectionTarget,
}

#[allow(clippy::derivable_impls)]
#[cfg(not(target_arch = "wasm32"))]
impl Default for Connection {
    fn default() -> Self {
        #[cfg(all(feature = "tor", not(target_os = "android"), not(target_os = "ios")))]
        {
            Self {
                mode: ConnectionMode::tor(),
                target: ConnectionTarget::Onion,
            }
        }

        #[cfg(any(
            not(feature = "tor"),
            all(feature = "tor", any(target_os = "android", target_os = "ios")),
        ))]
        Self {
            mode: ConnectionMode::default(),
            target: ConnectionTarget::default(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Connection {
    /// New default connection config
    #[inline]
    pub fn new() -> Self {
        Self {
            mode: ConnectionMode::default(),
            target: ConnectionTarget::default(),
        }
    }

    /// Set connection mode (default: direct)
    #[inline]
    pub fn mode(mut self, mode: ConnectionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set connection target (default: all)
    #[inline]
    pub fn target(mut self, target: ConnectionTarget) -> Self {
        self.target = target;
        self
    }

    /// Set direct connection
    #[inline]
    pub fn direct(mut self) -> Self {
        self.mode = ConnectionMode::direct();
        self
    }

    /// Set proxy
    #[inline]
    pub fn proxy(mut self, addr: SocketAddr) -> Self {
        self.mode = ConnectionMode::proxy(addr);
        self
    }

    /// Use embedded tor client
    #[inline]
    #[cfg(feature = "tor")]
    pub fn embedded_tor(mut self) -> Self {
        self.mode = ConnectionMode::tor();
        self
    }

    /// Use embedded tor client
    ///
    /// Specify a path where to store data
    #[inline]
    #[cfg(feature = "tor")]
    pub fn embedded_tor_with_path<P>(mut self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.mode = ConnectionMode::tor_with_path(path);
        self
    }
}
