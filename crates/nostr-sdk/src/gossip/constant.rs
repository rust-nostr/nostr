// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

/// Take at max N relays per NIP-65 marker.
pub(super) const MAX_RELAYS_PER_NIP65_MARKER: usize = 3;
pub(super) const MAX_NIP17_RELAYS: usize = 3;
/// Used as a kind of protection if someone inserts too many relays in the NIP65 list.
/// Only the first 10 relays are extracted from the NIP65 list and then handled.
pub(super) const MAX_RELAYS_ALLOWED_IN_NIP65: usize = 10;
pub const PUBKEY_METADATA_OUTDATED_AFTER: Duration = Duration::from_secs(60 * 60); // 60 min
pub const CHECK_OUTDATED_INTERVAL: Duration = Duration::from_secs(60 * 5); // 5 min
