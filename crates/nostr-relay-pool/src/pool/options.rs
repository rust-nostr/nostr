// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Pool options

use crate::relay::RelayFilteringMode;

/// Relay Pool Options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayPoolOptions {
    pub(super) notification_channel_size: usize,
    pub(super) filtering_mode: RelayFilteringMode,
}

impl Default for RelayPoolOptions {
    fn default() -> Self {
        Self {
            notification_channel_size: 4096,
            filtering_mode: RelayFilteringMode::default(),
        }
    }
}

impl RelayPoolOptions {
    /// New default options
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Notification channel size (default: 4096)
    #[inline]
    pub fn notification_channel_size(mut self, size: usize) -> Self {
        self.notification_channel_size = size;
        self
    }

    /// Relay filtering mode (default: blacklist)
    #[inline]
    pub fn filtering_mode(mut self, mode: RelayFilteringMode) -> Self {
        self.filtering_mode = mode;
        self
    }
}
