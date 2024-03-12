// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use nostr_relay_pool::relay::options::DEFAULT_SEND_TIMEOUT;
use nostr_relay_pool::{RelayLimits, RelayPoolOptions, RelaySendOptions};

/// Options
#[derive(Debug, Clone)]
pub struct Options {
    /// Wait for the msg to be sent (default: true)
    wait_for_send: Arc<AtomicBool>,
    /// Wait for the subscription msg to be sent (default: false)
    wait_for_subscription: Arc<AtomicBool>,
    new_events_difficulty: Arc<AtomicU8>,
    min_pow_difficulty: Arc<AtomicU8>,
    /// REQ filters chunk size (default: 10)
    req_filters_chunk_size: Arc<AtomicU8>,
    /// Skip disconnected relays during send methods (default: true)
    ///
    /// If the relay made just 1 attempt, the relay will not be skipped
    skip_disconnected_relays: Arc<AtomicBool>,
    /// Timeout (default: 60)
    ///
    /// Used in `get_events_of`, `req_events_of` and similar as default timeout.
    pub timeout: Duration,
    /// Relay connection timeout (default: None)
    ///
    /// If set to `None`, the client will try to connect to relay without waiting.
    pub connection_timeout: Option<Duration>,
    /// Send timeout (default: 20 secs)
    pub send_timeout: Option<Duration>,
    /// Proxy
    #[cfg(not(target_arch = "wasm32"))]
    pub proxy: Option<SocketAddr>,
    /// Default limits for new added relays
    pub relay_limits: RelayLimits,
    /// Pool Options
    pub pool: RelayPoolOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            wait_for_send: Arc::new(AtomicBool::new(true)),
            wait_for_subscription: Arc::new(AtomicBool::new(false)),
            new_events_difficulty: Arc::new(AtomicU8::new(0)),
            min_pow_difficulty: Arc::new(AtomicU8::new(0)),
            req_filters_chunk_size: Arc::new(AtomicU8::new(10)),
            skip_disconnected_relays: Arc::new(AtomicBool::new(true)),
            timeout: Duration::from_secs(60),
            connection_timeout: None,
            send_timeout: Some(DEFAULT_SEND_TIMEOUT),
            #[cfg(not(target_arch = "wasm32"))]
            proxy: None,
            relay_limits: RelayLimits::default(),
            pool: RelayPoolOptions::default(),
        }
    }
}

impl Options {
    /// Create new (default) [`Options`]
    pub fn new() -> Self {
        Self::default()
    }

    /// If set to `true`, `Client` wait that a message is sent before continue.
    pub fn wait_for_send(self, wait: bool) -> Self {
        Self {
            wait_for_send: Arc::new(AtomicBool::new(wait)),
            ..self
        }
    }

    pub(crate) fn get_wait_for_send(&self) -> RelaySendOptions {
        let skip_disconnected = self.get_skip_disconnected_relays();
        let wait_for_send = self.wait_for_send.load(Ordering::SeqCst);
        RelaySendOptions::new()
            .timeout(self.send_timeout)
            .skip_send_confirmation(!wait_for_send)
            .skip_disconnected(skip_disconnected)
    }

    /// If set to `true`, `Client` wait that a subscription msg is sent before continue (`subscribe` and `unsubscribe` methods)
    pub fn wait_for_subscription(self, wait: bool) -> Self {
        Self {
            wait_for_subscription: Arc::new(AtomicBool::new(wait)),
            ..self
        }
    }

    pub(crate) fn get_wait_for_subscription(&self) -> RelaySendOptions {
        let skip_disconnected = self.get_skip_disconnected_relays();
        let wait_for_subscription = self.wait_for_subscription.load(Ordering::SeqCst);
        RelaySendOptions::new()
            .timeout(self.send_timeout)
            .skip_send_confirmation(!wait_for_subscription)
            .skip_disconnected(skip_disconnected)
    }

    /// Set default POW difficulty for `Event`
    pub fn difficulty(self, difficulty: u8) -> Self {
        Self {
            new_events_difficulty: Arc::new(AtomicU8::new(difficulty)),
            ..self
        }
    }

    pub(crate) fn get_difficulty(&self) -> u8 {
        self.new_events_difficulty.load(Ordering::SeqCst)
    }

    pub(crate) fn update_difficulty(&self, difficulty: u8) {
        self.new_events_difficulty
            .store(difficulty, Ordering::SeqCst);
    }

    /// Minimum POW difficulty for received events
    pub fn min_pow(self, difficulty: u8) -> Self {
        Self {
            min_pow_difficulty: Arc::new(AtomicU8::new(difficulty)),
            ..self
        }
    }

    pub(crate) fn get_min_pow_difficulty(&self) -> u8 {
        self.min_pow_difficulty.load(Ordering::SeqCst)
    }

    /// Update min POW difficulty
    pub fn update_min_pow_difficulty(&self, difficulty: u8) {
        self.min_pow_difficulty.store(difficulty, Ordering::SeqCst);
    }

    /// Set `REQ` filters chunk size
    pub fn req_filters_chunk_size(self, size: u8) -> Self {
        Self {
            req_filters_chunk_size: Arc::new(AtomicU8::new(size)),
            ..self
        }
    }

    pub(crate) fn get_req_filters_chunk_size(&self) -> usize {
        self.req_filters_chunk_size.load(Ordering::SeqCst) as usize
    }

    /// Skip disconnected relays during send methods (default: true)
    ///
    /// If the relay made just 1 attempt, the relay will not be skipped
    pub fn skip_disconnected_relays(self, skip: bool) -> Self {
        Self {
            skip_disconnected_relays: Arc::new(AtomicBool::new(skip)),
            ..self
        }
    }

    pub(crate) fn get_skip_disconnected_relays(&self) -> bool {
        self.skip_disconnected_relays.load(Ordering::SeqCst)
    }

    /// Set default timeout
    pub fn timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }

    /// Connection timeout (default: None)
    ///
    /// If set to `None`, the client will try to connect to the relays without waiting.
    pub fn connection_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set default send timeout
    pub fn send_timeout(self, timeout: Option<Duration>) -> Self {
        Self {
            send_timeout: timeout,
            ..self
        }
    }

    /// Proxy
    #[cfg(not(target_arch = "wasm32"))]
    pub fn proxy(mut self, proxy: Option<SocketAddr>) -> Self {
        self.proxy = proxy;
        self
    }

    /// Shutdown client on drop
    #[deprecated(since = "0.29.0", note = "No longer needed")]
    pub fn shutdown_on_drop(self, _value: bool) -> Self {
        self
    }

    /// Set custom relay limits
    pub fn relay_limits(mut self, limits: RelayLimits) -> Self {
        self.relay_limits = limits;
        self
    }

    /// Set pool options
    pub fn pool(self, opts: RelayPoolOptions) -> Self {
        Self { pool: opts, ..self }
    }
}
