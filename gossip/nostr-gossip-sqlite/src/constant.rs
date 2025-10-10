// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use crate::flags::Flags;

pub(super) const PUBKEY_METADATA_OUTDATED_AFTER: Duration = Duration::from_secs(60 * 60); // 60 min

pub(super) const READ_WRITE_FLAGS: Flags = Flags::read_write();
