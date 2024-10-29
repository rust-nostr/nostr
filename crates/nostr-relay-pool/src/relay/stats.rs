// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Stats

#[cfg(not(target_arch = "wasm32"))]
use std::collections::VecDeque;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use nostr::Timestamp;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::RwLock;

#[cfg(not(target_arch = "wasm32"))]
use super::constants::{LATENCY_MAX_VALUES, LATENCY_MIN_READS};

/// Ping Stats
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub(super) struct PingStats {
    sent_at: RwLock<Instant>,
    last_nonce: AtomicU64,
    replied: AtomicBool,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for PingStats {
    fn default() -> Self {
        Self {
            sent_at: RwLock::new(Instant::now()),
            last_nonce: AtomicU64::new(0),
            replied: AtomicBool::new(false),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PingStats {
    /// Get sent at
    #[inline]
    pub async fn sent_at(&self) -> Instant {
        *self.sent_at.read().await
    }

    /// Last nonce
    #[inline]
    pub fn last_nonce(&self) -> u64 {
        self.last_nonce.load(Ordering::SeqCst)
    }

    /// Replied
    #[inline]
    pub fn replied(&self) -> bool {
        self.replied.load(Ordering::SeqCst)
    }

    #[inline]
    pub(super) fn reset(&self) {
        self.set_last_nonce(0);
        self.set_replied(false);
    }

    #[inline]
    pub(super) async fn just_sent(&self) {
        let mut sent_at = self.sent_at.write().await;
        *sent_at = Instant::now();
    }

    #[inline]
    pub(super) fn set_last_nonce(&self, nonce: u64) {
        self.last_nonce.store(nonce, Ordering::SeqCst)
    }

    #[inline]
    pub(super) fn set_replied(&self, replied: bool) {
        self.replied.store(replied, Ordering::SeqCst);
    }
}

#[derive(Debug, Default)]
struct InnerRelayConnectionStats {
    attempts: AtomicUsize,
    success: AtomicUsize,
    bytes_sent: AtomicUsize,
    bytes_received: AtomicUsize,
    connected_at: AtomicU64,
    first_connection_at: AtomicU64,
    #[cfg(not(target_arch = "wasm32"))]
    latencies: RwLock<VecDeque<Duration>>,
    #[cfg(not(target_arch = "wasm32"))]
    ping: PingStats,
}

/// Relay connection stats
#[derive(Debug, Clone, Default)]
pub struct RelayConnectionStats {
    inner: Arc<InnerRelayConnectionStats>,
}

impl RelayConnectionStats {
    /// The number of times a connection has been attempted
    #[inline]
    pub fn attempts(&self) -> usize {
        self.inner.attempts.load(Ordering::SeqCst)
    }

    /// The number of times a connection has been successfully established
    #[inline]
    pub fn success(&self) -> usize {
        self.inner.success.load(Ordering::SeqCst)
    }

    /// Uptime
    #[deprecated(since = "0.36.0", note = "Use `success_rate` instead")]
    pub fn uptime(&self) -> f64 {
        self.success_rate()
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let attempts: usize = self.attempts();
        if attempts > 0 {
            self.success() as f64 / attempts as f64
        } else {
            0.0
        }
    }

    /// Bytes sent
    #[inline]
    pub fn bytes_sent(&self) -> usize {
        self.inner.bytes_sent.load(Ordering::SeqCst)
    }

    /// Bytes received
    #[inline]
    pub fn bytes_received(&self) -> usize {
        self.inner.bytes_received.load(Ordering::SeqCst)
    }

    /// Get UNIX timestamp of the last connection
    #[inline]
    pub fn connected_at(&self) -> Timestamp {
        Timestamp::from(self.inner.connected_at.load(Ordering::SeqCst))
    }

    /// Get UNIX timestamp of the first connection
    #[inline]
    pub fn first_connection_timestamp(&self) -> Timestamp {
        Timestamp::from(self.inner.first_connection_at.load(Ordering::SeqCst))
    }

    /// Calculate latency
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn latency(&self) -> Option<Duration> {
        // Acquire list
        let latencies = self.inner.latencies.read().await;

        // Check number of reads
        if latencies.len() < LATENCY_MIN_READS {
            return None;
        }

        // Calc latency
        let sum: Duration = latencies.iter().sum();
        sum.checked_div(latencies.len() as u32)
    }

    #[inline]
    pub(super) fn new_attempt(&self) {
        self.inner.attempts.fetch_add(1, Ordering::SeqCst);
    }

    pub(super) fn new_success(&self) {
        self.inner.success.fetch_add(1, Ordering::SeqCst);

        let now: u64 = Timestamp::now().as_u64();

        self.inner.connected_at.store(now, Ordering::SeqCst);

        if self.first_connection_timestamp() == Timestamp::from(0) {
            self.inner.first_connection_at.store(now, Ordering::SeqCst);
        }
    }

    #[inline]
    pub(super) fn add_bytes_sent(&self, size: usize) {
        if size > 0 {
            self.inner.bytes_sent.fetch_add(size, Ordering::SeqCst);
        }
    }

    #[inline]
    pub(super) fn add_bytes_received(&self, size: usize) {
        if size > 0 {
            self.inner.bytes_received.fetch_add(size, Ordering::SeqCst);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) async fn save_latency(&self, latency: Duration) {
        let mut latencies = self.inner.latencies.write().await;
        if latencies.len() >= LATENCY_MAX_VALUES {
            latencies.pop_back();
        }
        latencies.push_front(latency)
    }

    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn ping(&self) -> &PingStats {
        &self.inner.ping
    }
}
