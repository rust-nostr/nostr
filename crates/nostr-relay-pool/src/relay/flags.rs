// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
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

    /// READ means a manually added relay that will perform read operations.
    pub const READ: Self = Self(1 << 0); // 1

    /// WRITE means a manually added relay that will perform write operations.
    pub const WRITE: Self = Self(1 << 1); // 2

    /// PING means that client will ping relay to keep connection up.
    pub const PING: Self = Self(1 << 2); // 4

    /// GOSSIP means automatically added relay that will perform read/write operations.
    ///
    /// Use for NIP17, NIP65 or similar NIPs.
    pub const GOSSIP: Self = Self(1 << 3); // 8

    /// DISCOVERY means that relay has role to get relay lists (i.e., events with kind `10002`) of public keys.
    pub const DISCOVERY: Self = Self(1 << 4); // 16

    /// Add service flags together.
    #[inline]
    pub fn add(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Remove service flags from this one.
    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.0 ^= other.0;
    }

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

/// Relay Service Flags which can be safely shared between threads.
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
    pub fn has_any(&self, flags: RelayServiceFlags) -> bool {
        self.has(flags, FlagCheck::Any)
    }

    /// Check if [RelayServiceFlags] has **all** the passed flags.
    pub fn has_all(&self, flags: RelayServiceFlags) -> bool {
        self.has(flags, FlagCheck::All)
    }

    /// Check if `READ` service is enabled
    pub fn has_read(&self) -> bool {
        self.has_all(RelayServiceFlags::READ)
    }

    /// Check if `WRITE` service is enabled
    pub fn has_write(&self) -> bool {
        self.has_all(RelayServiceFlags::WRITE)
    }

    /// Check if `PING` service is enabled
    pub fn has_ping(&self) -> bool {
        self.has_all(RelayServiceFlags::PING)
    }

    /// Check if `GOSSIP` service is enabled
    pub fn has_gossip(&self) -> bool {
        self.has_all(RelayServiceFlags::GOSSIP)
    }

    /// Check if `DISCOVERY` service is enabled
    pub fn has_discovery(&self) -> bool {
        self.has_all(RelayServiceFlags::DISCOVERY)
    }

    /// Check if `READ`, `GOSSIP` or `DISCOVERY` services are enabled
    pub fn can_read(&self) -> bool {
        self.has_any(
            RelayServiceFlags::READ | RelayServiceFlags::GOSSIP | RelayServiceFlags::DISCOVERY,
        )
    }

    /// Check if `WRITE` or `GOSSIP` services are enabled
    pub fn can_write(&self) -> bool {
        self.has_any(RelayServiceFlags::WRITE | RelayServiceFlags::GOSSIP)
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
        assert!(!flags.has(RelayServiceFlags::GOSSIP, FlagCheck::All));
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

        // Try to remove multiple flags
        flags.add(RelayServiceFlags::WRITE | RelayServiceFlags::DISCOVERY);
        flags.remove(
            RelayServiceFlags::WRITE | RelayServiceFlags::GOSSIP | RelayServiceFlags::DISCOVERY,
        );
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
        flags.add(RelayServiceFlags::GOSSIP);
        assert!(flags.has(RelayServiceFlags::GOSSIP, FlagCheck::All));

        // Try to add flag
        flags.add(RelayServiceFlags::DISCOVERY);
        assert!(flags.has(RelayServiceFlags::DISCOVERY, FlagCheck::All));

        let flags = RelayServiceFlags::READ | RelayServiceFlags::GOSSIP | RelayServiceFlags::PING;
        assert!(flags.has(
            RelayServiceFlags::READ | RelayServiceFlags::GOSSIP,
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

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::GOSSIP);
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

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::GOSSIP);
        assert!(f.can_write());

        let f = AtomicRelayServiceFlags::new(RelayServiceFlags::DISCOVERY);
        assert!(!f.can_write());
    }
}
