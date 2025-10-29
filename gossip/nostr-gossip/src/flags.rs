//! Gossip flags

/// Gossip flags
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GossipFlags(u16);

impl GossipFlags {
    /// Empty flags.
    pub const NONE: Self = Self(0); // 0

    /// Read flag.
    pub const READ: Self = Self(1 << 0); // 1

    /// Write flag.
    pub const WRITE: Self = Self(1 << 1); // 2

    /// Private message (NIP-17) flag.
    pub const PRIVATE_MESSAGE: Self = Self(1 << 2); // 4

    /// Hint flag.
    pub const HINT: Self = Self(1 << 3); // 8

    /// Received flag.
    pub const RECEIVED: Self = Self(1 << 4); // 16

    /// New empty flags.
    #[inline]
    pub const fn new() -> Self {
        Self::NONE
    }

    /// Add flag.
    #[inline]
    pub const fn add(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Remove flag.
    #[inline]
    pub const fn remove(&mut self, other: Self) {
        self.0 ^= other.0;
    }

    /// Check if has flag.
    #[inline]
    pub const fn has(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Get flags as [`u16`].
    #[inline]
    pub const fn as_u16(&self) -> u16 {
        self.0
    }
}
