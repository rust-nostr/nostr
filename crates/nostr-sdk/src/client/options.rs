// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Client Options

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::relay::RelayPoolOptions;

pub(crate) const DEFAULT_SEND_TIMEOUT: Duration = Duration::from_secs(20);

/// Options
#[derive(Debug, Clone)]
pub struct Options {
    /// Wait for connection (default: false)
    pub wait_for_connection: bool,
    /// Wait for the msg to be sent (default: true)
    wait_for_send: Arc<AtomicBool>,
    /// Wait for the subscription msg to be sent (default: false)
    wait_for_subscription: Arc<AtomicBool>,
    /// POW difficulty for all events (default: 0)
    difficulty: Arc<AtomicU8>,
    /// REQ filters chunk size (default: 10)
    req_filters_chunk_size: Arc<AtomicU8>,
    /// Skip disconnected relays during send methods (default: false)
    ///
    /// If the relay made just 1 attempt, the relay will not be skipped
    skip_disconnected_relays: Arc<AtomicBool>,
    /// Timeout (default: 60)
    ///
    /// Used in `get_events_of`, `req_events_of` and similar as default timeout.
    pub timeout: Duration,
    /// Send timeout (default: 20 secs)
    pub send_timeout: Option<Duration>,
    /// NIP46 timeout (default: 180 secs)
    #[cfg(feature = "nip46")]
    pub nip46_timeout: Option<Duration>,
    /// Shutdown on [Client](super::Client) drop
    pub shutdown_on_drop: bool,
    /// Pool Options
    pub pool: RelayPoolOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            wait_for_connection: false,
            wait_for_send: Arc::new(AtomicBool::new(true)),
            wait_for_subscription: Arc::new(AtomicBool::new(false)),
            difficulty: Arc::new(AtomicU8::new(0)),
            req_filters_chunk_size: Arc::new(AtomicU8::new(10)),
            skip_disconnected_relays: Arc::new(AtomicBool::new(false)),
            timeout: Duration::from_secs(60),
            send_timeout: Some(DEFAULT_SEND_TIMEOUT),
            #[cfg(feature = "nip46")]
            nip46_timeout: Some(Duration::from_secs(180)),
            shutdown_on_drop: false,
            pool: RelayPoolOptions::default(),
        }
    }
}

impl Options {
    /// Create new (default) [`Options`]
    pub fn new() -> Self {
        Self::default()
    }

    /// If set to `true`, `Client` wait that `Relay` try at least one time to enstablish a connection before continue.
    pub fn wait_for_connection(self, wait: bool) -> Self {
        Self {
            wait_for_connection: wait,
            ..self
        }
    }

    /// If set to `true`, `Client` wait that a message is sent before continue.
    pub fn wait_for_send(self, wait: bool) -> Self {
        Self {
            wait_for_send: Arc::new(AtomicBool::new(wait)),
            ..self
        }
    }

    pub(crate) fn get_wait_for_send(&self) -> bool {
        self.wait_for_send.load(Ordering::SeqCst)
    }

    /// If set to `true`, `Client` wait that a subscription msg is sent before continue (`subscribe` and `unsubscribe` methods)
    pub fn wait_for_subscription(self, wait: bool) -> Self {
        Self {
            wait_for_subscription: Arc::new(AtomicBool::new(wait)),
            ..self
        }
    }

    pub(crate) fn get_wait_for_subscription(&self) -> bool {
        self.wait_for_subscription.load(Ordering::SeqCst)
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

    /// Skip disconnected relays during send methods (default: false)
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

    /// Set default send timeout
    pub fn send_timeout(self, timeout: Option<Duration>) -> Self {
        Self {
            send_timeout: timeout,
            ..self
        }
    }

    /// Set NIP46 timeout
    #[cfg(feature = "nip46")]
    pub fn nip46_timeout(self, timeout: Option<Duration>) -> Self {
        Self {
            nip46_timeout: timeout,
            ..self
        }
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
}
