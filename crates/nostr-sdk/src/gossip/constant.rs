// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

/// Max number of relays allowed in NIP17/NIP65 lists
pub const MAX_RELAYS_LIST: usize = 5;
pub const PUBKEY_METADATA_OUTDATED_AFTER: Duration = Duration::from_secs(60 * 60); // 60 min
pub const CHECK_OUTDATED_INTERVAL: Duration = Duration::from_secs(60 * 5); // 5 min
