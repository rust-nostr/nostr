// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Wipe trait

use nostr::util::BoxedFuture;

use crate::error::DatabaseError;

/// Nostr Database wipe trait
pub trait NostrDatabaseWipe {
    /// Wipe all data
    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>>;
}
