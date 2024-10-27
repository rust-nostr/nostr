// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use uniffi::Enum;

#[derive(Enum)]
pub enum RelayStatus {
    /// Relay initialized
    Initialized,
    /// Connecting
    Connecting,
    /// Relay connected
    Connected,
    /// Relay disconnected, will retry to connect again
    Disconnected,
    /// Relay completely disconnected
    Terminated,
}

impl From<nostr_sdk::RelayStatus> for RelayStatus {
    fn from(value: nostr_sdk::RelayStatus) -> Self {
        match value {
            nostr_sdk::RelayStatus::Initialized => Self::Initialized,
            nostr_sdk::RelayStatus::Connecting => Self::Connecting,
            nostr_sdk::RelayStatus::Connected => Self::Connected,
            nostr_sdk::RelayStatus::Disconnected => Self::Disconnected,
            nostr_sdk::RelayStatus::Terminated => Self::Terminated,
        }
    }
}
