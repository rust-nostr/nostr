// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Service Flags

use std::ops::{BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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
    pub const NONE: Self = Self(0);

    /// READ means that client will perform read operations with relay.
    pub const READ: Self = Self(1 << 0);

    /// WRITE means that client will perform write operations with relay.
    pub const WRITE: Self = Self(1 << 1);

    /// PING means that
    pub const PING: Self = Self(1 << 2);

    /// Add [RelayServiceFlags] together.
    pub fn add(&mut self, other: Self) -> Self {
        self.0 |= other.0;
        *self
    }

    /// Remove [RelayServiceFlags] from this.
    pub fn remove(&mut self, other: Self) -> Self {
        self.0 ^= other.0;
        *self
    }

    fn has(self, flags: Self) -> bool {
        (self.0 | flags.0) == self.0
    }

    fn to_u64(self) -> u64 {
        self.0
    }
}

impl BitOr for RelayServiceFlags {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self {
        self.add(rhs)
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
        self.remove(rhs)
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
                let new = f.add(other);
                Some(new.to_u64())
            });
    }

    /// Remove [RelayServiceFlags] from this.
    pub fn remove(&self, other: RelayServiceFlags) {
        let _ = self
            .flags
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |f| {
                let mut f: RelayServiceFlags = RelayServiceFlags(f);
                let new = f.remove(other);
                Some(new.to_u64())
            });
    }

    /// Check whether [RelayServiceFlags] are included in this one.
    pub fn has(&self, flags: RelayServiceFlags) -> bool {
        let _f: u64 = self.flags.load(Ordering::SeqCst);
        let f: RelayServiceFlags = RelayServiceFlags(_f);
        f.has(flags)
    }

    /// Check if `READ` service is enabled
    pub fn has_read(&self) -> bool {
        self.has(RelayServiceFlags::READ)
    }

    /// Check if `WRITE` service is enabled
    pub fn has_write(&self) -> bool {
        self.has(RelayServiceFlags::WRITE)
    }

    /// Check if `PING` service is enabled
    pub fn has_ping(&self) -> bool {
        self.has(RelayServiceFlags::PING)
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
            assert!(!flags.has(f));
        }

        let mut flags = RelayServiceFlags::READ | RelayServiceFlags::WRITE;
        assert!(flags.has(RelayServiceFlags::READ));
        assert!(flags.has(RelayServiceFlags::WRITE));
        assert!(!flags.has(RelayServiceFlags::PING));

        // Try to add flag
        flags.add(RelayServiceFlags::PING);
        assert!(flags.has(RelayServiceFlags::PING));

        // Try to remove flag
        flags.remove(RelayServiceFlags::WRITE);
        assert!(flags.has(RelayServiceFlags::READ));
        assert!(flags.has(RelayServiceFlags::PING));

        // Try to re-add already existing flag
        flags.add(RelayServiceFlags::PING);
        assert!(flags.has(RelayServiceFlags::READ));
        assert!(flags.has(RelayServiceFlags::PING));
    }
}
