// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use nostr_relay_pool::relay::options::DEFAULT_SEND_TIMEOUT;
use nostr_relay_pool::{RelayLimits, RelayPoolOptions, RelaySendOptions};

/// Options
#[derive(Debug, Clone)]
pub struct Options {
    wait_for_send: bool,
    wait_for_subscription: bool,
    new_events_difficulty: Arc<AtomicU8>,
    min_pow_difficulty: Arc<AtomicU8>,
    pub(super) req_filters_chunk_size: u8,
    skip_disconnected_relays: bool,
    pub(super) timeout: Duration,
    pub(super) connection_timeout: Option<Duration>,
    send_timeout: Option<Duration>,
    pub(super) nip42_auto_authentication: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) proxy: Proxy,
    pub(super) relay_limits: RelayLimits,
    pub(super) pool: RelayPoolOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            wait_for_send: true,
            wait_for_subscription: false,
            new_events_difficulty: Arc::new(AtomicU8::new(0)),
            min_pow_difficulty: Arc::new(AtomicU8::new(0)),
            req_filters_chunk_size: 10,
            skip_disconnected_relays: true,
            timeout: Duration::from_secs(60),
            connection_timeout: None,
            send_timeout: Some(DEFAULT_SEND_TIMEOUT),
            nip42_auto_authentication: true,
            #[cfg(not(target_arch = "wasm32"))]
            proxy: Proxy::default(),
            relay_limits: RelayLimits::default(),
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

    /// Timeout (default: 60)
    ///
    /// Used in `get_events_of` and similar methods as default timeout.
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
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
        self.nip42_auto_authentication = enabled;
        self
    }

    /// Proxy
    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.proxy = proxy;
        self
    }

    /// Set relay limits
    #[inline]
    pub fn relay_limits(mut self, limits: RelayLimits) -> Self {
        self.relay_limits = limits;
        self
    }

    /// Set pool options
    #[inline]
    pub fn pool(mut self, opts: RelayPoolOptions) -> Self {
        self.pool = opts;
        self
    }
}

/// Proxy target
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ProxyTarget {
    /// Use proxy for all relays
    #[default]
    All,
    /// Use proxy only for `.onion` relays
    Onion,
}

/// Proxy
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Proxy {
    pub(super) addr: Option<SocketAddr>,
    pub(super) target: ProxyTarget,
}

#[cfg(not(target_arch = "wasm32"))]
impl Proxy {
    /// Compose proxy
    #[inline]
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr: Some(addr),
            target: ProxyTarget::default(),
        }
    }

    /// Set proxy target (default: all)
    #[inline]
    pub fn target(mut self, target: ProxyTarget) -> Self {
        self.target = target;
        self
    }
}
