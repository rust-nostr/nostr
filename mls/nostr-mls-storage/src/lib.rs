//! Nostr MLS storage - A set of storage provider traits and types for implementing MLS storage
//! It is designed to be used in conjunction with the `openmls` crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use openmls_traits::storage::StorageProvider;

pub mod groups;
pub mod messages;
pub mod welcomes;

use self::groups::GroupStorage;
use self::messages::MessageStorage;
use self::welcomes::WelcomeStorage;

const CURRENT_VERSION: u16 = 1;

/// Backend
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    /// Memory
    Memory,
    /// SQLite
    SQLite,
}

impl Backend {
    /// Check if it's a persistent backend
    ///
    /// All values different from [`Backend::Memory`] are considered persistent
    pub fn is_persistent(&self) -> bool {
        !matches!(self, Self::Memory)
    }
}

/// Storage provider for the Nostr MLS storage
pub trait NostrMlsStorageProvider: GroupStorage + MessageStorage + WelcomeStorage {
    /// The OpenMLS storage provider
    type OpenMlsStorageProvider: StorageProvider<CURRENT_VERSION>;

    /// Returns the backend type.
    ///
    /// # Returns
    ///
    /// [`Backend::Memory`] indicating this is a memory-based storage implementation.
    fn backend(&self) -> Backend;

    /// Get a reference to the openmls storage provider.
    ///
    /// This method provides access to the underlying OpenMLS storage provider.
    /// This is primarily useful for internal operations and testing.
    ///
    /// # Returns
    ///
    /// A reference to the openmls storage implementation.
    fn openmls_storage(&self) -> &Self::OpenMlsStorageProvider;

    /// Get a mutable reference to the openmls storage provider.
    ///
    /// This method provides mutable access to the underlying OpenMLS storage provider.
    /// This is primarily useful for internal operations and testing.
    ///
    /// # Returns
    ///
    /// A mutable reference to the openmls storage implementation.
    fn openmls_storage_mut(&mut self) -> &mut Self::OpenMlsStorageProvider;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_is_persistent() {
        assert!(!Backend::Memory.is_persistent());
        assert!(Backend::SQLite.is_persistent());
    }
}
