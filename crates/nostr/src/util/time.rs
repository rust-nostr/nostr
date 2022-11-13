// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::{SystemTime, UNIX_EPOCH};

/// Timestamp in seconds
pub fn timestamp() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => time.as_secs(),
        Err(_) => panic!("Invalid system time"),
    }
}
