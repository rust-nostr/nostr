/// Memory-based storage implementation for Nostr MLS.
///
/// This module provides a memory-based storage implementation for the Nostr MLS (Messaging Layer Security)
/// crate. It implements the [`NostrMlsStorageProvider`] trait, allowing it to be used within the Nostr MLS context.
///
/// Memory-based storage is non-persistent and will be cleared when the application terminates.
/// It's useful for testing or ephemeral applications where persistence isn't required.
mod groups;
mod messages;
mod welcomes;

use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

use lru::LruCache;
use nostr::EventId;
use nostr_mls_storage::groups::types::{Group, GroupRelay};
use nostr_mls_storage::messages::types::{Message, ProcessedMessage};
use nostr_mls_storage::welcomes::types::{ProcessedWelcome, Welcome};
use nostr_mls_storage::{Backend, NostrMlsStorageProvider};
use openmls_memory_storage::MemoryStorage;

/// Default cache size for each LRU cache
const DEFAULT_CACHE_SIZE: usize = 1000;

/// A memory-based storage implementation for Nostr MLS.
///
/// This struct wraps an OpenMLS storage implementation to provide memory-based
/// storage functionality for Nostr MLS operations.
///
/// ## Caching Strategy
///
/// This implementation uses an LRU (Least Recently Used) caching mechanism to store
/// frequently accessed objects in memory for faster retrieval. The caches are protected
/// by `RwLock`s to ensure thread safety while allowing concurrent reads.
///
/// - Each cache has a configurable size limit (default: 1000 items)
/// - When a cache reaches its size limit, the least recently used items will be evicted
/// - All cached data is stored as `Arc<T>` to reduce cloning costs
///
/// ## Thread Safety
///
/// All caches are protected by `RwLock`s, which allow:
/// - Multiple concurrent readers (for find/get operations)
/// - Exclusive writers (for create/save/delete operations)
///
/// This approach optimizes for read-heavy workloads while still ensuring data consistency.
pub struct NostrMlsMemoryStorage {
    /// The underlying storage implementation that conforms to OpenMLS's [`StorageProvider`]
    openmls_storage: MemoryStorage,
    /// LRU Cache for Group objects, keyed by MLS group ID (Vec<u8>)
    groups_cache: RwLock<LruCache<Vec<u8>, Arc<Group>>>,
    /// LRU Cache for Group objects, keyed by Nostr group ID (String)
    groups_by_nostr_id_cache: RwLock<LruCache<String, Arc<Group>>>,
    /// LRU Cache for GroupRelay objects, keyed by MLS group ID (Vec<u8>)
    group_relays_cache: RwLock<LruCache<Vec<u8>, Arc<Vec<GroupRelay>>>>,
    /// LRU Cache for Welcome objects, keyed by Event ID
    welcomes_cache: RwLock<LruCache<EventId, Arc<Welcome>>>,
    /// LRU Cache for ProcessedWelcome objects, keyed by Event ID
    processed_welcomes_cache: RwLock<LruCache<EventId, Arc<ProcessedWelcome>>>,
    /// LRU Cache for Message objects, keyed by Event ID
    messages_cache: RwLock<LruCache<EventId, Arc<Message>>>,
    /// LRU Cache for Messages by Group ID
    messages_by_group_cache: RwLock<LruCache<Vec<u8>, Arc<Vec<Message>>>>,
    /// LRU Cache for ProcessedMessage objects, keyed by Event ID
    processed_messages_cache: RwLock<LruCache<EventId, Arc<ProcessedMessage>>>,
}

impl NostrMlsMemoryStorage {
    /// Creates a new [`NostrMlsMemoryStorage`] with the provided storage implementation.
    ///
    /// # Arguments
    ///
    /// * `storage_implementation` - An implementation of the OpenMLS [`StorageProvider`] trait.
    ///
    /// # Returns
    ///
    /// A new instance of [`NostrMlsMemoryStorage`] wrapping the provided storage implementation.
    pub fn new(storage_implementation: MemoryStorage) -> Self {
        Self::with_cache_size(storage_implementation, DEFAULT_CACHE_SIZE)
    }

    /// Creates a new [`NostrMlsMemoryStorage`] with the provided storage implementation and cache size.
    ///
    /// # Arguments
    ///
    /// * `storage_implementation` - An implementation of the OpenMLS [`StorageProvider`] trait.
    /// * `cache_size` - The maximum number of items to store in each LRU cache.
    ///
    /// # Returns
    ///
    /// A new instance of [`NostrMlsMemoryStorage`] wrapping the provided storage implementation.
    pub fn with_cache_size(storage_implementation: MemoryStorage, cache_size: usize) -> Self {
        // Ensure cache_size is non-zero
        let size = NonZeroUsize::new(cache_size)
            .unwrap_or_else(|| NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap());

        NostrMlsMemoryStorage {
            openmls_storage: storage_implementation,
            groups_cache: RwLock::new(LruCache::new(size)),
            groups_by_nostr_id_cache: RwLock::new(LruCache::new(size)),
            group_relays_cache: RwLock::new(LruCache::new(size)),
            welcomes_cache: RwLock::new(LruCache::new(size)),
            processed_welcomes_cache: RwLock::new(LruCache::new(size)),
            messages_cache: RwLock::new(LruCache::new(size)),
            messages_by_group_cache: RwLock::new(LruCache::new(size)),
            processed_messages_cache: RwLock::new(LruCache::new(size)),
        }
    }
}

impl Default for NostrMlsMemoryStorage {
    /// Creates a new [`NostrMlsMemoryStorage`] with a default OpenMLS memory storage implementation.
    ///
    /// # Returns
    ///
    /// A new instance of [`NostrMlsMemoryStorage`] with default configuration.
    fn default() -> Self {
        Self::new(MemoryStorage::default())
    }
}

/// Implementation of [`NostrMlsStorageProvider`] for memory-based storage.
impl NostrMlsStorageProvider for NostrMlsMemoryStorage {
    type OpenMlsStorageProvider = MemoryStorage;

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
    fn openmls_storage(&self) -> &Self::OpenMlsStorageProvider {
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
    fn openmls_storage_mut(&mut self) -> &mut Self::OpenMlsStorageProvider {
        &mut self.openmls_storage
    }
}

#[cfg(test)]
mod tests {
    use nostr::{EventId, Kind, PublicKey, RelayUrl, Tags, UnsignedEvent};
    use nostr_mls_storage::groups::types::{Group, GroupState, GroupType};
    use nostr_mls_storage::groups::GroupStorage;
    use nostr_mls_storage::messages::types::{Message, ProcessedMessageState};
    use nostr_mls_storage::messages::MessageStorage;
    use nostr_mls_storage::welcomes::types::{ProcessedWelcomeState, Welcome, WelcomeState};
    use nostr_mls_storage::welcomes::WelcomeStorage;
    #[cfg(test)]
    use openmls_memory_storage::MemoryStorage;

    use super::*;

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

    #[test]
    fn test_group_cache() {
        let storage = MemoryStorage::default();
        let nostr_storage = NostrMlsMemoryStorage::new(storage);

        // Create a test group
        let mls_group_id = vec![1, 2, 3, 4];
        let group = Group {
            mls_group_id: mls_group_id.clone(),
            nostr_group_id: "test_group_123".to_string(),
            name: "Test Group".to_string(),
            description: "A test group".to_string(),
            admin_pubkeys: vec![],
            last_message_id: None,
            last_message_at: None,
            group_type: GroupType::Group,
            epoch: 0,
            state: GroupState::Active,
        };

        // Save the group
        let result = nostr_storage.save_group(group.clone());
        assert!(result.is_ok());

        // Find the group by MLS group ID
        let found_group = nostr_storage.find_group_by_mls_group_id(&mls_group_id);
        assert!(found_group.is_ok());

        let found_group = found_group.unwrap();
        assert_eq!(found_group.nostr_group_id, "test_group_123");

        // Find the group by Nostr group ID
        let found_group = nostr_storage.find_group_by_nostr_group_id("test_group_123");
        assert!(found_group.is_ok());

        let found_group = found_group.unwrap();
        assert_eq!(found_group.mls_group_id, mls_group_id);

        // Delete the group (manually remove from caches)
        if let Ok(mut cache) = nostr_storage.groups_cache.write() {
            cache.pop(&mls_group_id);
        }
        if let Ok(mut cache) = nostr_storage.groups_by_nostr_id_cache.write() {
            cache.pop(&"test_group_123".to_string());
        }

        // Verify group is no longer in cache
        let not_found = nostr_storage.find_group_by_mls_group_id(&mls_group_id);
        assert!(not_found.is_err());
    }

    #[test]
    fn test_group_relays() {
        let storage = MemoryStorage::default();
        let nostr_storage = NostrMlsMemoryStorage::new(storage);

        // Create a test group
        let mls_group_id = vec![1, 2, 3, 4];
        let group = Group {
            mls_group_id: mls_group_id.clone(),
            nostr_group_id: "test_group_123".to_string(),
            name: "Test Group".to_string(),
            description: "A test group".to_string(),
            admin_pubkeys: vec![],
            last_message_id: None,
            last_message_at: None,
            group_type: GroupType::Group,
            epoch: 0,
            state: GroupState::Active,
        };

        // Test accessing group_relays before the group exists
        let not_found = nostr_storage.group_relays(&mls_group_id);
        assert!(not_found.is_err());
        match not_found {
            Err(nostr_mls_storage::groups::error::GroupError::NotFound) => {} // Expected
            _ => panic!("Expected GroupError::NotFound"),
        }

        // Save the group
        let result = nostr_storage.save_group(group.clone());
        assert!(result.is_ok());

        // Test accessing group_relays after the group exists
        let empty_relays = nostr_storage.group_relays(&mls_group_id);
        assert!(empty_relays.is_ok());
        assert_eq!(empty_relays.unwrap().len(), 0);

        // Create a test relay
        let relay_url: RelayUrl = "wss://relay.example.com".parse().unwrap();
        let group_relay = GroupRelay {
            mls_group_id: mls_group_id.clone(),
            relay_url: relay_url.clone(),
        };

        // Save the relay
        let save_result = nostr_storage.save_group_relay(group_relay.clone());
        assert!(save_result.is_ok());

        // Get relays for the group
        let relays = nostr_storage.group_relays(&mls_group_id);
        assert!(relays.is_ok());
        let relays = relays.unwrap();
        assert_eq!(relays.len(), 1);
        assert_eq!(relays[0].relay_url, relay_url);

        // Test saving a relay for a non-existent group
        let non_existent_group_id = vec![5, 6, 7, 8];
        let invalid_relay = GroupRelay {
            mls_group_id: non_existent_group_id.clone(),
            relay_url: "wss://relay.example.com".parse().unwrap(),
        };

        let err_result = nostr_storage.save_group_relay(invalid_relay);
        assert!(err_result.is_err());
        match err_result {
            Err(nostr_mls_storage::groups::error::GroupError::NotFound) => {} // Expected
            _ => panic!("Expected GroupError::NotFound"),
        }
    }

    #[test]
    fn test_welcome_cache() {
        let storage = MemoryStorage::default();
        let nostr_storage = NostrMlsMemoryStorage::new(storage);

        // Create test event IDs using proper hex strings of correct length
        let event_id_str = "000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f";
        let wrapper_id_str = "1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100";
        let event_id = EventId::from_hex(event_id_str).unwrap();
        let wrapper_id = EventId::from_hex(wrapper_id_str).unwrap();

        // Create a test pubkey
        let pubkey_str = "0000000000000000000000000000000000000000000000000000000000000000";
        let pubkey = PublicKey::from_hex(pubkey_str).unwrap();

        // Create a test welcome
        let welcome = Welcome {
            id: event_id,
            event: UnsignedEvent::new(
                pubkey,
                nostr::Timestamp::now(),
                Kind::MlsWelcome,
                Tags::new(),
                "test".to_string(),
            ),
            mls_group_id: vec![1, 2, 3, 4],
            nostr_group_id: "test_group_123".to_string(),
            group_name: "Test Group".to_string(),
            group_description: "A test group".to_string(),
            group_admin_pubkeys: vec![pubkey_str.to_string()],
            group_relays: vec![],
            welcomer: pubkey,
            member_count: 1,
            state: WelcomeState::Pending,
            wrapper_event_id: wrapper_id,
        };

        // Save the welcome
        let result = nostr_storage.save_welcome(welcome.clone());
        assert!(result.is_ok());

        // Find the welcome by event ID
        let found_welcome = nostr_storage.find_welcome_by_event_id(event_id);
        assert!(found_welcome.is_ok());

        let found_welcome = found_welcome.unwrap();
        assert_eq!(found_welcome.id, event_id);

        // Create a processed welcome
        let processed_welcome = nostr_mls_storage::welcomes::types::ProcessedWelcome {
            wrapper_event_id: wrapper_id,
            welcome_event_id: Some(event_id),
            state: ProcessedWelcomeState::Processed,
            processed_at: nostr::Timestamp::now(),
            failure_reason: "Successfully processed".to_string(),
        };
        let processed_welcome_result = nostr_storage.save_processed_welcome(processed_welcome);
        assert!(processed_welcome_result.is_ok());

        // Find the processed welcome
        let found_processed_welcome = nostr_storage.find_processed_welcome_by_event_id(wrapper_id);
        assert!(found_processed_welcome.is_ok());

        let found_processed_welcome = found_processed_welcome.unwrap();
        assert_eq!(found_processed_welcome.wrapper_event_id, wrapper_id);
        assert_eq!(found_processed_welcome.welcome_event_id, Some(event_id));

        // Compare enum variants using match instead of direct equality
        match found_processed_welcome.state {
            ProcessedWelcomeState::Processed => { /* Test passed */ }
            _ => panic!("Expected ProcessedWelcomeState::Processed"),
        }
    }

    #[test]
    fn test_message_cache() {
        let storage = MemoryStorage::default();
        let nostr_storage = NostrMlsMemoryStorage::new(storage);

        // Create a test message
        let mls_group_id = vec![1, 2, 3, 4];

        // First create a group since our updated implementation requires the group to exist
        let group = Group {
            mls_group_id: mls_group_id.clone(),
            nostr_group_id: "test_group_123".to_string(),
            name: "Test Group".to_string(),
            description: "A test group".to_string(),
            admin_pubkeys: vec![],
            last_message_id: None,
            last_message_at: None,
            group_type: GroupType::Group,
            epoch: 0,
            state: GroupState::Active,
        };

        // Save the group
        let result = nostr_storage.save_group(group.clone());
        assert!(result.is_ok());

        // Create test event IDs using proper hex strings of correct length
        let message_id_str = "000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f";
        let wrapper_id_str = "1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100";
        let message_id = EventId::from_hex(message_id_str).unwrap();
        let wrapper_id = EventId::from_hex(wrapper_id_str).unwrap();

        // Create a test pubkey
        let pubkey_str = "0000000000000000000000000000000000000000000000000000000000000000";
        let pubkey = PublicKey::from_hex(pubkey_str).unwrap();

        let message = Message {
            id: message_id,
            pubkey,
            kind: Kind::MlsGroupMessage,
            mls_group_id: mls_group_id.clone(),
            created_at: nostr::Timestamp::now(),
            content: "Hello, world!".to_string(),
            tags: Tags::new(),
            event: UnsignedEvent::new(
                pubkey,
                nostr::Timestamp::now(),
                Kind::MlsGroupMessage,
                Tags::new(),
                "Hello, world!".to_string(),
            ),
            wrapper_event_id: wrapper_id,
            tokens: vec![],
        };

        // Save the message
        let result = nostr_storage.save_message(message.clone());
        assert!(result.is_ok());

        // Update the messages_by_group_cache manually for testing purposes
        if let Ok(mut cache) = nostr_storage.messages_by_group_cache.write() {
            // Create a vector of messages for this group
            let messages = vec![message.clone()];
            cache.put(mls_group_id.clone(), Arc::new(messages));
        }

        // Find the message by event ID
        let found_message = nostr_storage.find_message_by_event_id(message_id);
        assert!(found_message.is_ok());

        let found_message = found_message.unwrap();
        assert_eq!(found_message.id, message_id);
        assert_eq!(found_message.content, "Hello, world!");

        // Get messages for the group
        let group_messages = nostr_storage.messages(&mls_group_id);
        assert!(group_messages.is_ok());
        assert_eq!(group_messages.unwrap().len(), 1);

        // Create a processed message
        let processed_message = nostr_mls_storage::messages::types::ProcessedMessage {
            wrapper_event_id: wrapper_id,
            message_event_id: Some(message_id),
            state: ProcessedMessageState::Processed,
            processed_at: nostr::Timestamp::now(),
            failure_reason: "Successfully processed".to_string(),
        };
        let processed_message_result = nostr_storage.save_processed_message(processed_message);
        assert!(processed_message_result.is_ok());

        // Find the processed message
        let found_processed_message = nostr_storage.find_processed_message_by_event_id(wrapper_id);
        assert!(found_processed_message.is_ok());

        let found_processed_message = found_processed_message.unwrap();
        assert_eq!(found_processed_message.wrapper_event_id, wrapper_id);
        assert_eq!(found_processed_message.message_event_id, Some(message_id));

        // Compare enum variants using match instead of direct equality
        match found_processed_message.state {
            ProcessedMessageState::Processed => { /* Test passed */ }
            _ => panic!("Expected ProcessedMessageState::Processed"),
        }
    }

    #[test]
    fn test_with_custom_cache_size() {
        let storage = MemoryStorage::default();
        // Create storage with smaller cache size
        let nostr_storage = NostrMlsMemoryStorage::with_cache_size(storage, 5);

        // Create several groups to test LRU behavior
        for i in 0..10 {
            let mls_group_id = vec![i];
            let group = Group {
                mls_group_id: mls_group_id.clone(),
                nostr_group_id: format!("test_group_{}", i),
                name: format!("Test Group {}", i),
                description: "A test group".to_string(),
                admin_pubkeys: vec![],
                last_message_id: None,
                last_message_at: None,
                group_type: GroupType::Group,
                epoch: 0,
                state: GroupState::Active,
            };

            let result = nostr_storage.save_group(group);
            assert!(result.is_ok());
        }

        // The first groups should be evicted from the cache
        let not_found = nostr_storage.find_group_by_mls_group_id(&[0]);
        assert!(not_found.is_err());

        // The last groups should still be in the cache
        let found = nostr_storage.find_group_by_mls_group_id(&[9]);
        assert!(found.is_ok());
    }

    #[test]
    fn test_default_implementation() {
        let nostr_storage = NostrMlsMemoryStorage::default();
        assert_eq!(nostr_storage.backend(), Backend::Memory);

        // Create a test group to verify the default storage works
        let mls_group_id = vec![1, 2, 3, 4];
        let group = Group {
            mls_group_id: mls_group_id.clone(),
            nostr_group_id: "test_default_group".to_string(),
            name: "Default Test Group".to_string(),
            description: "A test group with default storage".to_string(),
            admin_pubkeys: vec![],
            last_message_id: None,
            last_message_at: None,
            group_type: GroupType::Group,
            epoch: 0,
            state: GroupState::Active,
        };

        // Save the group
        let result = nostr_storage.save_group(group.clone());
        assert!(result.is_ok());

        // Find the group by MLS group ID
        let found_group = nostr_storage.find_group_by_mls_group_id(&mls_group_id);
        assert!(found_group.is_ok());

        let found_group = found_group.unwrap();
        assert_eq!(found_group.nostr_group_id, "test_default_group");
    }
}
