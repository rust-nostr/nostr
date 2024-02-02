// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::relay::RelayPoolOptions;
use crate::RelaySendOptions;

pub(crate) const DEFAULT_SEND_TIMEOUT: Duration = Duration::from_secs(20);
/// Default Support Rust Nostr LUD16
pub const SUPPORT_RUST_NOSTR_LUD16: &str = "yuki@getalby.com"; // TODO: use a rust-nostr dedicated LUD16
/// Default Support Rust Nostr basis points
pub const DEFAULT_SUPPORT_RUST_NOSTR_BSP: u64 = 500; // 5%

/// Options
#[derive(Debug, Clone)]
pub struct Options {
    /// Wait for the msg to be sent (default: true)
    wait_for_send: Arc<AtomicBool>,
    /// Wait for the subscription msg to be sent (default: false)
    wait_for_subscription: Arc<AtomicBool>,
    /// POW difficulty for all events (default: 0)
    difficulty: Arc<AtomicU8>,
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
    /// Shutdown on [Client](super::Client) drop
    pub shutdown_on_drop: bool,
    /// Pool Options
    pub pool: RelayPoolOptions,
    /// Support Rust Nostr in basis points (default: 5%)
    ///
    /// 100 bps = 1%
    support_rust_nostr_bps: Arc<AtomicU64>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            wait_for_send: Arc::new(AtomicBool::new(true)),
            wait_for_subscription: Arc::new(AtomicBool::new(false)),
            difficulty: Arc::new(AtomicU8::new(0)),
            req_filters_chunk_size: Arc::new(AtomicU8::new(10)),
            skip_disconnected_relays: Arc::new(AtomicBool::new(true)),
            timeout: Duration::from_secs(60),
            connection_timeout: None,
            send_timeout: Some(DEFAULT_SEND_TIMEOUT),
            #[cfg(not(target_arch = "wasm32"))]
            proxy: None,
            shutdown_on_drop: false,
            pool: RelayPoolOptions::default(),
            support_rust_nostr_bps: Arc::new(AtomicU64::new(DEFAULT_SUPPORT_RUST_NOSTR_BSP)),
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

    /// Set default POW diffficulty for `Event`
    pub fn difficulty(self, difficulty: u8) -> Self {
        Self {
            difficulty: Arc::new(AtomicU8::new(difficulty)),
            ..self
        }
    }

    pub(crate) fn get_difficulty(&self) -> u8 {
        self.difficulty.load(Ordering::SeqCst)
    }

    pub(crate) fn update_difficulty(&self, difficulty: u8) {
        let _ = self
            .difficulty
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(difficulty));
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
    pub fn shutdown_on_drop(self, value: bool) -> Self {
        Self {
            shutdown_on_drop: value,
            ..self
        }
    }

    /// Set pool options
    pub fn pool(self, opts: RelayPoolOptions) -> Self {
        Self { pool: opts, ..self }
    }

    /// Support Rust Nostr with a % of zaps (default: 5%)
    ///
    /// 100 bps = 1%
    pub fn support_rust_nostr(mut self, bps: u64) -> Self {
        self.support_rust_nostr_bps = Arc::new(AtomicU64::new(bps));
        self
    }

    /// Update Support Rust Nostr basis points
    ///
    /// 100 bps = 1%
    pub fn update_support_rust_nostr(&self, bps: u64) {
        let _ =
            self.support_rust_nostr_bps
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(bps));
    }

    /// Get Support Rust Nostr percentage
    pub fn get_support_rust_nostr_percentage(&self) -> Option<f64> {
        let bps: u64 = self.support_rust_nostr_bps.load(Ordering::SeqCst);
        if bps != 0 {
            Some(bps as f64 / 10_000.0)
        } else {
            None
        }
    }
}
