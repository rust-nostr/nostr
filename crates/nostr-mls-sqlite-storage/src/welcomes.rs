//! Implementation of WelcomeStorage trait for SQLite storage.

use nostr::EventId;
use nostr_mls_storage::welcomes::error::WelcomeError;
use nostr_mls_storage::welcomes::types::{ProcessedWelcome, Welcome};
use nostr_mls_storage::welcomes::WelcomeStorage;
use rusqlite::params;

use crate::{db, NostrMlsSqliteStorage};

impl WelcomeStorage for NostrMlsSqliteStorage {
    fn save_welcome(&self, welcome: Welcome) -> Result<(), WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            WelcomeError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        // Serialize complex types to JSON
        let event_json = serde_json::to_string(&welcome.event).map_err(|e| {
            WelcomeError::DatabaseError(format!("Failed to serialize event: {}", e))
        })?;

        let group_admin_pubkeys_json = serde_json::to_string(&welcome.group_admin_pubkeys)
            .map_err(|e| {
                WelcomeError::DatabaseError(format!("Failed to serialize admin pubkeys: {}", e))
            })?;

        let group_relays_json = serde_json::to_string(&welcome.group_relays).map_err(|e| {
            WelcomeError::DatabaseError(format!("Failed to serialize group relays: {}", e))
        })?;

        let state_str: String = welcome.state.to_string();

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO welcomes
             (id, event, mls_group_id, nostr_group_id, group_name, group_description,
              group_admin_pubkeys, group_relays, welcomer, member_count, state, wrapper_event_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    &welcome.id.to_bytes(),
                    &event_json,
                    &welcome.mls_group_id,
                    &welcome.nostr_group_id,
                    &welcome.group_name,
                    &welcome.group_description,
                    &group_admin_pubkeys_json,
                    &group_relays_json,
                    &welcome.welcomer.to_bytes(),
                    &(welcome.member_count as i64),
                    &state_str,
                    &welcome.wrapper_event_id.to_bytes()
                ],
            )
            .map_err(|e| WelcomeError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    fn find_welcome_by_event_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<Welcome>, WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            WelcomeError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM welcomes WHERE id = ?")
            .map_err(|e| WelcomeError::DatabaseError(e.to_string()))?;

        match stmt.query_row(params![event_id.to_bytes()], db::row_to_welcome) {
            Ok(welcome) => Ok(Some(welcome)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(WelcomeError::DatabaseError(e.to_string())),
        }
    }

    fn pending_welcomes(&self) -> Result<Vec<Welcome>, WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            WelcomeError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM welcomes WHERE state = 'pending'")
            .map_err(|e| WelcomeError::DatabaseError(e.to_string()))?;

        let welcomes_iter = stmt
            .query_map([], db::row_to_welcome)
            .map_err(|e| WelcomeError::DatabaseError(e.to_string()))?;

        let mut welcomes = Vec::new();
        for welcome_result in welcomes_iter {
            match welcome_result {
                Ok(welcome) => welcomes.push(welcome),
                Err(e) => {
                    return Err(WelcomeError::DatabaseError(format!(
                        "Error parsing welcome: {}",
                        e
                    )))
                }
            }
        }

        Ok(welcomes)
    }

    fn save_processed_welcome(
        &self,
        processed_welcome: ProcessedWelcome,
    ) -> Result<(), WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            WelcomeError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        // Convert welcome_event_id to string if it exists
        let welcome_event_id = processed_welcome
            .welcome_event_id
            .as_ref()
            .map(|id| id.to_bytes());

        let state_str: String = processed_welcome.state.to_string();

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO processed_welcomes
             (wrapper_event_id, welcome_event_id, processed_at, state, failure_reason)
             VALUES (?, ?, ?, ?, ?)",
                params![
                    &processed_welcome.wrapper_event_id.to_bytes(),
                    &welcome_event_id,
                    &processed_welcome.processed_at.as_u64(),
                    &state_str,
                    &processed_welcome.failure_reason
                ],
            )
            .map_err(|e| WelcomeError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    fn find_processed_welcome_by_event_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<ProcessedWelcome>, WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            WelcomeError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM processed_welcomes WHERE wrapper_event_id = ?")
            .map_err(|e| WelcomeError::DatabaseError(e.to_string()))?;

        match stmt.query_row(params![event_id.to_bytes()], db::row_to_processed_welcome) {
            Ok(welcome) => Ok(Some(welcome)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(WelcomeError::DatabaseError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use nostr::{EventId, Kind, PublicKey, Timestamp, UnsignedEvent};
    use nostr_mls_storage::groups::types::{Group, GroupState, GroupType};
    use nostr_mls_storage::groups::GroupStorage;
    use nostr_mls_storage::welcomes::types::{ProcessedWelcomeState, WelcomeState};

    use super::*;

    #[test]
    fn test_save_and_find_welcome() {
        let storage = crate::NostrMlsSqliteStorage::new_in_memory().unwrap();

        // First create a group (welcomes require a valid group foreign key)
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
        let result = storage.save_group(group);
        assert!(result.is_ok());

        // Create a test welcome
        let event_id =
            EventId::parse("6a2affe9878ebcf50c10cf74c7b25aad62e0db9fb347f6aafeda30e9f578f260")
                .unwrap();
        let pubkey =
            PublicKey::parse("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
                .unwrap();
        let wrapper_event_id =
            EventId::parse("3287abd422284bc3679812c373c52ed4aa0af4f7c57b9c63ec440f6c3ed6c3a2")
                .unwrap();

        let welcome = Welcome {
            id: event_id,
            event: UnsignedEvent::new(
                pubkey,
                Timestamp::now(),
                Kind::MlsWelcome,
                vec![],
                "content".to_string(),
            ),
            mls_group_id: mls_group_id.clone(),
            nostr_group_id: "test_group_123".to_string(),
            group_name: "Test Group".to_string(),
            group_description: "A test group".to_string(),
            group_admin_pubkeys: vec![
                "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798".to_string(),
            ],
            group_relays: vec!["wss://relay.example.com".to_string()],
            welcomer: pubkey,
            member_count: 3,
            state: WelcomeState::Pending,
            wrapper_event_id,
        };

        // Save the welcome
        let result = storage.save_welcome(welcome.clone());
        assert!(result.is_ok());

        // Find by event ID
        let found_welcome = storage
            .find_welcome_by_event_id(&event_id)
            .unwrap()
            .unwrap();
        assert_eq!(found_welcome.id, event_id);
        assert_eq!(found_welcome.nostr_group_id, "test_group_123");
        assert_eq!(found_welcome.state, WelcomeState::Pending);

        // Test pending welcomes
        let pending_welcomes = storage.pending_welcomes().unwrap();
        assert_eq!(pending_welcomes.len(), 1);
        assert_eq!(pending_welcomes[0].id, event_id);
    }

    #[test]
    fn test_processed_welcome() {
        let storage = crate::NostrMlsSqliteStorage::new_in_memory().unwrap();

        // Create a test processed welcome
        let wrapper_event_id =
            EventId::parse("6a2affe9878ebcf50c10cf74c7b25aad62e0db9fb347f6aafeda30e9f578f260")
                .unwrap();
        let welcome_event_id =
            EventId::parse("3287abd422284bc3679812c373c52ed4aa0af4f7c57b9c63ec440f6c3ed6c3a2")
                .unwrap();

        let processed_welcome = ProcessedWelcome {
            wrapper_event_id,
            welcome_event_id: Some(welcome_event_id),
            processed_at: Timestamp::from(1_000_000_000u64),
            state: ProcessedWelcomeState::Processed,
            failure_reason: "".to_string(),
        };

        // Save the processed welcome
        let result = storage.save_processed_welcome(processed_welcome.clone());
        assert!(result.is_ok());

        // Find by event ID
        let found_processed_welcome = storage
            .find_processed_welcome_by_event_id(&wrapper_event_id)
            .unwrap()
            .unwrap();
        assert_eq!(found_processed_welcome.wrapper_event_id, wrapper_event_id);
        assert_eq!(
            found_processed_welcome.welcome_event_id.unwrap(),
            welcome_event_id
        );
        assert_eq!(
            found_processed_welcome.state,
            ProcessedWelcomeState::Processed
        );
    }
}
