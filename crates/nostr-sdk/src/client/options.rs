// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
#[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
use std::path::Path;
use std::time::Duration;

use nostr_relay_pool::prelude::*;

/// Options
#[derive(Debug, Clone)]
pub struct Options {
    pub(super) autoconnect: bool,
    pub(super) min_pow_difficulty: u8,
    pub(super) nip42_auto_authentication: bool,
    pub(super) gossip: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) connection: Connection,
    pub(super) relay_limits: RelayLimits,
    pub(super) max_avg_latency: Option<Duration>,
    pub(super) pool: RelayPoolOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            autoconnect: false,
            min_pow_difficulty: 0,
            nip42_auto_authentication: true,
            gossip: false,
            #[cfg(not(target_arch = "wasm32"))]
            connection: Connection::default(),
            relay_limits: RelayLimits::default(),
            max_avg_latency: None,
            pool: RelayPoolOptions::default(),
        }
    }
}

impl Options {
    /// Create new (default) [`Options`]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Automatically start connection with relays (default: false)
    ///
    /// When set to `true`, there isn't the need of calling the connect methods.
    #[inline]
    pub fn autoconnect(mut self, val: bool) -> Self {
        self.autoconnect = val;
        self
    }

    /// Minimum POW difficulty for received events (default: 0)
    #[inline]
    pub fn min_pow(mut self, difficulty: u8) -> Self {
        self.min_pow_difficulty = difficulty;
        self
    }

    /// REQ filters chunk size (default: 10)
    #[deprecated(since = "0.39.0")]
    pub fn req_filters_chunk_size(self, _size: u8) -> Self {
        self
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn automatic_authentication(mut self, enabled: bool) -> Self {
        self.nip42_auto_authentication = enabled;
        self
    }

    /// Enable gossip model (default: false)
    #[inline]
    pub fn gossip(mut self, enable: bool) -> Self {
        self.gossip = enable;
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

    /// Notification channel size (default: [`DEFAULT_NOTIFICATION_CHANNEL_SIZE`])
    #[inline]
    pub fn notification_channel_size(mut self, size: usize) -> Self {
        self.pool = self.pool.notification_channel_size(size);
        self
    }
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
