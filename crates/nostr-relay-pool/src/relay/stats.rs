// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Stats

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use nostr::Timestamp;

#[cfg(not(target_arch = "wasm32"))]
use super::constants::LATENCY_MIN_READS;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default)]
struct AverageLatency {
    /// Sum of all latencies in milliseconds
    total: AtomicU64,
    /// Count of latencies
    count: AtomicU64,
}

#[derive(Debug, Default)]
struct InnerRelayConnectionStats {
    attempts: AtomicUsize,
    success: AtomicUsize,
    bytes_sent: AtomicUsize,
    bytes_received: AtomicUsize,
    // TODO: keep track of msg/event sending attempts and success?
    connected_at: AtomicU64,
    first_connection_at: AtomicU64,
    // Activity tracking for on-demand connections
    last_activity_at: AtomicU64,
    /// When has waked up from sleep
    woke_up_at: AtomicU64,
    #[cfg(not(target_arch = "wasm32"))]
    latency: AverageLatency,
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

    /// Get UNIX timestamp of the last activity
    #[inline]
    pub(super) fn last_activity_at(&self) -> Timestamp {
        Timestamp::from(self.inner.last_activity_at.load(Ordering::SeqCst))
    }

    /// Update last activity timestamp
    #[inline]
    pub(super) fn update_activity(&self) {
        let now: u64 = Timestamp::now().as_u64();
        self.inner.last_activity_at.store(now, Ordering::SeqCst);
    }

    /// When has woke up from sleep
    #[inline]
    pub(super) fn woke_up_at(&self) -> Timestamp {
        Timestamp::from(self.inner.woke_up_at.load(Ordering::SeqCst))
    }

    /// Update the wake-up timestamp
    #[inline]
    pub(super) fn just_woke_up(&self) {
        let now: u64 = Timestamp::now().as_u64();
        self.inner.woke_up_at.store(now, Ordering::SeqCst);
    }

    /// Calculate latency
    #[cfg(not(target_arch = "wasm32"))]
    pub fn latency(&self) -> Option<Duration> {
        let total: u64 = self.inner.latency.total.load(Ordering::SeqCst);
        let count: u64 = self.inner.latency.count.load(Ordering::SeqCst);

        // Check number of reads
        if count < LATENCY_MIN_READS {
            return None;
        }

        // Calc latency
        total.checked_div(count).map(Duration::from_millis)
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
    pub(super) fn save_latency(&self, latency: Duration) {
        let ms: u128 = latency.as_millis();
        if ms <= u64::MAX as u128 {
            self.inner
                .latency
                .total
                .fetch_add(ms as u64, Ordering::SeqCst);
            self.inner.latency.count.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attempt_success() {
        let stats = RelayConnectionStats::default();
        stats.new_attempt();
        stats.new_success();
        assert_eq!(stats.attempts(), 1);
        assert_eq!(stats.success(), 1);
        assert_eq!(stats.success_rate(), 1.0);
        assert!(stats.connected_at().as_u64() > 0);
        assert!(stats.first_connection_timestamp().as_u64() > 0);
    }

    #[test]
    fn test_add_bytes() {
        let stats = RelayConnectionStats::default();
        stats.add_bytes_sent(0);
        assert_eq!(stats.bytes_sent(), 0);
        stats.add_bytes_sent(50);
        assert_eq!(stats.bytes_sent(), 50);
        stats.add_bytes_received(0);
        assert_eq!(stats.bytes_received(), 0);
        stats.add_bytes_received(30);
        assert_eq!(stats.bytes_received(), 30);
    }
}
