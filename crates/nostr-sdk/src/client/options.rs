// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Client Options

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Options
#[derive(Debug, Clone)]
pub struct Options {
    /// Wait for connection (default: false)
    wait_for_connection: Arc<AtomicBool>,
    /// Wait for the msg to be sent (default: true)
    wait_for_send: Arc<AtomicBool>,
    /// POW difficulty for all events (default: 0)
    difficulty: Arc<AtomicU8>,
    /// REQ filters chunk size (default: 10)
    req_filters_chunk_size: Arc<AtomicU8>,
    /// Timeout (default: none)
    timeout: Option<Duration>,
    /// NIP46 timeout (default: 180 secs)
    #[cfg(feature = "nip46")]
    nip46_timeout: Option<Duration>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            wait_for_connection: Arc::new(AtomicBool::new(false)),
            wait_for_send: Arc::new(AtomicBool::new(true)),
            difficulty: Arc::new(AtomicU8::new(0)),
            req_filters_chunk_size: Arc::new(AtomicU8::new(10)),
            timeout: None,
            #[cfg(feature = "nip46")]
            nip46_timeout: Some(Duration::from_secs(180)),
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
            wait_for_connection: Arc::new(AtomicBool::new(wait)),
            ..self
        }
    }

    pub(crate) fn get_wait_for_connection(&self) -> bool {
        self.wait_for_connection.load(Ordering::SeqCst)
    }

    /// If set to `true`, `Client` wait that an event is sent before continue.
    pub fn wait_for_send(self, wait: bool) -> Self {
        Self {
            wait_for_send: Arc::new(AtomicBool::new(wait)),
            ..self
        }
    }

    pub(crate) fn get_wait_for_send(&self) -> bool {
        self.wait_for_send.load(Ordering::SeqCst)
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

    /// Set default timeout
    pub fn timeout(self, timeout: Option<Duration>) -> Self {
        Self { timeout, ..self }
    }

    pub(crate) fn get_timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Set NIP46 timeout
    #[cfg(feature = "nip46")]
    pub fn nip46_timeout(self, timeout: Option<Duration>) -> Self {
        Self {
            nip46_timeout: timeout,
            ..self
        }
    }

    #[cfg(feature = "nip46")]
    pub(crate) fn get_nip46_timeout(&self) -> Option<Duration> {
        self.nip46_timeout
    }
}
