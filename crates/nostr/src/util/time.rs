// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::{SystemTime, UNIX_EPOCH};

/// Timestamp in seconds
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Invalid system time")
        .as_secs()
}

/// Timestamp in nanos
pub fn timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Invalid system time")
        .as_nanos()
}
