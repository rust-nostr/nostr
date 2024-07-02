// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Pool options

/// Relay Pool Options
#[derive(Debug, Clone, Copy)]
pub struct RelayPoolOptions {
    pub(super) notification_channel_size: usize,
    pub(super) gossip: bool,
    pub(super) restore_relays_from_database: bool,
}

impl Default for RelayPoolOptions {
    fn default() -> Self {
        Self {
            notification_channel_size: 4096,
            gossip: false,
            restore_relays_from_database: false,
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

    /// Enable gossip model (default: false)
    #[inline]
    pub fn gossip(mut self, enable: bool) -> Self {
        self.gossip = enable;
        self
    }
}
