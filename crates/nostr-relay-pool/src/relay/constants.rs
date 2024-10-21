// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay constants

use core::time::Duration;

/// Default send timeout
// IF CHANGED, REMEMBER TO UPDATE THE DOCS!
pub const DEFAULT_SEND_TIMEOUT: Duration = Duration::from_secs(20);

pub(super) const DEFAULT_RETRY_SEC: u64 = 10;
pub(super) const MIN_RETRY_SEC: u64 = 5;
pub(super) const MAX_ADJ_RETRY_SEC: u64 = 60;

pub(super) const NEGENTROPY_HIGH_WATER_UP: usize = 100;
pub(super) const NEGENTROPY_LOW_WATER_UP: usize = 50;
pub(super) const NEGENTROPY_BATCH_SIZE_DOWN: usize = 50;

pub(super) const MIN_ATTEMPTS: usize = 1;
pub(super) const MIN_UPTIME: f64 = 0.90;

pub(super) const PING_INTERVAL: Duration = Duration::from_secs(50); // Used also for latency calculation

pub(super) const WEBSOCKET_TX_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum number of reads to be saved in memory to calculate latency
#[cfg(not(target_arch = "wasm32"))]
pub const LATENCY_MAX_VALUES: usize = 50;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) const LATENCY_MIN_READS: usize = 3;
