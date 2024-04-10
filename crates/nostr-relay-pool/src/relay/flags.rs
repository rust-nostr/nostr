// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Service Flags

use std::ops::{BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Flag checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlagCheck {
    /// Use `OR` logic operator
    Any,
    /// Use `AND` logic operator
    All,
}

/// Relay Service Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelayServiceFlags(u64);

impl Default for RelayServiceFlags {
    /// Default flags: `READ`, `WRITE` and `PING`
    fn default() -> Self {
        Self::READ | Self::WRITE | Self::PING
    }
}

impl RelayServiceFlags {
    /// NONE means no services supported.
    pub const NONE: Self = Self(0); // 0

    /// READ means that client will perform read operations with relay.
    pub const READ: Self = Self(1 << 0); // 1

    /// WRITE means that client will perform write operations with relay.
    pub const WRITE: Self = Self(1 << 1); // 2

    /// PING means that client will ping relay to keep connection up
    pub const PING: Self = Self(1 << 2); // 4

    /// INBOX means READ of kind 10002
    pub const INBOX: Self = Self(1 << 3); // 8

    /// OUTBOX means WRITE of kind 10002
    pub const OUTBOX: Self = Self(1 << 4); // 16

    /// DISCOVERY means that relay has role to get metadata (i.e. events with kind 0 or 10002) of public keys
    pub const DISCOVERY: Self = Self(1 << 5); // 32

    /// Add [RelayServiceFlags] together.
    #[inline]
    pub fn add(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Remove [RelayServiceFlags] from this.
    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.0 ^= other.0;
    }

    #[inline]
    fn has(self, flags: Self, check: FlagCheck) -> bool {
        match check {
            FlagCheck::Any => (self.0 & flags.0) != 0,
            FlagCheck::All => (self.0 | flags.0) == self.0,
        }
    }

    #[inline]
    fn to_u64(self) -> u64 {
        self.0
    }
}

impl BitOr for RelayServiceFlags {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self {
        self.add(rhs);
        self
    }
}

impl BitOrAssign for RelayServiceFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.add(rhs);
    }
}

impl BitXor for RelayServiceFlags {
    type Output = Self;

    fn bitxor(mut self, rhs: Self) -> Self {
        self.remove(rhs);
        self
    }
}

impl BitXorAssign for RelayServiceFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.remove(rhs);
    }
}

/// Realy Service Flags which can be safely shared between threads.
#[derive(Debug, Clone)]
pub struct AtomicRelayServiceFlags {
    flags: Arc<AtomicU64>,
}

impl Default for AtomicRelayServiceFlags {
    fn default() -> Self {
        Self::new(RelayServiceFlags::default())
    }
}

impl AtomicRelayServiceFlags {
    /// Compose new from [RelayServiceFlags]
    #[inline]
    pub fn new(flags: RelayServiceFlags) -> Self {
        Self {
            flags: Arc::new(AtomicU64::new(flags.to_u64())),
        }
    }

    /// Add [RelayServiceFlags] together.
    pub fn add(&self, other: RelayServiceFlags) {
        let _ = self
            .flags
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |f| {
                let mut f: RelayServiceFlags = RelayServiceFlags(f);
                f.add(other);
                Some(f.to_u64())
            });
    }

    /// Remove [RelayServiceFlags] from this.
    pub fn remove(&self, other: RelayServiceFlags) {
        let _ = self
            .flags
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |f| {
                let mut f: RelayServiceFlags = RelayServiceFlags(f);
                f.remove(other);
                Some(f.to_u64())
            });
    }

    /// Check whether [RelayServiceFlags] are included in this one.
    pub fn has(&self, flags: RelayServiceFlags, check: FlagCheck) -> bool {
        let _f: u64 = self.flags.load(Ordering::SeqCst);
        let f: RelayServiceFlags = RelayServiceFlags(_f);
        f.has(flags, check)
    }

    /// Check if [RelayServiceFlags] has **any** of the passed flags.
    #[inline]
    pub fn has_any(&self, flags: RelayServiceFlags) -> bool {
        self.has(flags, FlagCheck::Any)
    }

    /// Check if [RelayServiceFlags] has **all** the passed flags.
    #[inline]
    pub fn has_all(&self, flags: RelayServiceFlags) -> bool {
        self.has(flags, FlagCheck::All)
    }

    /// Check if `READ` service is enabled
    #[inline]
    pub fn has_read(&self) -> bool {
        self.has_all(RelayServiceFlags::READ)
    }

    /// Check if `WRITE` service is enabled
    #[inline]
    pub fn has_write(&self) -> bool {
        self.has_all(RelayServiceFlags::WRITE)
    }

    /// Check if `PING` service is enabled
    #[inline]
    pub fn has_ping(&self) -> bool {
        self.has_all(RelayServiceFlags::PING)
    }

    /// Check if `INBOX` service is enabled
    #[inline]
    pub fn has_inbox(&self) -> bool {
        self.has_all(RelayServiceFlags::INBOX)
    }

    /// Check if `OUTBOX` service is enabled
    #[inline]
    pub fn has_outbox(&self) -> bool {
        self.has_all(RelayServiceFlags::OUTBOX)
    }

    /// Check if `DISCOVERY` service is enabled
    #[inline]
    pub fn has_discovery(&self) -> bool {
        self.has_all(RelayServiceFlags::DISCOVERY)
    }

    /// Check if `READ`, `INBOX`, `OUTBOX` or `DISCOVERY` services are enabled
    #[inline]
    pub fn can_read(&self) -> bool {
        self.has_any(
            RelayServiceFlags::READ
                | RelayServiceFlags::INBOX
                | RelayServiceFlags::OUTBOX
                | RelayServiceFlags::DISCOVERY,
        )
    }

    /// Check if `WRITE`, `INBOX` or `OUTBOX` services are enabled
    #[inline]
    pub fn can_write(&self) -> bool {
        self.has_any(
            RelayServiceFlags::WRITE | RelayServiceFlags::INBOX | RelayServiceFlags::OUTBOX,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_flags() {
        let all = [
            RelayServiceFlags::READ,
            RelayServiceFlags::WRITE,
            RelayServiceFlags::PING,
        ];

        let flags = RelayServiceFlags::NONE;
        for f in all.into_iter() {
            assert!(!flags.has(f, FlagCheck::All));
        }

        let mut flags = RelayServiceFlags::READ | RelayServiceFlags::WRITE;
        assert!(flags.has(RelayServiceFlags::READ, FlagCheck::All));
        assert!(flags.has(RelayServiceFlags::WRITE, FlagCheck::All));
        assert!(!flags.has(RelayServiceFlags::PING, FlagCheck::All));
        assert!(!flags.has(RelayServiceFlags::INBOX, FlagCheck::All));
        assert!(!flags.has(RelayServiceFlags::OUTBOX, FlagCheck::All));
        assert!(!flags.has(RelayServiceFlags::DISCOVERY, FlagCheck::All));

        // Try to add flag
        flags.add(RelayServiceFlags::PING);
        assert!(flags.has(
            RelayServiceFlags::PING | RelayServiceFlags::READ | RelayServiceFlags::WRITE,
            FlagCheck::All
        ));

        // Try to remove flag
        flags.remove(RelayServiceFlags::WRITE);
        assert!(flags.has(
            RelayServiceFlags::PING | RelayServiceFlags::READ,
            FlagCheck::All
        ));

        // Try to re-add already existing flag
        flags.add(RelayServiceFlags::PING);
        assert!(flags.has(
            RelayServiceFlags::PING | RelayServiceFlags::READ,
            FlagCheck::All
        ));

        // Try to re-add already existing flag + new one
        flags.add(RelayServiceFlags::PING | RelayServiceFlags::WRITE);
        assert!(flags.has(
            RelayServiceFlags::PING | RelayServiceFlags::READ | RelayServiceFlags::WRITE,
            FlagCheck::All
        ));

        // Try to add flag
        flags.add(RelayServiceFlags::INBOX);
        assert!(flags.has(RelayServiceFlags::INBOX, FlagCheck::All));

        // Try to add flag
        flags.add(RelayServiceFlags::OUTBOX);
        assert!(flags.has(RelayServiceFlags::OUTBOX, FlagCheck::All));

        // Try to add flag
        flags.add(RelayServiceFlags::DISCOVERY);
        assert!(flags.has(RelayServiceFlags::DISCOVERY, FlagCheck::All));

        let flags = RelayServiceFlags::READ | RelayServiceFlags::INBOX | RelayServiceFlags::PING;
        assert!(flags.has(
            RelayServiceFlags::READ | RelayServiceFlags::INBOX,
            FlagCheck::All
        ));
        assert!(!flags.has(
            RelayServiceFlags::READ | RelayServiceFlags::WRITE,
            FlagCheck::All
        ));
    }

    #[test]
    fn test_service_flags_can_read() {
        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::WRITE);
        assert!(!f.can_read());

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::READ | RelayServiceFlags::WRITE);
        assert!(f.can_read());

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::INBOX);
        assert!(f.can_read());

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::DISCOVERY);
        assert!(f.can_read());
    }

    #[test]
    fn test_service_flags_can_write() {
        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::WRITE);
        assert!(f.can_write());

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::READ | RelayServiceFlags::WRITE);
        assert!(f.can_write());

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::INBOX);
        assert!(!f.can_write());

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::DISCOVERY);
        assert!(!f.can_write());
    }
}
