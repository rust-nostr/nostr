// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
#[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
use std::path::Path;
use std::time::Duration;

use async_wsocket::ConnectionMode;
use nostr_gossip::GossipAllowedRelays;

use crate::pool::RelayPoolOptions;
use crate::relay::RelayLimits;

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

/// Gossip options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GossipOptions {
    /// Max number of relays to use
    pub limits: GossipRelayLimits,
    /// Allowed relay during selection
    pub allowed: GossipAllowedRelays,
}

impl GossipOptions {
    /// Set limits
    #[inline]
    pub fn limits(mut self, limits: GossipRelayLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Set allowed
    #[inline]
    pub fn allowed(mut self, allowed: GossipAllowedRelays) -> Self {
        self.allowed = allowed;
        self
    }
}

/// Options
#[derive(Debug, Clone, Default)]
pub struct ClientOptions {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) connection: Connection,
    pub(super) relay_limits: RelayLimits,
    pub(super) max_avg_latency: Option<Duration>,
    pub(super) sleep_when_idle: SleepWhenIdle,
    pub(super) verify_subscriptions: bool,
    pub(super) ban_relay_on_mismatch: bool,
    pub(super) gossip: GossipOptions,
    pub(super) pool: RelayPoolOptions,
}

impl ClientOptions {
    /// Create new default options
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn automatic_authentication(mut self, enabled: bool) -> Self {
        self.pool = self.pool.automatic_authentication(enabled);
        self
    }

    /// Connection mode and target
    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn connection(mut self, connection: Connection) -> Self {
        self.connection = connection;
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

    /// Set gossip options
    #[inline]
    pub fn gossip(mut self, opts: GossipOptions) -> Self {
        self.gossip = opts;
        self
    }

    /// Set relay pool options
    #[inline]
    pub fn pool(mut self, opts: RelayPoolOptions) -> Self {
        self.pool = opts;
        self
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
