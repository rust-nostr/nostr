// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Pool options

use super::constants::DEFAULT_NOTIFICATION_CHANNEL_SIZE;

/// Relay Pool Options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayPoolOptions {
    pub(super) max_relays: Option<usize>,
    pub(super) nip42_auto_authentication: bool,
    pub(super) notification_channel_size: usize,
}

impl Default for RelayPoolOptions {
    fn default() -> Self {
        Self {
            max_relays: None,
            nip42_auto_authentication: true,
            notification_channel_size: DEFAULT_NOTIFICATION_CHANNEL_SIZE,
        }
    }
}

impl RelayPoolOptions {
    /// New default options
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Max relays (default: None)
    #[inline]
    pub fn max_relays(mut self, num: Option<usize>) -> Self {
        self.max_relays = num;
        self
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn automatic_authentication(mut self, enabled: bool) -> Self {
        self.nip42_auto_authentication = enabled;
        self
    }

    /// Notification channel size (default: [`DEFAULT_NOTIFICATION_CHANNEL_SIZE`])
    #[inline]
    pub fn notification_channel_size(mut self, size: usize) -> Self {
        self.notification_channel_size = size;
        self
    }
}
