// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
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
            1 => RelayStatus::Pending,
            2 => RelayStatus::Connecting,
            3 => RelayStatus::Connected,
            4 => RelayStatus::Disconnected,
            5 => RelayStatus::Terminated,
            6 => RelayStatus::Banned,
            _ => unreachable!(),
        }
    }
}

/// Relay connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayStatus {
    /// The relay has just been created.
    Initialized = 0,
    /// The relay will try to connect shortly.
    Pending = 1,
    /// Trying to connect.
    Connecting = 2,
    /// Connected.
    Connected = 3,
    /// The connection failed, but another attempt will occur soon.
    Disconnected = 4,
    /// The connection has been terminated and no retry will occur.
    Terminated = 5,
    /// The relay has been banned.
    Banned = 6,
}

impl fmt::Display for RelayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialized => write!(f, "Initialized"),
            Self::Pending => write!(f, "Pending"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Terminated => write!(f, "Terminated"),
            Self::Banned => write!(f, "Banned"),
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

    /// Check if is `disconnected`, `terminated` or `banned`.
    #[inline]
    pub(crate) fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Terminated | Self::Banned)
    }

    /// Check if is [`RelayStatus::Terminated`]
    pub(crate) fn is_terminated(&self) -> bool {
        matches!(self, Self::Terminated)
    }

    /// Check if is [`RelayStatus::Banned`]
    pub(crate) fn is_banned(&self) -> bool {
        matches!(self, Self::Banned)
    }

    /// Check if relay can start a connection (status is `initialized` or `terminated`)
    #[inline]
    pub(crate) fn can_connect(&self) -> bool {
        matches!(self, Self::Initialized | Self::Terminated)
    }
}
