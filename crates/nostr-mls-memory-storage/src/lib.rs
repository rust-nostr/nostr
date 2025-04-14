/// Memory-based storage implementation for Nostr MLS.
///
/// This module provides a memory-based storage implementation for the Nostr MLS (Messaging Layer Security)
/// crate. It implements the [`NostrMlsStorageProvider`] trait, allowing it to be used within the Nostr MLS context.
///
/// Memory-based storage is non-persistent and will be cleared when the application terminates.
/// It's useful for testing or ephemeral applications where persistence isn't required.
///
mod groups;
mod invites;
mod messages;

use nostr_mls_storage::Backend;
use nostr_mls_storage::NostrMlsStorageProvider;
use openmls_traits::storage::StorageProvider;

const CURRENT_VERSION: u16 = 1;

/// A memory-based storage implementation for Nostr MLS.
///
/// This struct wraps an OpenMLS storage implementation to provide memory-based
/// storage functionality for Nostr MLS operations.
pub struct NostrMlsMemoryStorage<S>
where
    S: StorageProvider<CURRENT_VERSION>,
{
    /// The underlying storage implementation that conforms to OpenMLS's [`StorageProvider`]
    openmls_storage: S,
}

impl<S> NostrMlsMemoryStorage<S>
where
    S: StorageProvider<CURRENT_VERSION>,
{
    /// Creates a new [`NostrMlsMemoryStorage`] with the provided storage implementation.
    ///
    /// # Arguments
    ///
    /// * `storage_implementation` - An implementation of the OpenMLS [`StorageProvider`] trait.
    ///
    /// # Returns
    ///
    /// A new instance of [`NostrMlsMemoryStorage`] wrapping the provided storage implementation.
    pub fn new(storage_implementation: S) -> Self {
        NostrMlsMemoryStorage {
            openmls_storage: storage_implementation,
        }
    }
}

/// Implementation of [`NostrMlsStorageProvider`] for memory-based storage.
impl<S> NostrMlsStorageProvider<S> for NostrMlsMemoryStorage<S>
where
    S: StorageProvider<CURRENT_VERSION>,
{
    /// Returns the backend type.
    ///
    /// # Returns
    ///
    /// [`Backend::Memory`] indicating this is a memory-based storage implementation.
    fn backend(&self) -> Backend {
        Backend::Memory
    }

    /// Get a reference to the openmls storage provider.
    ///
    /// This method provides access to the underlying OpenMLS storage provider.
    /// This is primarily useful for internal operations and testing.
    ///
    /// # Returns
    ///
    /// A reference to the openmls storage implementation.
    fn openmls_storage(&self) -> &S {
        &self.openmls_storage
    }

    /// Get a mutable reference to the openmls storage provider.
    ///
    /// This method provides mutable access to the underlying OpenMLS storage provider.
    /// This is primarily useful for internal operations and testing.
    ///
    /// # Returns
    ///
    /// A mutable reference to the openmls storage implementation.
    fn openmls_storage_mut(&mut self) -> &mut S {
        &mut self.openmls_storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    use openmls_memory_storage::MemoryStorage;

    #[test]
    fn test_new_with_storage() {
        let storage = MemoryStorage::default();
        let nostr_storage = NostrMlsMemoryStorage::new(storage);
        assert_eq!(nostr_storage.backend(), Backend::Memory);
    }

    #[test]
    fn test_backend_type() {
        let storage = MemoryStorage::default();
        let nostr_storage = NostrMlsMemoryStorage::new(storage);
        assert_eq!(nostr_storage.backend(), Backend::Memory);
        assert!(!nostr_storage.backend().is_persistent());
    }

    #[test]
    fn test_storage_is_memory_based() {
        let storage = MemoryStorage::default();
        let nostr_storage = NostrMlsMemoryStorage::new(storage);
        assert!(!nostr_storage.backend().is_persistent());
    }

    #[test]
    fn test_compare_backend_types() {
        let storage = MemoryStorage::default();
        let nostr_storage = NostrMlsMemoryStorage::new(storage);
        let memory_backend = nostr_storage.backend();
        assert_eq!(memory_backend, Backend::Memory);
        assert_ne!(memory_backend, Backend::SQLite);
    }

    #[test]
    fn test_create_multiple_instances() {
        let storage1 = MemoryStorage::default();
        let storage2 = MemoryStorage::default();
        let nostr_storage1 = NostrMlsMemoryStorage::new(storage1);
        let nostr_storage2 = NostrMlsMemoryStorage::new(storage2);

        assert_eq!(nostr_storage1.backend(), nostr_storage2.backend());
        assert_eq!(nostr_storage1.backend(), Backend::Memory);
        assert_eq!(nostr_storage2.backend(), Backend::Memory);
    }
}
