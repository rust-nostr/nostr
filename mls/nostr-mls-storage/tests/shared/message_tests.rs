//! Message storage test functions

use nostr::EventId;
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::MessageStorage;
use openmls::group::GroupId;

use super::{create_test_group, create_test_message, create_test_processed_message};

/// Test message storage functionality
#[allow(dead_code)]
pub fn test_save_and_find_message<S: MessageStorage + GroupStorage>(storage: S) {
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

/// Test processed message functionality
#[allow(dead_code)]
pub fn test_processed_message<S: MessageStorage>(storage: S) {
    let wrapper_event_id = EventId::all_zeros();
    let message_event_id =
        EventId::from_hex("1111111111111111111111111111111111111111111111111111111111111111")
            .unwrap();
    let processed_message = create_test_processed_message(wrapper_event_id, Some(message_event_id));

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
