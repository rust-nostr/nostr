//! Cross-storage consistency tests
//!
//! These tests ensure that SQLite and Memory storage implementations behave identically
//! for the same operations by running them side-by-side and comparing results.

use std::collections::BTreeSet;

use nostr::RelayUrl;
use nostr_mls_memory_storage::NostrMlsMemoryStorage;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use nostr_mls_storage::groups::GroupStorage;
use openmls::group::GroupId;
use openmls_memory_storage::MemoryStorage;

mod shared;

/// Test harness for differential testing between storage implementations
pub struct StorageTestHarness {
    pub sqlite: NostrMlsSqliteStorage,
    pub memory: NostrMlsMemoryStorage,
}

impl Default for StorageTestHarness {
    fn default() -> Self {
        Self {
            sqlite: NostrMlsSqliteStorage::new(":memory:")
                .expect("Failed to create SQLite storage"),
            memory: NostrMlsMemoryStorage::new(MemoryStorage::default()),
        }
    }
}

impl StorageTestHarness {
    /// Create a new test harness with fresh storage instances
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute the same operation on both storages and assert they behave identically
    pub fn assert_consistent_save_group(&self, group: nostr_mls_storage::groups::types::Group) {
        let sqlite_result = self.sqlite.save_group(group.clone());
        let memory_result = self.memory.save_group(group.clone());

        assert_eq!(
            sqlite_result.is_ok(),
            memory_result.is_ok(),
            "save_group results differ"
        );

        if sqlite_result.is_ok() {
            let sqlite_group = self
                .sqlite
                .find_group_by_mls_group_id(&group.mls_group_id)
                .unwrap()
                .unwrap();
            let memory_group = self
                .memory
                .find_group_by_mls_group_id(&group.mls_group_id)
                .unwrap()
                .unwrap();
            assert_eq!(
                sqlite_group, memory_group,
                "Stored groups differ after successful save"
            );
        } else {
            assert_eq!(
                format!("{:?}", sqlite_result.unwrap_err()),
                format!("{:?}", memory_result.unwrap_err()),
                "Error messages differ"
            );
        }
    }

    pub fn assert_consistent_replace_relays(&self, group_id: &GroupId, relays: BTreeSet<RelayUrl>) {
        let sqlite_result = self.sqlite.replace_group_relays(group_id, relays.clone());
        let memory_result = self.memory.replace_group_relays(group_id, relays);

        assert_eq!(
            sqlite_result.is_ok(),
            memory_result.is_ok(),
            "replace_group_relays results differ"
        );

        if sqlite_result.is_ok() {
            let sqlite_relays = self.sqlite.group_relays(group_id).unwrap();
            let memory_relays = self.memory.group_relays(group_id).unwrap();
            assert_eq!(
                sqlite_relays, memory_relays,
                "Stored relays differ after successful replacement"
            );
        } else {
            assert_eq!(
                format!("{:?}", sqlite_result.unwrap_err()),
                format!("{:?}", memory_result.unwrap_err()),
                "Error messages differ"
            );
        }
    }

    pub fn assert_consistent_group_relays(&self, group_id: &GroupId) {
        let sqlite_relays = self.sqlite.group_relays(group_id).unwrap();
        let memory_relays = self.memory.group_relays(group_id).unwrap();
        assert_eq!(sqlite_relays, memory_relays, "Group relays differ");
    }
}

/// Helper to create a dummy group for testing
fn create_test_group_for_cross_storage(
    mls_group_id: &GroupId,
    nostr_group_id: [u8; 32],
) -> nostr_mls_storage::groups::types::Group {
    use nostr_mls_storage::groups::types::{Group, GroupState};

    Group {
        mls_group_id: mls_group_id.clone(),
        nostr_group_id,
        name: "Test Group".to_string(),
        description: "A test group".to_string(),
        admin_pubkeys: BTreeSet::new(),
        last_message_id: None,
        last_message_at: None,
        epoch: 0,
        state: GroupState::Active,
        image_url: None,
        image_key: None,
        image_nonce: None,
    }
}

#[test]
fn test_replace_relays_basic_consistency() {
    let harness = StorageTestHarness::new();
    let mls_group_id = GroupId::from_slice(&[1, 2, 3, 4]);
    let mut nostr_group_id = [0u8; 32];
    nostr_group_id[0..10].copy_from_slice(b"basic_cons");

    let group = create_test_group_for_cross_storage(&mls_group_id, nostr_group_id);
    harness.assert_consistent_save_group(group);

    let relay1 = RelayUrl::parse("wss://relay1.com").unwrap();
    let relay2 = RelayUrl::parse("wss://relay2.com").unwrap();
    let relays = BTreeSet::from([relay1, relay2]);

    harness.assert_consistent_replace_relays(&mls_group_id, relays);
}

#[test]
fn test_replace_relays_error_consistency() {
    let harness = StorageTestHarness::new();
    let non_existent_group_id = GroupId::from_slice(&[99, 99, 99, 99]);
    let relay = RelayUrl::parse("wss://error.com").unwrap();
    let relays = BTreeSet::from([relay]);

    harness.assert_consistent_replace_relays(&non_existent_group_id, relays);
}

#[test]
fn test_replace_relays_sequence_consistency() {
    let harness = StorageTestHarness::new();
    let mls_group_id = GroupId::from_slice(&[1, 2, 3, 6]);
    let mut nostr_group_id = [0u8; 32];
    nostr_group_id[0..8].copy_from_slice(b"seq_cons");

    let group = create_test_group_for_cross_storage(&mls_group_id, nostr_group_id);
    harness.assert_consistent_save_group(group);

    let r1 = RelayUrl::parse("wss://r1.com").unwrap();
    let r2 = RelayUrl::parse("wss://r2.com").unwrap();
    let r3 = RelayUrl::parse("wss://r3.com").unwrap();

    // Execute sequence of operations
    harness.assert_consistent_replace_relays(&mls_group_id, BTreeSet::from([r1.clone()]));
    harness
        .assert_consistent_replace_relays(&mls_group_id, BTreeSet::from([r1.clone(), r2.clone()]));
    harness.assert_consistent_replace_relays(&mls_group_id, BTreeSet::from([r3.clone()]));
    harness.assert_consistent_replace_relays(&mls_group_id, BTreeSet::new());
    harness.assert_consistent_replace_relays(&mls_group_id, BTreeSet::from([r1, r2, r3]));
}
