//! Welcome storage test functions

use nostr::EventId;
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::welcomes::WelcomeStorage;
use openmls::group::GroupId;

use super::{create_test_group, create_test_processed_welcome, create_test_welcome};

/// Test welcome storage functionality
#[allow(dead_code)]
pub fn test_save_and_find_welcome<S: WelcomeStorage + GroupStorage>(storage: S) {
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
#[allow(dead_code)]
pub fn test_processed_welcome<S: WelcomeStorage>(storage: S) {
    let wrapper_event_id = EventId::all_zeros();
    let welcome_event_id =
        EventId::from_hex("1111111111111111111111111111111111111111111111111111111111111111")
            .unwrap();
    let processed_welcome = create_test_processed_welcome(wrapper_event_id, Some(welcome_event_id));

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
