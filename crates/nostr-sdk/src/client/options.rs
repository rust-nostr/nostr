// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
#[cfg(all(feature = "tor", any(target_os = "android", target_os = "ios")))]
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use nostr_relay_pool::prelude::*;

/// Options
#[derive(Debug, Clone)]
pub struct Options {
    pub(super) autoconnect: bool,
    new_events_difficulty: Arc<AtomicU8>,
    min_pow_difficulty: Arc<AtomicU8>,
    pub(super) req_filters_chunk_size: u8,
    pub(super) timeout: Duration,
    nip42_auto_authentication: Arc<AtomicBool>,
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
            new_events_difficulty: Arc::new(AtomicU8::new(0)),
            min_pow_difficulty: Arc::new(AtomicU8::new(0)),
            req_filters_chunk_size: 10,
            timeout: Duration::from_secs(60),
            nip42_auto_authentication: Arc::new(AtomicBool::new(true)),
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

    /// Set default POW difficulty for `Event`
    #[inline]
    pub fn difficulty(mut self, difficulty: u8) -> Self {
        self.new_events_difficulty = Arc::new(AtomicU8::new(difficulty));
        self
    }

    #[inline]
    pub(crate) fn get_difficulty(&self) -> u8 {
        self.new_events_difficulty.load(Ordering::SeqCst)
    }

    #[inline]
    pub(crate) fn update_difficulty(&self, difficulty: u8) {
        self.new_events_difficulty
            .store(difficulty, Ordering::SeqCst);
    }

    /// Minimum POW difficulty for received events
    #[inline]
    pub fn min_pow(mut self, difficulty: u8) -> Self {
        self.min_pow_difficulty = Arc::new(AtomicU8::new(difficulty));
        self
    }

    #[inline]
    pub(crate) fn get_min_pow_difficulty(&self) -> u8 {
        self.min_pow_difficulty.load(Ordering::SeqCst)
    }

    /// Update minimum POW difficulty for received events
    #[inline]
    pub fn update_min_pow_difficulty(&self, difficulty: u8) {
        self.min_pow_difficulty.store(difficulty, Ordering::SeqCst);
    }

    /// REQ filters chunk size (default: 10)
    #[inline]
    pub fn req_filters_chunk_size(mut self, size: u8) -> Self {
        self.req_filters_chunk_size = size;
        self
    }

    /// Timeout (default: 60)
    ///
    /// Used in `fetch_events` and similar methods as default timeout.
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Relay connection timeout (default: None)
    ///
    /// If set to `None`, the client will try to connect to relay without waiting.
    #[deprecated(since = "0.37.0")]
    pub fn connection_timeout(self, _timeout: Option<Duration>) -> Self {
        self
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn automatic_authentication(mut self, enabled: bool) -> Self {
        self.nip42_auto_authentication = Arc::new(AtomicBool::new(enabled));
        self
    }

    /// Enable gossip model (default: false)
    #[inline]
    pub fn gossip(mut self, enable: bool) -> Self {
        self.gossip = enable;
        self
    }

    #[inline]
    pub(super) fn is_nip42_auto_authentication_enabled(&self) -> bool {
        self.nip42_auto_authentication.load(Ordering::SeqCst)
    }

    #[inline]
    pub(super) fn update_automatic_authentication(&self, enabled: bool) {
        self.nip42_auto_authentication
            .store(enabled, Ordering::SeqCst);
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

    /// Set filtering mode (default: blacklist)
    #[inline]
    pub fn filtering_mode(mut self, mode: RelayFilteringMode) -> Self {
        self.pool = self.pool.filtering_mode(mode);
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
    #[cfg(all(feature = "tor", not(target_os = "android"), not(target_os = "ios")))]
    pub fn embedded_tor(mut self) -> Self {
        self.mode = ConnectionMode::tor();
        self
    }

    /// Use embedded tor client
    #[inline]
    #[cfg(all(feature = "tor", any(target_os = "android", target_os = "ios")))]
    pub fn embedded_tor<P>(mut self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.mode = ConnectionMode::tor(path.as_ref().to_path_buf());
        self
    }
}
