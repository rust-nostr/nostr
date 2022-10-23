// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use chrono::Utc;

/// Timestamp in seconds
pub fn timestamp() -> u64 {
    Utc::now().timestamp() as u64
}
