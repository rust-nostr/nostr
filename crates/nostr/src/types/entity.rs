// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Entity

/// Nostr [`Entity`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Entity {
    /// Account
    Account,
    /// Channel
    Channel,
    /// Unknown
    Unknown,
}
