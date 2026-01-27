//! Relay Capabilities
//!
//! This module defines relay capabilities that indicate what a relay can do:
//! - **READ**: Can perform read operations
//! - **WRITE**: Can perform write operations
//! - **GOSSIP**: Automatically added relay for NIP-17/NIP-65 gossip
//! - **DISCOVERY**: Used for discovering relay lists

use core::ops::{BitOr, BitOrAssign, BitXor, BitXorAssign};
use core::sync::atomic::{AtomicU64, Ordering};

/// Relay capabilities
///
/// Represents what operations a relay can perform.
/// Multiple capabilities can be combined using bitwise OR operations.
///
/// # Examples
///
/// ```rust,no_run
/// use nostr_sdk::prelude::*;
///
/// // Default relay with read and write
/// let caps = RelayCapabilities::default();
/// assert!(caps.has_any(RelayCapabilities::READ));
/// assert!(caps.has_any(RelayCapabilities::WRITE));
///
/// // Gossip relay (read and write for NIP-17/65)
/// let gossip = RelayCapabilities::GOSSIP;
///
/// // Discovery-only relay
/// let discovery = RelayCapabilities::DISCOVERY;
///
/// // Combined capabilities
/// let multi = RelayCapabilities::READ | RelayCapabilities::WRITE | RelayCapabilities::DISCOVERY;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelayCapabilities(u64);

impl Default for RelayCapabilities {
    /// Default capabilities: `READ | WRITE`
    #[inline]
    fn default() -> Self {
        Self::READ | Self::WRITE
    }
}

impl RelayCapabilities {
    /// No capabilities
    pub const NONE: Self = Self(0);

    /// Can perform read operations (i.e., fetch/query events)
    pub const READ: Self = Self(1 << 0); // 1

    /// Can perform write operations (i.e., publish events)
    pub const WRITE: Self = Self(1 << 1); // 2

    /// Gossip relay for NIP-17/NIP-65 (implies read/write)
    pub const GOSSIP: Self = Self(1 << 2); // 4

    /// Discovery relay for relay lists (i.e., kind 10002)
    pub const DISCOVERY: Self = Self(1 << 3); // 8

    /// Create new capabilities from raw bits
    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Get raw bits value
    #[inline]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Check if this has **any** of the specified capabilities
    #[inline]
    pub const fn has_any(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Check if this has **all** of the specified capabilities
    #[inline]
    pub const fn has_all(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Add capabilities
    #[inline]
    pub fn add(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Remove capabilities
    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    /// Check if relay can read (has READ, GOSSIP, or DISCOVERY)
    #[inline]
    pub fn can_read(self) -> bool {
        self.has_any(Self::READ | Self::GOSSIP | Self::DISCOVERY)
    }

    /// Check if relay can write (has WRITE or GOSSIP)
    #[inline]
    pub fn can_write(self) -> bool {
        self.has_any(Self::WRITE | Self::GOSSIP)
    }
}

impl BitOr for RelayCapabilities {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self {
        self.add(rhs);
        self
    }
}

impl BitOrAssign for RelayCapabilities {
    fn bitor_assign(&mut self, rhs: Self) {
        self.add(rhs);
    }
}

impl BitXor for RelayCapabilities {
    type Output = Self;

    fn bitxor(mut self, rhs: Self) -> Self {
        self.remove(rhs);
        self
    }
}

impl BitXorAssign for RelayCapabilities {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.remove(rhs);
    }
}

/// Thread-safe atomic relay capabilities
///
/// This type allows safe concurrent access to relay capabilities
/// from multiple threads without requiring a mutex.
///
/// # Examples
///
/// ```rust,no_run
/// use nostr_sdk::prelude::*;
///
/// let caps = AtomicRelayCapabilities::new(RelayCapabilities::READ | RelayCapabilities::WRITE);
///
/// // Check capabilities
/// assert!(caps.can_read());
/// assert!(caps.can_write());
///
/// // Add capability
/// caps.add(RelayCapabilities::GOSSIP);
/// assert!(caps.has_gossip());
///
/// // Remove capability
/// caps.remove(RelayCapabilities::WRITE);
/// assert!(!caps.has_write());
/// ```
#[derive(Debug)]
pub struct AtomicRelayCapabilities(AtomicU64);

impl AtomicRelayCapabilities {
    /// Create new atomic capabilities
    #[inline]
    pub const fn new(capabilities: RelayCapabilities) -> Self {
        Self(AtomicU64::new(capabilities.bits()))
    }

    /// Load current capabilities
    #[inline]
    pub fn load(&self) -> RelayCapabilities {
        let bits: u64 = self.0.load(Ordering::SeqCst);
        RelayCapabilities::from_bits(bits)
    }

    /// Store new capabilities
    #[inline]
    pub fn store(&self, capabilities: RelayCapabilities) {
        self.0.store(capabilities.bits(), Ordering::SeqCst);
    }

    /// Add capabilities (atomic OR operation)
    #[inline]
    pub fn add(&self, other: RelayCapabilities) {
        self.0.fetch_or(other.bits(), Ordering::SeqCst);
    }

    /// Remove capabilities (atomic AND NOT operation)
    #[inline]
    pub fn remove(&self, other: RelayCapabilities) {
        self.0.fetch_and(!other.bits(), Ordering::SeqCst);
    }

    /// Check if has **any** of the specified capabilities
    #[inline]
    pub fn has_any(&self, capabilities: RelayCapabilities) -> bool {
        self.load().has_any(capabilities)
    }

    /// Check if has **all** of the specified capabilities
    #[inline]
    pub fn has_all(&self, capabilities: RelayCapabilities) -> bool {
        self.load().has_all(capabilities)
    }

    /// Check if `READ` capability is enabled
    #[inline]
    pub fn has_read(&self) -> bool {
        self.has_all(RelayCapabilities::READ)
    }

    /// Check if `WRITE` capability is enabled
    #[inline]
    pub fn has_write(&self) -> bool {
        self.has_all(RelayCapabilities::WRITE)
    }

    /// Check if `GOSSIP` capability is enabled
    #[inline]
    pub fn has_gossip(&self) -> bool {
        self.has_all(RelayCapabilities::GOSSIP)
    }

    /// Check if `DISCOVERY` capability is enabled
    #[inline]
    pub fn has_discovery(&self) -> bool {
        self.has_all(RelayCapabilities::DISCOVERY)
    }

    /// Check if relay can read (has READ, GOSSIP, or DISCOVERY)
    #[inline]
    pub fn can_read(&self) -> bool {
        self.load().can_read()
    }

    /// Check if relay can write (has WRITE or GOSSIP)
    #[inline]
    pub fn can_write(&self) -> bool {
        self.load().can_write()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let caps = RelayCapabilities::default();
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(caps.has_all(RelayCapabilities::WRITE));
        assert!(!caps.has_any(RelayCapabilities::GOSSIP));
        assert!(!caps.has_any(RelayCapabilities::DISCOVERY));
    }

    #[test]
    fn test_none() {
        let caps = RelayCapabilities::NONE;
        assert!(!caps.has_any(RelayCapabilities::READ));
        assert!(!caps.has_any(RelayCapabilities::WRITE));
        assert!(!caps.has_any(RelayCapabilities::GOSSIP));
        assert!(!caps.has_any(RelayCapabilities::DISCOVERY));
        assert!(!caps.can_read());
        assert!(!caps.can_write());
    }

    #[test]
    fn test_const_values() {
        assert_eq!(RelayCapabilities::NONE.bits(), 0);
        assert_eq!(RelayCapabilities::READ.bits(), 1);
        assert_eq!(RelayCapabilities::WRITE.bits(), 2);
        assert_eq!(RelayCapabilities::GOSSIP.bits(), 4);
        assert_eq!(RelayCapabilities::DISCOVERY.bits(), 8);
    }

    #[test]
    fn test_from_bits() {
        let caps = RelayCapabilities::from_bits(3); // READ | WRITE
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(caps.has_all(RelayCapabilities::WRITE));
        assert!(!caps.has_any(RelayCapabilities::GOSSIP));
    }

    #[test]
    fn test_bitor() {
        let caps = RelayCapabilities::READ | RelayCapabilities::WRITE;
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(caps.has_all(RelayCapabilities::WRITE));

        let caps = RelayCapabilities::GOSSIP | RelayCapabilities::DISCOVERY;
        assert!(caps.has_all(RelayCapabilities::GOSSIP));
        assert!(caps.has_all(RelayCapabilities::DISCOVERY));
    }

    #[test]
    fn test_bitxor() {
        let mut caps =
            RelayCapabilities::READ | RelayCapabilities::WRITE | RelayCapabilities::GOSSIP;
        caps ^= RelayCapabilities::WRITE;
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(!caps.has_any(RelayCapabilities::WRITE));
        assert!(caps.has_all(RelayCapabilities::GOSSIP));
    }

    #[test]
    fn test_has_any() {
        let caps = RelayCapabilities::READ | RelayCapabilities::WRITE;
        assert!(caps.has_any(RelayCapabilities::READ));
        assert!(caps.has_any(RelayCapabilities::WRITE));
        assert!(caps.has_any(RelayCapabilities::READ | RelayCapabilities::GOSSIP));
        assert!(!caps.has_any(RelayCapabilities::GOSSIP));
        assert!(!caps.has_any(RelayCapabilities::DISCOVERY));
    }

    #[test]
    fn test_has_all() {
        let caps = RelayCapabilities::READ | RelayCapabilities::WRITE;
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(caps.has_all(RelayCapabilities::WRITE));
        assert!(caps.has_all(RelayCapabilities::READ | RelayCapabilities::WRITE));
        assert!(!caps.has_all(RelayCapabilities::READ | RelayCapabilities::GOSSIP));
        assert!(!caps.has_all(RelayCapabilities::GOSSIP));
    }

    #[test]
    fn test_add() {
        let mut caps = RelayCapabilities::READ;
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(!caps.has_any(RelayCapabilities::WRITE));

        caps.add(RelayCapabilities::WRITE);
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(caps.has_all(RelayCapabilities::WRITE));

        // Adding multiple at once
        caps.add(RelayCapabilities::GOSSIP | RelayCapabilities::DISCOVERY);
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(caps.has_all(RelayCapabilities::WRITE));
        assert!(caps.has_all(RelayCapabilities::GOSSIP));
        assert!(caps.has_all(RelayCapabilities::DISCOVERY));

        // Re-adding existing should be idempotent
        caps.add(RelayCapabilities::READ);
        assert_eq!(caps.bits(), 15); // All capabilities
    }

    #[test]
    fn test_remove() {
        let mut caps =
            RelayCapabilities::READ | RelayCapabilities::WRITE | RelayCapabilities::GOSSIP;

        caps.remove(RelayCapabilities::WRITE);
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(!caps.has_any(RelayCapabilities::WRITE));
        assert!(caps.has_all(RelayCapabilities::GOSSIP));

        // Removing multiple at once
        caps.add(RelayCapabilities::WRITE | RelayCapabilities::DISCOVERY);
        caps.remove(RelayCapabilities::WRITE | RelayCapabilities::GOSSIP);
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(!caps.has_any(RelayCapabilities::WRITE));
        assert!(!caps.has_any(RelayCapabilities::GOSSIP));
        assert!(caps.has_all(RelayCapabilities::DISCOVERY));

        // Re-removing should be idempotent
        caps.remove(RelayCapabilities::WRITE);
        assert!(!caps.has_any(RelayCapabilities::WRITE));
    }

    #[test]
    fn test_can_read() {
        assert!(!RelayCapabilities::NONE.can_read());
        assert!(RelayCapabilities::READ.can_read());
        assert!(!RelayCapabilities::WRITE.can_read());
        assert!(RelayCapabilities::GOSSIP.can_read());
        assert!(RelayCapabilities::DISCOVERY.can_read());
        assert!((RelayCapabilities::READ | RelayCapabilities::WRITE).can_read());
    }

    #[test]
    fn test_can_write() {
        assert!(!RelayCapabilities::NONE.can_write());
        assert!(!RelayCapabilities::READ.can_write());
        assert!(RelayCapabilities::WRITE.can_write());
        assert!(RelayCapabilities::GOSSIP.can_write());
        assert!(!RelayCapabilities::DISCOVERY.can_write());
        assert!((RelayCapabilities::READ | RelayCapabilities::WRITE).can_write());
    }

    // #[test]
    // fn test_debug_format() {
    //     let caps = RelayCapabilities::NONE;
    //     assert_eq!(format!("{:?}", caps), "RelayCapabilities(0x0)");
    //
    //     let caps = RelayCapabilities::READ;
    //     assert_eq!(format!("{:?}", caps), "RelayCapabilities(0x1)");
    //
    //     let caps = RelayCapabilities::READ | RelayCapabilities::WRITE;
    //     assert_eq!(format!("{:?}", caps), "RelayCapabilities(0x3)");
    //
    //     let caps = RelayCapabilities::READ | RelayCapabilities::WRITE | RelayCapabilities::GOSSIP | RelayCapabilities::DISCOVERY;
    //     assert_eq!(format!("{:?}", caps), "RelayCapabilities(0xf)");
    // }

    // #[test]
    // fn test_display_format() {
    //     let caps = RelayCapabilities::NONE;
    //     assert_eq!(format!("{}", caps), "NONE");
    //
    //     let caps = RelayCapabilities::READ;
    //     assert_eq!(format!("{}", caps), "READ");
    //
    //     let caps = RelayCapabilities::READ | RelayCapabilities::WRITE;
    //     assert_eq!(format!("{}", caps), "READ | WRITE");
    //
    //     let caps = RelayCapabilities::READ | RelayCapabilities::WRITE | RelayCapabilities::GOSSIP | RelayCapabilities::DISCOVERY;
    //     assert_eq!(format!("{}", caps), "READ | WRITE | GOSSIP | DISCOVERY");
    // }

    #[test]
    fn test_atomic_new() {
        let caps = AtomicRelayCapabilities::new(RelayCapabilities::GOSSIP);
        assert!(!caps.has_read());
        assert!(!caps.has_write());
        assert!(caps.has_gossip());
        assert!(caps.can_read());
        assert!(caps.can_write());
    }

    #[test]
    fn test_atomic_load_store() {
        let caps = AtomicRelayCapabilities::new(RelayCapabilities::READ);
        assert_eq!(caps.load().bits(), 1);

        caps.store(RelayCapabilities::WRITE | RelayCapabilities::GOSSIP);
        assert!(!caps.has_read());
        assert!(caps.has_write());
        assert!(caps.has_gossip());
    }

    #[test]
    fn test_atomic_add() {
        let caps = AtomicRelayCapabilities::new(RelayCapabilities::READ);

        caps.add(RelayCapabilities::WRITE);
        assert!(caps.has_read());
        assert!(caps.has_write());

        caps.add(RelayCapabilities::GOSSIP | RelayCapabilities::DISCOVERY);
        assert!(caps.has_read());
        assert!(caps.has_write());
        assert!(caps.has_gossip());
        assert!(caps.has_discovery());
    }

    #[test]
    fn test_atomic_remove() {
        let caps = AtomicRelayCapabilities::new(
            RelayCapabilities::READ | RelayCapabilities::WRITE | RelayCapabilities::GOSSIP,
        );

        caps.remove(RelayCapabilities::WRITE);
        assert!(caps.has_read());
        assert!(!caps.has_write());
        assert!(caps.has_gossip());

        caps.remove(RelayCapabilities::READ | RelayCapabilities::GOSSIP);
        assert!(!caps.has_read());
        assert!(!caps.has_gossip());
    }

    #[test]
    fn test_atomic_has_any() {
        let caps = AtomicRelayCapabilities::new(RelayCapabilities::READ | RelayCapabilities::WRITE);
        assert!(caps.has_any(RelayCapabilities::READ));
        assert!(caps.has_any(RelayCapabilities::WRITE));
        assert!(caps.has_any(RelayCapabilities::READ | RelayCapabilities::GOSSIP));
        assert!(!caps.has_any(RelayCapabilities::GOSSIP));
    }

    #[test]
    fn test_atomic_has_all() {
        let caps = AtomicRelayCapabilities::new(RelayCapabilities::READ | RelayCapabilities::WRITE);
        assert!(caps.has_all(RelayCapabilities::READ));
        assert!(caps.has_all(RelayCapabilities::WRITE));
        assert!(caps.has_all(RelayCapabilities::READ | RelayCapabilities::WRITE));
        assert!(!caps.has_all(RelayCapabilities::READ | RelayCapabilities::GOSSIP));
    }

    #[test]
    fn test_atomic_can_read() {
        assert!(!AtomicRelayCapabilities::new(RelayCapabilities::NONE).can_read());
        assert!(AtomicRelayCapabilities::new(RelayCapabilities::READ).can_read());
        assert!(!AtomicRelayCapabilities::new(RelayCapabilities::WRITE).can_read());
        assert!(AtomicRelayCapabilities::new(RelayCapabilities::GOSSIP).can_read());
        assert!(AtomicRelayCapabilities::new(RelayCapabilities::DISCOVERY).can_read());
    }

    #[test]
    fn test_atomic_can_write() {
        assert!(!AtomicRelayCapabilities::new(RelayCapabilities::NONE).can_write());
        assert!(!AtomicRelayCapabilities::new(RelayCapabilities::READ).can_write());
        assert!(AtomicRelayCapabilities::new(RelayCapabilities::WRITE).can_write());
        assert!(AtomicRelayCapabilities::new(RelayCapabilities::GOSSIP).can_write());
        assert!(!AtomicRelayCapabilities::new(RelayCapabilities::DISCOVERY).can_write());
    }

    #[test]
    fn test_atomic_individual_capabilities() {
        let caps =
            AtomicRelayCapabilities::new(RelayCapabilities::READ | RelayCapabilities::GOSSIP);
        assert!(caps.has_read());
        assert!(!caps.has_write());
        assert!(caps.has_gossip());
        assert!(!caps.has_discovery());
    }

    // #[test]
    // fn test_atomic_debug_format() {
    //     let caps = AtomicRelayCapabilities::new(RelayCapabilities::READ | RelayCapabilities::WRITE);
    //     let debug_str = format!("{:?}", caps);
    //     assert!(debug_str.contains("AtomicRelayCapabilities"));
    //     assert!(debug_str.contains("0x3")); // READ | WRITE = 0x3
    // }
}
