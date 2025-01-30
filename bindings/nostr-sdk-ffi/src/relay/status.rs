// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use uniffi::Enum;

#[derive(Enum)]
pub enum RelayStatus {
    /// Initialized
    Initialized,
    /// Pending
    Pending,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Disconnected, will retry to connect again
    Disconnected,
    /// Completely disconnected
    Terminated,
}

impl From<nostr_sdk::RelayStatus> for RelayStatus {
    fn from(value: nostr_sdk::RelayStatus) -> Self {
        match value {
            nostr_sdk::RelayStatus::Initialized => Self::Initialized,
            nostr_sdk::RelayStatus::Pending => Self::Pending,
            nostr_sdk::RelayStatus::Connecting => Self::Connecting,
            nostr_sdk::RelayStatus::Connected => Self::Connected,
            nostr_sdk::RelayStatus::Disconnected => Self::Disconnected,
            nostr_sdk::RelayStatus::Terminated => Self::Terminated,
        }
    }
}
