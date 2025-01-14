// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Wipe trait

use async_trait::async_trait;

use crate::error::DatabaseError;

/// Nostr Database wipe trait
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrDatabaseWipe {
    /// Wipe all data
    async fn wipe(&self) -> Result<(), DatabaseError>;
}
