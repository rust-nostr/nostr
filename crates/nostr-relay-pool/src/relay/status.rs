// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay status

use core::fmt;
use core::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug)]
pub(super) struct AtomicRelayStatus {
    value: AtomicU8,
}

impl Default for AtomicRelayStatus {
    fn default() -> Self {
        Self::new(RelayStatus::Initialized)
    }
}

impl AtomicRelayStatus {
    #[inline]
    pub(super) fn new(status: RelayStatus) -> Self {
        Self {
            value: AtomicU8::new(status as u8),
        }
    }

    #[inline]
    pub fn set(&self, status: RelayStatus) {
        self.value.store(status as u8, Ordering::SeqCst);
    }

    pub(super) fn load(&self) -> RelayStatus {
        let val: u8 = self.value.load(Ordering::SeqCst);
        match val {
            0 => RelayStatus::Initialized,
            1 => RelayStatus::Connecting,
            2 => RelayStatus::Connected,
            3 => RelayStatus::Disconnected,
            4 => RelayStatus::Terminated,
            _ => unreachable!(),
        }
    }
}

/// Relay connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayStatus {
    /// Relay initialized
    Initialized = 0,
    /// Connecting
    Connecting = 1,
    /// Relay connected
    Connected = 2,
    /// Relay disconnected, will retry to connect again
    Disconnected = 3,
    /// Relay completely disconnected
    Terminated = 4,
}

impl fmt::Display for RelayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialized => write!(f, "Initialized"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

impl RelayStatus {
    #[inline]
    pub(crate) fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized)
    }

    #[inline]
    pub(crate) fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if is `disconnected` or `terminated`
    #[inline]
    pub(crate) fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Terminated)
    }

    /// Check if relay can start connection (status is `initialized` or `terminated`)
    #[inline]
    pub(crate) fn can_connect(&self) -> bool {
        matches!(self, Self::Initialized | Self::Terminated)
    }
}
