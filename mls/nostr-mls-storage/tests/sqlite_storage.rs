//! SQLite storage implementation tests using shared test functions

use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;

#[path = "shared/mod.rs"]
mod shared;

/// Macro to generate tests that run against SQLite storage using shared test functions
macro_rules! test_sqlite_storage {
    ($test_name:ident, $test_fn:path) => {
        #[test]
        fn $test_name() {
            let storage = NostrMlsSqliteStorage::new(":memory:").unwrap();
            $test_fn(storage);
        }
    };
}

// Group functionality tests
test_sqlite_storage!(
    test_save_and_find_group_sqlite,
    shared::group_tests::test_save_and_find_group
);

test_sqlite_storage!(test_all_groups_sqlite, shared::group_tests::test_all_groups);

test_sqlite_storage!(
    test_group_exporter_secret_sqlite,
    shared::group_tests::test_group_exporter_secret
);

test_sqlite_storage!(
    test_basic_group_relays_sqlite,
    shared::group_tests::test_basic_group_relays
);

test_sqlite_storage!(
    test_group_edge_cases_sqlite,
    shared::group_tests::test_group_edge_cases
);

test_sqlite_storage!(
    test_replace_relays_edge_cases_sqlite,
    shared::group_tests::test_replace_relays_edge_cases
);

// Comprehensive relay tests
test_sqlite_storage!(
    test_replace_group_relays_comprehensive_sqlite,
    shared::group_tests::test_replace_group_relays_comprehensive
);

test_sqlite_storage!(
    test_replace_group_relays_error_cases_sqlite,
    shared::group_tests::test_replace_group_relays_error_cases
);

test_sqlite_storage!(
    test_replace_group_relays_duplicate_handling_sqlite,
    shared::group_tests::test_replace_group_relays_duplicate_handling
);

// Message functionality tests
test_sqlite_storage!(
    test_save_and_find_message_sqlite,
    shared::message_tests::test_save_and_find_message
);

test_sqlite_storage!(
    test_processed_message_sqlite,
    shared::message_tests::test_processed_message
);

test_sqlite_storage!(
    test_messages_for_group_sqlite,
    shared::group_tests::test_messages_for_group
);

// Welcome functionality tests
test_sqlite_storage!(
    test_save_and_find_welcome_sqlite,
    shared::welcome_tests::test_save_and_find_welcome
);

test_sqlite_storage!(
    test_processed_welcome_sqlite,
    shared::welcome_tests::test_processed_welcome
);
