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
use nostr_relay_pool::relay::options::DEFAULT_SEND_TIMEOUT;

/// Options
#[derive(Debug, Clone)]
pub struct Options {
    wait_for_send: bool,
    wait_for_subscription: bool,
    pub(super) autoconnect: bool,
    new_events_difficulty: Arc<AtomicU8>,
    min_pow_difficulty: Arc<AtomicU8>,
    pub(super) req_filters_chunk_size: u8,
    skip_disconnected_relays: bool,
    pub(super) fetch_policy: FetchPolicy,
    pub(super) connection_timeout: Option<Duration>,
    send_timeout: Option<Duration>,
    nip42_auto_authentication: Arc<AtomicBool>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) connection: Connection,
    pub(super) relay_limits: RelayLimits,
    pub(super) max_avg_latency: Option<Duration>,
    pub(super) pool: RelayPoolOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            wait_for_send: true,
            wait_for_subscription: false,
            autoconnect: false,
            new_events_difficulty: Arc::new(AtomicU8::new(0)),
            min_pow_difficulty: Arc::new(AtomicU8::new(0)),
            req_filters_chunk_size: 10,
            skip_disconnected_relays: true,
            fetch_policy: FetchPolicy::both(),
            connection_timeout: None,
            send_timeout: Some(DEFAULT_SEND_TIMEOUT),
            nip42_auto_authentication: Arc::new(AtomicBool::new(true)),
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

    /// Wait for the msg to be sent (default: true)
    #[inline]
    pub fn wait_for_send(mut self, wait: bool) -> Self {
        self.wait_for_send = wait;
        self
    }

    pub(crate) fn get_wait_for_send(&self) -> RelaySendOptions {
        RelaySendOptions::new()
            .timeout(self.send_timeout)
            .skip_send_confirmation(!self.wait_for_send)
            .skip_disconnected(self.skip_disconnected_relays)
    }

    /// Wait for the subscription msg to be sent (default: false)
    ///
    /// Used in `subscribe` and `unsubscribe` methods
    #[inline]
    pub fn wait_for_subscription(mut self, wait: bool) -> Self {
        self.wait_for_subscription = wait;
        self
    }

    pub(crate) fn get_wait_for_subscription(&self) -> RelaySendOptions {
        RelaySendOptions::new()
            .timeout(self.send_timeout)
            .skip_send_confirmation(!self.wait_for_subscription)
            .skip_disconnected(self.skip_disconnected_relays)
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

    /// Skip disconnected relays during send methods (default: true)
    ///
    /// If the relay made just 1 attempt, the relay will not be skipped
    #[inline]
    pub fn skip_disconnected_relays(mut self, skip: bool) -> Self {
        self.skip_disconnected_relays = skip;
        self
    }

    /// Set a default fetch policy
    #[inline]
    pub fn fetch_policy(mut self, policy: FetchPolicy) -> Self {
        self.fetch_policy = policy;
        self
    }

    /// Timeout (default: 60)
    ///
    /// Used in `get_events_of` and similar methods as default timeout.
    #[deprecated(since = "0.35.0", note = "Use `fetch_policy` instead.")]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.fetch_policy.timeout = timeout;
        self
    }

    /// Relay connection timeout (default: None)
    ///
    /// If set to `None`, the client will try to connect to relay without waiting.
    #[inline]
    pub fn connection_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Send timeout (default: 20 secs)
    #[inline]
    pub fn send_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.send_timeout = timeout;
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

    /// Set pool options
    #[inline]
    pub fn pool(mut self, opts: RelayPoolOptions) -> Self {
        self.pool = opts;
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Connection {
    /// Mode
    pub mode: ConnectionMode,
    /// Target
    pub target: ConnectionTarget,
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
    ///
    /// The library used under the hood for websocket require a little change ([PR here](https://github.com/snapview/tungstenite-rs/pull/431)).
    /// Until it's merged, you have to add the following lines in your `Cargo.toml`:
    ///
    /// ```toml
    /// [patch.crates-io]
    /// tungstenite = { git = "https://github.com/yukibtc/tungstenite-rs", branch = "tor" }
    /// ```
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

/// Fallback policy
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FallbackPolicy {
    pub(super) older: Option<Duration>,
}

impl FallbackPolicy {
    /// Fallback if one or more events are older than [Duration]
    ///
    /// This apply only to `replaceable` and `param. replaceable` events!
    pub fn older(mut self, duration: Duration) -> Self {
        self.older = Some(duration);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FetchPolicyRelays {
    All,
    Specific(Vec<String>),
}

/// Fetch policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchPolicy {
    pub(super) database: bool,
    pub(super) relays: Option<FetchPolicyRelays>,
    pub(super) fallback: Option<FallbackPolicy>,
    pub(super) timeout: Duration,
}

impl Default for FetchPolicy {
    fn default() -> Self {
        Self::empty()
    }
}

impl FetchPolicy {
    /// New empty fetch policy
    #[inline]
    pub fn empty() -> Self {
        Self {
            database: false,
            relays: None,
            fallback: None,
            timeout: Duration::from_secs(10),
        }
    }

    /// Use both database and relays
    #[inline]
    pub fn both() -> Self {
        Self::empty().database().relays()
    }

    /// Use database
    #[inline]
    pub fn database(mut self) -> Self {
        self.database = true;
        self
    }

    /// Use all relays
    #[inline]
    pub fn relays(mut self) -> Self {
        self.relays = Some(FetchPolicyRelays::All);
        self
    }

    /// Use specified relays
    #[inline]
    pub fn specific_relays<I, S>(mut self, urls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.relays = Some(FetchPolicyRelays::Specific(
            urls.into_iter().map(|u| u.into()).collect(),
        ));
        self
    }

    /// Set a fallback policy
    #[inline]
    pub fn fallback(mut self, fallback: FallbackPolicy) -> Self {
        self.fallback = Some(fallback);
        self
    }

    /// Use relays as fallback
    #[inline]
    pub fn relays_as_fallback(self) -> Self {
        self.fallback(FallbackPolicy::default())
    }

    /// Timeout (default: 10 secs)
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
