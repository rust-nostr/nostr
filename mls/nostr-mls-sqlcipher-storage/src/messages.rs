//! Implementation of MessageStorage trait for SQLite storage.

use nostr::{EventId, JsonUtil};
use nostr_mls_storage::messages::error::MessageError;
use nostr_mls_storage::messages::types::{Message, ProcessedMessage};
use nostr_mls_storage::messages::MessageStorage;
use rusqlite::{params, OptionalExtension};

use crate::{db, NostrMlsSqliteStorage};

#[inline]
fn into_message_err<T>(e: T) -> MessageError
where
    T: std::error::Error,
{
    MessageError::DatabaseError(e.to_string())
}

impl MessageStorage for NostrMlsSqliteStorage {
    fn save_message(&self, message: Message) -> Result<(), MessageError> {
        let conn_guard = self.db_connection.lock().map_err(into_message_err)?;

        // Serialize complex types to JSON
        let tags_json: String = serde_json::to_string(&message.tags)
            .map_err(|e| MessageError::DatabaseError(format!("Failed to serialize tags: {}", e)))?;

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO messages
             (id, pubkey, kind, mls_group_id, created_at, content, tags, event, wrapper_event_id, state)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    message.id.as_bytes(),
                    message.pubkey.as_bytes(),
                    message.kind.as_u16(),
                    message.mls_group_id.as_slice(),
                    message.created_at.as_u64(),
                    message.content,
                    tags_json,
                    message.event.as_json(),
                    message.wrapper_event_id.as_bytes(),
                    message.state.as_str(),
                ],
            )
            .map_err(into_message_err)?;

        Ok(())
    }

    fn find_message_by_event_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<Message>, MessageError> {
        let conn_guard = self.db_connection.lock().map_err(into_message_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM messages WHERE id = ?")
            .map_err(into_message_err)?;

        stmt.query_row(params![event_id.to_bytes()], db::row_to_message)
            .optional()
            .map_err(into_message_err)
    }

    fn save_processed_message(
        &self,
        processed_message: ProcessedMessage,
    ) -> Result<(), MessageError> {
        let conn_guard = self.db_connection.lock().map_err(into_message_err)?;

        // Convert message_event_id to string if it exists
        let message_event_id = processed_message
            .message_event_id
            .as_ref()
            .map(|id| id.to_bytes());

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO processed_messages
             (wrapper_event_id, message_event_id, processed_at, state, failure_reason)
             VALUES (?, ?, ?, ?, ?)",
                params![
                    &processed_message.wrapper_event_id.to_bytes(),
                    &message_event_id,
                    &processed_message.processed_at.as_u64(),
                    &processed_message.state.to_string(),
                    &processed_message.failure_reason
                ],
            )
            .map_err(into_message_err)?;

        Ok(())
    }

    fn find_processed_message_by_event_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<ProcessedMessage>, MessageError> {
        let conn_guard = self.db_connection.lock().map_err(into_message_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM processed_messages WHERE wrapper_event_id = ?")
            .map_err(into_message_err)?;

        stmt.query_row(params![event_id.to_bytes()], db::row_to_processed_message)
            .optional()
            .map_err(into_message_err)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use nostr::{EventId, Kind, PublicKey, Tags, Timestamp, UnsignedEvent};
    use nostr_mls_storage::groups::types::{Group, GroupState, GroupType};
    use nostr_mls_storage::groups::GroupStorage;
    use nostr_mls_storage::messages::types::{MessageState, ProcessedMessageState};
    use openmls::group::GroupId;

    use super::*;

    #[test]
    fn test_save_and_find_message() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();

        // First create a group (messages require a valid group foreign key)
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 4]);
        let mut nostr_group_id = [0u8; 32];
        nostr_group_id[0..13].copy_from_slice(b"test_group_12");

        let group = Group {
            mls_group_id: mls_group_id.clone(),
            nostr_group_id,
            name: "Test Group".to_string(),
            description: "A test group".to_string(),
            admin_pubkeys: BTreeSet::new(),
            last_message_id: None,
            last_message_at: None,
            group_type: GroupType::Group,
            epoch: 0,
            state: GroupState::Active,
        };

        // Save the group
        let result = storage.save_group(group);
        assert!(result.is_ok());

        // Create a test message
        let event_id =
            EventId::parse("6a2affe9878ebcf50c10cf74c7b25aad62e0db9fb347f6aafeda30e9f578f260")
                .unwrap();
        let pubkey =
            PublicKey::parse("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
                .unwrap();
        let wrapper_event_id =
            EventId::parse("3287abd422284bc3679812c373c52ed4aa0af4f7c57b9c63ec440f6c3ed6c3a2")
                .unwrap();

        let message = Message {
            id: event_id,
            pubkey,
            kind: Kind::from(1u16),
            mls_group_id: mls_group_id.clone(),
            created_at: Timestamp::now(),
            content: "Test message content".to_string(),
            tags: Tags::new(),
            event: UnsignedEvent::new(
                pubkey,
                Timestamp::now(),
                Kind::from(9u16),
                vec![],
                "content".to_string(),
            ),
            wrapper_event_id,
            state: MessageState::Created,
        };

        // Save the message
        let result = storage.save_message(message.clone());
        assert!(result.is_ok());

        // Find by event ID
        let found_message = storage
            .find_message_by_event_id(&event_id)
            .unwrap()
            .unwrap();
        assert_eq!(found_message.id, event_id);
        assert_eq!(found_message.pubkey, pubkey);
        assert_eq!(found_message.content, "Test message content");
    }

    #[test]
    fn test_processed_message() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();

        // Create a test processed message
        let wrapper_event_id =
            EventId::parse("3287abd422284bc3679812c373c52ed4aa0af4f7c57b9c63ec440f6c3ed6c3a2")
                .unwrap();
        let message_event_id =
            EventId::parse("6a2affe9878ebcf50c10cf74c7b25aad62e0db9fb347f6aafeda30e9f578f260")
                .unwrap();

        let processed_message = ProcessedMessage {
            wrapper_event_id,
            message_event_id: Some(message_event_id),
            processed_at: Timestamp::from(1_000_000_000u64),
            state: ProcessedMessageState::Processed,
            failure_reason: None,
        };

        // Save the processed message
        let result = storage.save_processed_message(processed_message.clone());
        assert!(result.is_ok());

        // Find by event ID
        let found_processed_message = storage
            .find_processed_message_by_event_id(&wrapper_event_id)
            .unwrap()
            .unwrap();
        assert_eq!(found_processed_message.wrapper_event_id, wrapper_event_id);
        assert_eq!(
            found_processed_message.message_event_id.unwrap(),
            message_event_id
        );
        assert_eq!(
            found_processed_message.state,
            ProcessedMessageState::Processed
        );
    }
}
