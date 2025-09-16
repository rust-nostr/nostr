//! Test utilities for the nostr-mls crate
//!
//! This module provides shared test utilities used across multiple test modules
//! to avoid code duplication and ensure consistency in test setup.

use nostr::{Event, EventBuilder, Keys, Kind, PublicKey, RelayUrl};
use nostr_mls_storage::NostrMlsStorageProvider;
use openmls::group::GroupId;

use crate::NostrMls;
use crate::groups::NostrGroupConfigData;

/// Creates test group members with standard configuration
///
/// Returns a tuple of (creator_keys, member_keys_vec, admin_pubkeys_vec)
/// where the creator and first member are admins.
pub fn create_test_group_members() -> (Keys, Vec<Keys>, Vec<PublicKey>) {
    let creator = Keys::generate();
    let member1 = Keys::generate();
    let member2 = Keys::generate();

    let creator_pk = creator.public_key();
    let members = vec![member1, member2];
    let admins = vec![creator_pk, members[0].public_key()];

    (creator, members, admins)
}

/// Creates a key package event for a member
///
/// This helper creates a properly signed key package event that can be used
/// in group creation or member addition operations.
pub fn create_key_package_event<Storage>(nostr_mls: &NostrMls<Storage>, member_keys: &Keys) -> Event
where
    Storage: NostrMlsStorageProvider,
{
    let relays = vec![RelayUrl::parse("wss://test.relay").unwrap()];
    let (key_package_hex, tags) = nostr_mls
        .create_key_package_for_event(&member_keys.public_key(), relays)
        .expect("Failed to create key package");

    EventBuilder::new(Kind::MlsKeyPackage, key_package_hex)
        .tags(tags.to_vec())
        .sign_with_keys(member_keys)
        .expect("Failed to sign event")
}

/// Creates a key package event with specified public key and signing keys
///
/// This variant allows creating a key package for one public key but signing
/// it with different keys, useful for testing edge cases.
pub fn create_key_package_event_with_key<Storage>(
    nostr_mls: &NostrMls<Storage>,
    pubkey: &PublicKey,
    signing_keys: &Keys,
) -> Event
where
    Storage: NostrMlsStorageProvider,
{
    let relays = vec![RelayUrl::parse("wss://test.relay").unwrap()];
    let (key_package_hex, tags) = nostr_mls
        .create_key_package_for_event(pubkey, relays)
        .expect("Failed to create key package");

    EventBuilder::new(Kind::MlsKeyPackage, key_package_hex)
        .tags(tags.to_vec())
        .sign_with_keys(signing_keys)
        .expect("Failed to sign event")
}

/// Creates standard test group configuration data
///
/// Returns a NostrGroupConfigData with random test values for creating test groups.
pub fn create_nostr_group_config_data(admins: Vec<PublicKey>) -> NostrGroupConfigData {
    let relays = vec![RelayUrl::parse("wss://test.relay").unwrap()];
    let image_hash = nostr_mls_storage::test_utils::crypto_utils::generate_random_bytes(32)
        .try_into()
        .unwrap();
    let image_key = nostr_mls_storage::test_utils::crypto_utils::generate_random_bytes(32)
        .try_into()
        .unwrap();
    let image_nonce = nostr_mls_storage::test_utils::crypto_utils::generate_random_bytes(12)
        .try_into()
        .unwrap();
    let name = "Test Group".to_owned();
    let description = "A test group for basic testing".to_owned();
    NostrGroupConfigData::new(
        name,
        description,
        Some(image_hash),
        Some(image_key),
        Some(image_nonce),
        relays,
        admins,
    )
}

/// Creates a complete test group and returns the group ID
///
/// This helper function creates a group with the specified creator, members, and admins,
/// then merges the pending commit to complete the group setup.
pub fn create_test_group<Storage>(
    nostr_mls: &NostrMls<Storage>,
    creator: &Keys,
    members: &[Keys],
    admins: &[PublicKey],
) -> GroupId
where
    Storage: NostrMlsStorageProvider,
{
    let creator_pk = creator.public_key();

    // Create key package events for initial members
    let mut initial_key_package_events = Vec::new();
    for member_keys in members {
        let key_package_event = create_key_package_event(nostr_mls, member_keys);
        initial_key_package_events.push(key_package_event);
    }

    // Create the group
    let create_result = nostr_mls
        .create_group(
            &creator_pk,
            initial_key_package_events,
            create_nostr_group_config_data(admins.to_vec()),
        )
        .expect("Failed to create group");

    let group_id = create_result.group.mls_group_id.clone();

    // Merge the pending commit to apply the member additions
    nostr_mls
        .merge_pending_commit(&group_id)
        .expect("Failed to merge pending commit");

    group_id
}

/// Creates a test message rumor (unsigned event)
///
/// This helper creates an unsigned event that can be used for testing
/// message creation and processing.
pub fn create_test_rumor(sender_keys: &Keys, content: &str) -> nostr::UnsignedEvent {
    EventBuilder::new(Kind::TextNote, content).build(sender_keys.public_key())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_function_randomness() {
        let (_, _, admins) = create_test_group_members();

        // Test that the helper works and generates random data
        let config1 = create_nostr_group_config_data(admins.clone());
        let config2 = create_nostr_group_config_data(admins);

        // Both should have the same basic properties
        assert_eq!(config1.name, "Test Group");
        assert_eq!(config2.name, "Test Group");
        assert_eq!(config1.description, "A test group for basic testing");
        assert_eq!(config2.description, "A test group for basic testing");

        // Random helper should return different values (very unlikely to be the same)
        assert_ne!(config1.image_hash, config2.image_hash);
        assert_ne!(config1.image_key, config2.image_key);
        assert_ne!(config1.image_nonce, config2.image_nonce);

        // All should be Some (not None)
        assert!(config1.image_hash.is_some());
        assert!(config1.image_key.is_some());
        assert!(config1.image_nonce.is_some());
        assert!(config2.image_hash.is_some());
        assert!(config2.image_key.is_some());
        assert!(config2.image_nonce.is_some());
    }
}
