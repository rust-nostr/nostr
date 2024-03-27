// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_ffi::Timestamp;
use nostr_sdk::pool;
use uniffi::Object;

#[derive(Object)]
pub struct RelayConnectionStats {
    inner: pool::RelayConnectionStats,
}

impl From<pool::RelayConnectionStats> for RelayConnectionStats {
    fn from(inner: pool::RelayConnectionStats) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RelayConnectionStats {
    /// The number of times a connection has been attempted
    pub fn attempts(&self) -> u64 {
        self.inner.attempts() as u64
    }

    /// The number of times a connection has been successfully established
    pub fn success(&self) -> u64 {
        self.inner.success() as u64
    }

    /// Uptime
    pub fn uptime(&self) -> f64 {
        self.inner.uptime()
    }

    /// Bytes sent
    pub fn bytes_sent(&self) -> u64 {
        self.inner.bytes_sent() as u64
    }

    /// Bytes received
    pub fn bytes_received(&self) -> u64 {
        self.inner.bytes_received() as u64
    }

    /// Get UNIX timestamp of the last connection
    pub fn connected_at(&self) -> Timestamp {
        self.inner.connected_at().into()
    }

    /// Get UNIX timestamp of the first connection
    pub fn first_connection_timestamp(&self) -> Timestamp {
        self.inner.first_connection_timestamp().into()
    }

    pub async fn latency(&self) -> Option<Duration> {
        self.inner.latency().await
    }
}
