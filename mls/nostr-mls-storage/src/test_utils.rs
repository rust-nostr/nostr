//! Test utilities for cross-storage consistency testing

/// Random data generation utilities for testing MLS group features
pub mod crypto_utils {
    use aes_gcm::aead::rand_core::RngCore;
    use aes_gcm::aead::OsRng;

    /// Generates random bytes as Vec<u8> of the specified length
    pub fn generate_random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        RngCore::fill_bytes(&mut OsRng, &mut bytes);
        bytes
    }
}

/// Cross-storage consistency testing utilities
pub mod cross_storage {
    use std::collections::BTreeSet;

    use nostr::{EventId, RelayUrl, Timestamp};
    use openmls::group::GroupId;

    use crate::groups::error::GroupError;
    use crate::groups::types::{Group, GroupExporterSecret, GroupState};
    use crate::groups::GroupStorage;
    use crate::messages::types::{Message, MessageState, ProcessedMessage, ProcessedMessageState};
    use crate::messages::MessageStorage;
    use crate::welcomes::types::{ProcessedWelcome, ProcessedWelcomeState, Welcome, WelcomeState};
    use crate::welcomes::WelcomeStorage;

    /// Creates a test group with the given ID for testing purposes
    pub fn create_test_group(mls_group_id: GroupId) -> Group {
        let mut nostr_group_id = [0u8; 32];
        // Use first 4 bytes of mls_group_id to make nostr_group_id somewhat unique
        let mls_bytes = mls_group_id.as_slice();
        if mls_bytes.len() >= 4 {
            nostr_group_id[0..4].copy_from_slice(&mls_bytes[0..4]);
        }

        Group {
            mls_group_id,
            nostr_group_id,
            name: "Test Group".to_string(),
            description: "A test group".to_string(),
            admin_pubkeys: BTreeSet::new(),
            last_message_id: None,
            last_message_at: None,
            epoch: 0,
            state: GroupState::Active,
            image_hash: None,
            image_key: None,
            image_nonce: None,
        }
    }

    /// Test scenarios for replace_group_relays functionality
    pub fn test_replace_group_relays_comprehensive<S>(storage: S)
    where
        S: GroupStorage,
    {
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 4]);
        let group = create_test_group(mls_group_id.clone());

        // Save the test group
        storage.save_group(group).unwrap();

        // Test 1: Replace with initial relay set
        let relay1 = RelayUrl::parse("wss://relay1.example.com").unwrap();
        let relay2 = RelayUrl::parse("wss://relay2.example.com").unwrap();
        let initial_relays = BTreeSet::from([relay1.clone(), relay2.clone()]);

        storage
            .replace_group_relays(&mls_group_id, initial_relays.clone())
            .unwrap();
        let stored_relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(stored_relays.len(), 2);
        assert!(stored_relays.iter().any(|r| r.relay_url == relay1));
        assert!(stored_relays.iter().any(|r| r.relay_url == relay2));

        // Test 2: Replace with different relay set (should remove old ones)
        let relay3 = RelayUrl::parse("wss://relay3.example.com").unwrap();
        let relay4 = RelayUrl::parse("wss://relay4.example.com").unwrap();
        let new_relays = BTreeSet::from([relay3.clone(), relay4.clone()]);

        storage
            .replace_group_relays(&mls_group_id, new_relays.clone())
            .unwrap();
        let stored_relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(stored_relays.len(), 2);
        assert!(stored_relays.iter().any(|r| r.relay_url == relay3));
        assert!(stored_relays.iter().any(|r| r.relay_url == relay4));
        // Old relays should be gone
        assert!(!stored_relays.iter().any(|r| r.relay_url == relay1));
        assert!(!stored_relays.iter().any(|r| r.relay_url == relay2));

        // Test 3: Replace with empty set
        storage
            .replace_group_relays(&mls_group_id, BTreeSet::new())
            .unwrap();
        let stored_relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(stored_relays.len(), 0);

        // Test 4: Replace with single relay after empty
        let single_relay = BTreeSet::from([relay1.clone()]);
        storage
            .replace_group_relays(&mls_group_id, single_relay)
            .unwrap();
        let stored_relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(stored_relays.len(), 1);
        assert_eq!(stored_relays.first().unwrap().relay_url, relay1);

        // Test 5: Replace with large set
        let large_set: BTreeSet<RelayUrl> = (1..=10)
            .map(|i| RelayUrl::parse(&format!("wss://relay{}.example.com", i)).unwrap())
            .collect();
        storage
            .replace_group_relays(&mls_group_id, large_set.clone())
            .unwrap();
        let stored_relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(stored_relays.len(), 10);
        for expected_relay in &large_set {
            assert!(stored_relays.iter().any(|r| r.relay_url == *expected_relay));
        }
    }

    /// Test error cases for replace_group_relays
    pub fn test_replace_group_relays_error_cases<S>(storage: S)
    where
        S: GroupStorage,
    {
        // Test: Replace relays for non-existent group
        let non_existent_group_id = GroupId::from_slice(&[99, 99, 99, 99]);
        let relay = RelayUrl::parse("wss://relay.example.com").unwrap();
        let relays = BTreeSet::from([relay]);

        let result = storage.replace_group_relays(&non_existent_group_id, relays);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GroupError::InvalidParameters(_)
        ));
    }

    /// Test duplicate handling for replace_group_relays
    pub fn test_replace_group_relays_duplicate_handling<S>(storage: S)
    where
        S: GroupStorage,
    {
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 5]);
        let group = create_test_group(mls_group_id.clone());

        storage.save_group(group).unwrap();

        // Test: BTreeSet naturally handles duplicates, but test behavior is consistent
        let relay = RelayUrl::parse("wss://relay.example.com").unwrap();
        let relays = BTreeSet::from([relay.clone()]);

        // Add same relay multiple times - should be idempotent
        storage
            .replace_group_relays(&mls_group_id, relays.clone())
            .unwrap();
        storage
            .replace_group_relays(&mls_group_id, relays.clone())
            .unwrap();

        let stored_relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(stored_relays.len(), 1);
        assert_eq!(stored_relays.first().unwrap().relay_url, relay);
    }

    /// Test basic group save and find functionality
    pub fn test_save_and_find_group<S>(storage: S)
    where
        S: GroupStorage,
    {
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 6]);
        let group = create_test_group(mls_group_id.clone());

        // Test save
        storage.save_group(group.clone()).unwrap();

        // Test find by MLS group ID
        let found_group = storage.find_group_by_mls_group_id(&mls_group_id).unwrap();
        assert!(found_group.is_some());
        let found_group = found_group.unwrap();
        assert_eq!(found_group.mls_group_id, group.mls_group_id);
        assert_eq!(found_group.nostr_group_id, group.nostr_group_id);
        assert_eq!(found_group.name, group.name);
        assert_eq!(found_group.description, group.description);

        // Test find by Nostr group ID
        let found_group = storage
            .find_group_by_nostr_group_id(&group.nostr_group_id)
            .unwrap();
        assert!(found_group.is_some());
        let found_group = found_group.unwrap();
        assert_eq!(found_group.mls_group_id, group.mls_group_id);

        // Test find non-existent group
        let non_existent_id = GroupId::from_slice(&[99, 99, 99, 99]);
        let result = storage
            .find_group_by_mls_group_id(&non_existent_id)
            .unwrap();
        assert!(result.is_none());
    }

    /// Test edge cases and error conditions for group operations
    pub fn test_group_edge_cases<S>(storage: S)
    where
        S: GroupStorage,
    {
        // Test saving group with empty name
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 14]);
        let mut group = create_test_group(mls_group_id.clone());
        group.name = String::new();

        // Should still work (empty names are valid)
        storage.save_group(group.clone()).unwrap();
        let found = storage
            .find_group_by_mls_group_id(&mls_group_id)
            .unwrap()
            .unwrap();
        assert_eq!(found.name, "");

        // Test saving group with very long name
        let long_name = "a".repeat(1000);
        group.name = long_name.clone();
        storage.save_group(group).unwrap();
        let found = storage
            .find_group_by_mls_group_id(&mls_group_id)
            .unwrap()
            .unwrap();
        assert_eq!(found.name, long_name);

        // Test duplicate group save (should update existing)
        let mut updated_group = create_test_group(mls_group_id.clone());
        updated_group.description = "Updated description".to_string();
        storage.save_group(updated_group).unwrap();
        let found = storage
            .find_group_by_mls_group_id(&mls_group_id)
            .unwrap()
            .unwrap();
        assert_eq!(found.description, "Updated description");
    }

    /// Test concurrent relay operations and edge cases
    pub fn test_replace_relays_edge_cases<S>(storage: S)
    where
        S: GroupStorage,
    {
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 15]);
        let group = create_test_group(mls_group_id.clone());
        storage.save_group(group).unwrap();

        // Test with very large relay sets
        let large_relay_set: BTreeSet<RelayUrl> = (1..=100)
            .map(|i| RelayUrl::parse(&format!("wss://relay{}.example.com", i)).unwrap())
            .collect();

        storage
            .replace_group_relays(&mls_group_id, large_relay_set.clone())
            .unwrap();
        let stored = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(stored.len(), 100);

        // Test multiple rapid replacements
        for i in 0..10 {
            let relay_set =
                BTreeSet::from([RelayUrl::parse(&format!("wss://test{}.com", i)).unwrap()]);
            storage
                .replace_group_relays(&mls_group_id, relay_set)
                .unwrap();
        }
        let final_relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(final_relays.len(), 1);
        assert_eq!(
            final_relays.first().unwrap().relay_url.to_string(),
            "wss://test9.com"
        );
    }

    /// Test group exporter secret functionality
    pub fn test_group_exporter_secret<S>(storage: S)
    where
        S: GroupStorage,
    {
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 7]);
        let group = create_test_group(mls_group_id.clone());
        storage.save_group(group).unwrap();

        let epoch = 42u64;
        let secret = [0x42u8; 32];
        let exporter_secret = GroupExporterSecret {
            mls_group_id: mls_group_id.clone(),
            epoch,
            secret,
        };

        // Test save
        storage
            .save_group_exporter_secret(exporter_secret.clone())
            .unwrap();

        // Test get
        let retrieved = storage
            .get_group_exporter_secret(&mls_group_id, epoch)
            .unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.mls_group_id, mls_group_id);
        assert_eq!(retrieved.epoch, epoch);
        assert_eq!(retrieved.secret, secret);

        // Test get non-existent
        let result = storage
            .get_group_exporter_secret(&mls_group_id, 999)
            .unwrap();
        assert!(result.is_none());
    }

    /// Test all groups functionality
    pub fn test_all_groups<S>(storage: S)
    where
        S: GroupStorage,
    {
        // Initially should be empty
        let groups = storage.all_groups().unwrap();
        assert_eq!(groups.len(), 0);

        // Add some groups
        let group1 = create_test_group(GroupId::from_slice(&[1, 2, 3, 8]));
        let group2 = create_test_group(GroupId::from_slice(&[1, 2, 3, 9]));
        let group3 = create_test_group(GroupId::from_slice(&[1, 2, 3, 10]));

        storage.save_group(group1.clone()).unwrap();
        storage.save_group(group2.clone()).unwrap();
        storage.save_group(group3.clone()).unwrap();

        // Test all groups
        let groups = storage.all_groups().unwrap();
        assert_eq!(groups.len(), 3);

        let group_ids: BTreeSet<_> = groups.iter().map(|g| g.mls_group_id.clone()).collect();
        assert!(group_ids.contains(&group1.mls_group_id));
        assert!(group_ids.contains(&group2.mls_group_id));
        assert!(group_ids.contains(&group3.mls_group_id));
    }

    /// Test basic group relay functionality (not the comprehensive replace tests)
    pub fn test_basic_group_relays<S>(storage: S)
    where
        S: GroupStorage,
    {
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 11]);
        let group = create_test_group(mls_group_id.clone());
        storage.save_group(group).unwrap();

        // Initially should be empty
        let relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(relays.len(), 0);

        // Add a relay using replace
        let relay_url = RelayUrl::parse("wss://relay.example.com").unwrap();
        let relays_set = BTreeSet::from([relay_url.clone()]);
        storage
            .replace_group_relays(&mls_group_id, relays_set)
            .unwrap();

        // Verify it's there
        let relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(relays.len(), 1);
        assert_eq!(relays.first().unwrap().relay_url, relay_url);
    }

    /// Creates a test message for testing purposes
    pub fn create_test_message(mls_group_id: GroupId, event_id: EventId) -> Message {
        use nostr::{Kind, PublicKey, Tags, Timestamp, UnsignedEvent};

        let pubkey =
            PublicKey::parse("npub1a6awmmklxfmspwdv52qq58sk5c07kghwc4v2eaudjx2ju079cdqs2452ys")
                .unwrap();
        let created_at = Timestamp::now();
        let content = "Test message content".to_string();
        let tags = Tags::new();

        let event = UnsignedEvent {
            id: Some(event_id),
            pubkey,
            created_at,
            kind: Kind::Custom(445),
            tags: tags.clone(),
            content: content.clone(),
        };

        Message {
            id: event_id,
            pubkey,
            kind: Kind::Custom(445),
            mls_group_id,
            created_at,
            content,
            tags,
            event,
            wrapper_event_id: event_id,
            state: MessageState::Processed,
        }
    }

    /// Creates a test processed message for testing purposes
    pub fn create_test_processed_message(
        wrapper_event_id: EventId,
        message_event_id: Option<EventId>,
    ) -> ProcessedMessage {
        ProcessedMessage {
            wrapper_event_id,
            message_event_id,
            processed_at: Timestamp::now(),
            state: ProcessedMessageState::Processed,
            failure_reason: None,
        }
    }

    /// Test message storage functionality
    pub fn test_save_and_find_message<S>(storage: S)
    where
        S: MessageStorage + GroupStorage,
    {
        use nostr::EventId;

        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 12]);

        // First create the group (required for foreign key constraints)
        let group = create_test_group(mls_group_id.clone());
        storage.save_group(group).unwrap();

        let event_id = EventId::all_zeros();
        let message = create_test_message(mls_group_id.clone(), event_id);

        // Test save
        storage.save_message(message.clone()).unwrap();

        // Test find
        let found_message = storage.find_message_by_event_id(&event_id).unwrap();
        assert!(found_message.is_some());
        let found_message = found_message.unwrap();
        assert_eq!(found_message.id, message.id);
        assert_eq!(found_message.content, message.content);
        assert_eq!(found_message.mls_group_id, message.mls_group_id);

        // Test find non-existent
        let non_existent_id =
            EventId::from_hex("abababababababababababababababababababababababababababababababab")
                .unwrap();
        let result = storage.find_message_by_event_id(&non_existent_id).unwrap();
        assert!(result.is_none());
    }

    /// Test message storage functionality with group queries
    pub fn test_messages_for_group<S>(storage: S)
    where
        S: GroupStorage,
    {
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 12]);
        let group = create_test_group(mls_group_id.clone());
        storage.save_group(group).unwrap();

        // Test messages for group (initially empty)
        let messages = storage.messages(&mls_group_id).unwrap();
        assert_eq!(messages.len(), 0);
    }

    /// Test processed message functionality
    pub fn test_processed_message<S>(storage: S)
    where
        S: MessageStorage,
    {
        use nostr::EventId;

        let wrapper_event_id = EventId::all_zeros();
        let message_event_id =
            EventId::from_hex("1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap();
        let processed_message =
            create_test_processed_message(wrapper_event_id, Some(message_event_id));

        // Test save
        storage
            .save_processed_message(processed_message.clone())
            .unwrap();

        // Test find by wrapper event id
        let found = storage
            .find_processed_message_by_event_id(&wrapper_event_id)
            .unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.wrapper_event_id, wrapper_event_id);
        assert_eq!(found.message_event_id, Some(message_event_id));

        // Note: The MessageStorage trait doesn't have find_processed_message_by_message_event_id
        // We only test find_processed_message_by_event_id which finds by wrapper event id

        // Test find non-existent
        let non_existent_id =
            EventId::from_hex("abababababababababababababababababababababababababababababababab")
                .unwrap();
        let result = storage
            .find_processed_message_by_event_id(&non_existent_id)
            .unwrap();
        assert!(result.is_none());
    }

    /// Creates a test welcome for testing purposes
    pub fn create_test_welcome(mls_group_id: GroupId, event_id: EventId) -> Welcome {
        use nostr::{Kind, PublicKey, RelayUrl, Tags, Timestamp, UnsignedEvent};

        let pubkey =
            PublicKey::parse("npub1a6awmmklxfmspwdv52qq58sk5c07kghwc4v2eaudjx2ju079cdqs2452ys")
                .unwrap();
        let created_at = Timestamp::now();
        let content = "Test welcome content".to_string();
        let tags = Tags::new();

        let event = UnsignedEvent {
            id: Some(event_id),
            pubkey,
            created_at,
            kind: Kind::Custom(444),
            tags,
            content,
        };

        Welcome {
            id: event_id,
            event,
            mls_group_id,
            nostr_group_id: [0u8; 32],
            group_name: "Test Group".to_string(),
            group_description: "A test group".to_string(),
            group_image_hash: None,
            group_image_key: None,
            group_image_nonce: None,
            group_admin_pubkeys: BTreeSet::from([pubkey]),
            group_relays: BTreeSet::from([RelayUrl::parse("wss://relay.example.com").unwrap()]),
            welcomer: pubkey,
            member_count: 1,
            state: WelcomeState::Pending,
            wrapper_event_id: event_id,
        }
    }

    /// Creates a test processed welcome for testing purposes
    pub fn create_test_processed_welcome(
        wrapper_event_id: EventId,
        welcome_event_id: Option<EventId>,
    ) -> ProcessedWelcome {
        ProcessedWelcome {
            wrapper_event_id,
            welcome_event_id,
            processed_at: Timestamp::now(),
            state: ProcessedWelcomeState::Processed,
            failure_reason: None,
        }
    }

    /// Test welcome storage functionality
    pub fn test_save_and_find_welcome<S>(storage: S)
    where
        S: WelcomeStorage + GroupStorage,
    {
        use nostr::EventId;

        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 13]);

        // First create the group (required for foreign key constraints)
        let group = create_test_group(mls_group_id.clone());
        storage.save_group(group).unwrap();

        let event_id = EventId::all_zeros();
        let welcome = create_test_welcome(mls_group_id.clone(), event_id);

        // Test save
        storage.save_welcome(welcome.clone()).unwrap();

        // Test find
        let found_welcome = storage.find_welcome_by_event_id(&event_id).unwrap();
        assert!(found_welcome.is_some());
        let found_welcome = found_welcome.unwrap();
        assert_eq!(found_welcome.id, welcome.id);
        assert_eq!(found_welcome.group_name, welcome.group_name);
        assert_eq!(found_welcome.mls_group_id, welcome.mls_group_id);

        // Test find non-existent
        let non_existent_id =
            EventId::from_hex("abababababababababababababababababababababababababababababababab")
                .unwrap();
        let result = storage.find_welcome_by_event_id(&non_existent_id).unwrap();
        assert!(result.is_none());

        // Test pending welcomes
        let pending = storage.pending_welcomes().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, event_id);
    }

    /// Test processed welcome functionality
    pub fn test_processed_welcome<S>(storage: S)
    where
        S: WelcomeStorage,
    {
        use nostr::EventId;

        let wrapper_event_id = EventId::all_zeros();
        let welcome_event_id =
            EventId::from_hex("1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap();
        let processed_welcome =
            create_test_processed_welcome(wrapper_event_id, Some(welcome_event_id));

        // Test save
        storage
            .save_processed_welcome(processed_welcome.clone())
            .unwrap();

        // Test find by wrapper event id
        let found = storage
            .find_processed_welcome_by_event_id(&wrapper_event_id)
            .unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.wrapper_event_id, wrapper_event_id);
        assert_eq!(found.welcome_event_id, Some(welcome_event_id));

        // Test find non-existent
        let non_existent_id =
            EventId::from_hex("abababababababababababababababababababababababababababababababab")
                .unwrap();
        let result = storage
            .find_processed_welcome_by_event_id(&non_existent_id)
            .unwrap();
        assert!(result.is_none());
    }
}

/// Macro to generate cross-storage tests for both SQLite and Memory implementations
#[macro_export]
macro_rules! test_both_storages {
    ($test_name:ident, $test_fn:path) => {
        mod $test_name {
            use super::*;

            #[test]
            fn sqlite() {
                let storage = $crate::NostrMlsSqliteStorage::new_in_memory().unwrap();
                $test_fn(storage);
            }

            #[test]
            fn memory() {
                let storage = $crate::NostrMlsMemoryStorage::new(
                    ::openmls_memory_storage::MemoryStorage::default(),
                );
                $test_fn(storage);
            }
        }
    };
}
