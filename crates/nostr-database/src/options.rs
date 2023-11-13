// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr Database options

/// Database options
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DatabaseOptions {
    /// Store events (?)
    pub events: bool,
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self { events: true }
    }
}

impl DatabaseOptions {
    /// New default database options
    pub fn new() -> Self {
        Self::default()
    }
}
