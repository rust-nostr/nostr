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
            7 => RelayStatus::Sleeping,
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
    /// Relay is sleeping
    Sleeping = 7,
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
            Self::Sleeping => write!(f, "Sleeping"),
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

    /// Check if is [`RelayStatus::Sleeping`]
    pub(crate) fn is_sleeping(&self) -> bool {
        matches!(self, Self::Sleeping)
    }

    /// Check if relay can start a connection (status is `initialized` or `terminated`)
    #[inline]
    pub(crate) fn can_connect(&self) -> bool {
        matches!(self, Self::Initialized | Self::Terminated | Self::Sleeping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_set() {
        let relay = AtomicRelayStatus::default();
        relay.set(RelayStatus::Connected);
        assert_eq!(relay.load(), RelayStatus::Connected);
    }

    #[test]
    fn test_status_initialized() {
        let status = RelayStatus::Initialized;
        assert!(status.is_initialized());
        assert!(!status.is_connected());
        assert!(!status.is_disconnected());
        assert!(!status.is_terminated());
        assert!(!status.is_banned());
        assert!(status.can_connect());
        let relay = AtomicRelayStatus::new(status);
        assert_eq!(relay.load(), RelayStatus::Initialized);
    }

    #[test]
    fn test_status_pending() {
        let status = RelayStatus::Pending;
        assert!(!status.is_initialized());
        assert!(!status.is_connected());
        assert!(!status.is_disconnected());
        assert!(!status.is_terminated());
        assert!(!status.is_banned());
        assert!(!status.can_connect());
        let relay = AtomicRelayStatus::new(status);
        assert_eq!(relay.load(), RelayStatus::Pending);
    }

    #[test]
    fn test_status_connecting() {
        let status = RelayStatus::Connecting;
        assert!(!status.is_initialized());
        assert!(!status.is_connected());
        assert!(!status.is_disconnected());
        assert!(!status.is_terminated());
        assert!(!status.is_banned());
        assert!(!status.can_connect());
        let relay = AtomicRelayStatus::new(status);
        assert_eq!(relay.load(), RelayStatus::Connecting);
    }

    #[test]
    fn test_status_connected() {
        let status = RelayStatus::Connected;
        assert!(!status.is_initialized());
        assert!(status.is_connected());
        assert!(!status.is_disconnected());
        assert!(!status.is_terminated());
        assert!(!status.is_banned());
        assert!(!status.can_connect());
        let relay = AtomicRelayStatus::new(status);
        assert_eq!(relay.load(), RelayStatus::Connected);
    }

    #[test]
    fn test_status_disconnected() {
        let status = RelayStatus::Disconnected;
        assert!(!status.is_initialized());
        assert!(!status.is_connected());
        assert!(status.is_disconnected());
        assert!(!status.is_terminated());
        assert!(!status.is_banned());
        assert!(!status.can_connect());
        let relay = AtomicRelayStatus::new(status);
        assert_eq!(relay.load(), RelayStatus::Disconnected);
    }

    #[test]
    fn test_status_terminated() {
        let status = RelayStatus::Terminated;
        assert!(!status.is_initialized());
        assert!(!status.is_connected());
        assert!(status.is_disconnected());
        assert!(status.is_terminated());
        assert!(!status.is_banned());
        assert!(status.can_connect());
        let relay = AtomicRelayStatus::new(status);
        assert_eq!(relay.load(), RelayStatus::Terminated);
    }

    #[test]
    fn test_status_banned() {
        let status = RelayStatus::Banned;
        assert!(!status.is_initialized());
        assert!(!status.is_connected());
        assert!(status.is_disconnected());
        assert!(!status.is_terminated());
        assert!(status.is_banned());
        assert!(!status.can_connect());
        let relay = AtomicRelayStatus::new(status);
        assert_eq!(relay.load(), RelayStatus::Banned);
    }

    #[test]
    fn test_status_sleeping() {
        let status = RelayStatus::Sleeping;
        assert!(!status.is_initialized());
        assert!(!status.is_connected());
        assert!(!status.is_disconnected());
        assert!(!status.is_terminated());
        assert!(!status.is_banned());
        assert!(status.is_sleeping());
        assert!(status.can_connect());
    }
}
