// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Relay Stats

#[cfg(not(target_arch = "wasm32"))]
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use nostr::Timestamp;
use tokio::sync::RwLock;

#[cfg(feature = "blocking")]
use crate::RUNTIME;

/// Ping Stats
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct PingStats {
    sent_at: Arc<RwLock<Instant>>,
    last_nonce: Arc<AtomicU64>,
    replied: Arc<AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for PingStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PingStats {
    /// New default ping stats
    pub fn new() -> Self {
        Self {
            sent_at: Arc::new(RwLock::new(Instant::now())),
            last_nonce: Arc::new(AtomicU64::new(0)),
            replied: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get sent at
    pub async fn sent_at(&self) -> Instant {
        *self.sent_at.read().await
    }

    /// Last nonce
    pub fn last_nonce(&self) -> u64 {
        self.last_nonce.load(Ordering::SeqCst)
    }

    /// Replied
    pub fn replied(&self) -> bool {
        self.replied.load(Ordering::SeqCst)
    }

    pub(crate) fn reset(&self) {
        self.set_last_nonce(0);
        self.set_replied(false);
    }

    pub(crate) async fn just_sent(&self) {
        let mut sent_at = self.sent_at.write().await;
        *sent_at = Instant::now();
    }

    pub(crate) fn set_last_nonce(&self, nonce: u64) {
        let _ = self
            .last_nonce
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(nonce));
    }

    pub(crate) fn set_replied(&self, replied: bool) {
        let _ = self
            .replied
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(replied));
    }
}

/// [`Relay`] connection stats
#[derive(Debug, Clone)]
pub struct RelayConnectionStats {
    attempts: Arc<AtomicUsize>,
    success: Arc<AtomicUsize>,
    bytes_sent: Arc<AtomicUsize>,
    bytes_received: Arc<AtomicUsize>,
    connected_at: Arc<AtomicU64>,
    first_connection_timestamp: Arc<AtomicU64>,
    #[cfg(not(target_arch = "wasm32"))]
    latencies: Arc<RwLock<VecDeque<Duration>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) ping: PingStats,
}

impl Default for RelayConnectionStats {
    fn default() -> Self {
        Self::new()
    }
}

impl RelayConnectionStats {
    /// New connections stats
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(AtomicUsize::new(0)),
            success: Arc::new(AtomicUsize::new(0)),
            bytes_sent: Arc::new(AtomicUsize::new(0)),
            bytes_received: Arc::new(AtomicUsize::new(0)),
            connected_at: Arc::new(AtomicU64::new(0)),
            first_connection_timestamp: Arc::new(AtomicU64::new(0)),
            #[cfg(not(target_arch = "wasm32"))]
            latencies: Arc::new(RwLock::new(VecDeque::new())),
            #[cfg(not(target_arch = "wasm32"))]
            ping: PingStats::default(),
        }
    }

    /// The number of times a connection has been attempted
    pub fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    /// The number of times a connection has been successfully established
    pub fn success(&self) -> usize {
        self.success.load(Ordering::SeqCst)
    }

    /// Uptime
    pub fn uptime(&self) -> f64 {
        let success: f64 = self.success() as f64;
        let attempts: f64 = self.attempts() as f64;
        if attempts != 0.0 {
            success / attempts
        } else {
            0.0
        }
    }

    /// Bytes sent
    pub fn bytes_sent(&self) -> usize {
        self.bytes_sent.load(Ordering::SeqCst)
    }

    /// Bytes received
    pub fn bytes_received(&self) -> usize {
        self.bytes_received.load(Ordering::SeqCst)
    }

    /// Get UNIX timestamp of the last connection
    pub fn connected_at(&self) -> Timestamp {
        Timestamp::from(self.connected_at.load(Ordering::SeqCst))
    }

    /// Get UNIX timestamp of the first connection
    pub fn first_connection_timestamp(&self) -> Timestamp {
        Timestamp::from(self.first_connection_timestamp.load(Ordering::SeqCst))
    }

    /// Calculate latency
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn latency(&self) -> Option<Duration> {
        let latencies = self.latencies.read().await;
        let sum: Duration = latencies.iter().sum();
        sum.checked_div(latencies.len() as u32)
    }

    /// Calculate latency
    #[cfg(all(not(target_arch = "wasm32"), feature = "blocking"))]
    pub fn latency_blocking(&self) -> Option<Duration> {
        RUNTIME.block_on(async { self.latency().await })
    }

    pub(crate) fn new_attempt(&self) {
        self.attempts.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn new_success(&self) {
        self.success.fetch_add(1, Ordering::SeqCst);

        let now: u64 = Timestamp::now().as_u64();

        let _ = self
            .connected_at
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(now));

        if self.first_connection_timestamp() == Timestamp::from(0) {
            let _ = self.first_connection_timestamp.fetch_update(
                Ordering::SeqCst,
                Ordering::SeqCst,
                |_| Some(now),
            );
        }
    }

    pub(crate) fn add_bytes_sent(&self, size: usize) {
        self.bytes_sent.fetch_add(size, Ordering::SeqCst);
    }

    pub(crate) fn add_bytes_received(&self, size: usize) {
        if size > 0 {
            self.bytes_received.fetch_add(size, Ordering::SeqCst);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn save_latency(&self, latency: Duration) {
        let mut latencies = self.latencies.write().await;
        if latencies.len() >= 5 {
            latencies.pop_back();
        }
        latencies.push_front(latency)
    }
}
