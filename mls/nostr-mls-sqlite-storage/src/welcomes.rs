//! Implementation of WelcomeStorage trait for SQLite storage.

use nostr::{EventId, JsonUtil};
use nostr_mls_storage::welcomes::WelcomeStorage;
use nostr_mls_storage::welcomes::error::WelcomeError;
use nostr_mls_storage::welcomes::types::{ProcessedWelcome, Welcome};
use rusqlite::{OptionalExtension, params};

use crate::db::{Hash32, Nonce12};
use crate::{NostrMlsSqliteStorage, db};

#[inline]
fn into_welcome_err<T>(e: T) -> WelcomeError
where
    T: std::error::Error,
{
    WelcomeError::DatabaseError(e.to_string())
}

impl WelcomeStorage for NostrMlsSqliteStorage {
    fn save_welcome(&self, welcome: Welcome) -> Result<(), WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(into_welcome_err)?;

        // Serialize complex types to JSON
        let group_admin_pubkeys_json: String = serde_json::to_string(&welcome.group_admin_pubkeys)
            .map_err(|e| {
                WelcomeError::DatabaseError(format!("Failed to serialize admin pubkeys: {}", e))
            })?;

        let group_relays_json: String =
            serde_json::to_string(&welcome.group_relays).map_err(|e| {
                WelcomeError::DatabaseError(format!("Failed to serialize group relays: {}", e))
            })?;

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO welcomes
             (id, event, mls_group_id, nostr_group_id, group_name, group_description, group_image_hash, group_image_key, group_image_nonce,
              group_admin_pubkeys, group_relays, welcomer, member_count, state, wrapper_event_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    welcome.id.as_bytes(),
                    welcome.event.as_json(),
                    welcome.mls_group_id.as_slice(),
                    welcome.nostr_group_id,
                    welcome.group_name,
                    welcome.group_description,
                    welcome.group_image_hash.map(Hash32::from),
                    welcome.group_image_key.map(Hash32::from),
                    welcome.group_image_nonce.map(Nonce12::from),
                    group_admin_pubkeys_json,
                    group_relays_json,
                    welcome.welcomer.as_bytes(),
                    welcome.member_count as u64,
                    welcome.state.as_str(),
                    welcome.wrapper_event_id.as_bytes(),
                ],
            )
            .map_err(into_welcome_err)?;

        Ok(())
    }

    fn find_welcome_by_event_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<Welcome>, WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(into_welcome_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM welcomes WHERE id = ?")
            .map_err(into_welcome_err)?;

        stmt.query_row(params![event_id.as_bytes()], db::row_to_welcome)
            .optional()
            .map_err(into_welcome_err)
    }

    fn pending_welcomes(&self) -> Result<Vec<Welcome>, WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(into_welcome_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM welcomes WHERE state = 'pending'")
            .map_err(into_welcome_err)?;

        let welcomes_iter = stmt
            .query_map([], db::row_to_welcome)
            .map_err(into_welcome_err)?;

        let mut welcomes: Vec<Welcome> = Vec::new();

        for welcome_result in welcomes_iter {
            let welcome: Welcome = welcome_result.map_err(into_welcome_err)?;
            welcomes.push(welcome);
        }

        Ok(welcomes)
    }

    fn save_processed_welcome(
        &self,
        processed_welcome: ProcessedWelcome,
    ) -> Result<(), WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(into_welcome_err)?;

        // Convert welcome_event_id to string if it exists
        let welcome_event_id: Option<&[u8; 32]> = processed_welcome
            .welcome_event_id
            .as_ref()
            .map(|id| id.as_bytes());

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO processed_welcomes
             (wrapper_event_id, welcome_event_id, processed_at, state, failure_reason)
             VALUES (?, ?, ?, ?, ?)",
                params![
                    processed_welcome.wrapper_event_id.as_bytes(),
                    welcome_event_id,
                    processed_welcome.processed_at.as_u64(),
                    processed_welcome.state.as_str(),
                    processed_welcome.failure_reason
                ],
            )
            .map_err(into_welcome_err)?;

        Ok(())
    }

    fn find_processed_welcome_by_event_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<ProcessedWelcome>, WelcomeError> {
        let conn_guard = self.db_connection.lock().map_err(into_welcome_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM processed_welcomes WHERE wrapper_event_id = ?")
            .map_err(into_welcome_err)?;

        stmt.query_row(params![event_id.as_bytes()], db::row_to_processed_welcome)
            .optional()
            .map_err(into_welcome_err)
    }
}

#[cfg(test)]
mod tests {
    use nostr::EventId;
    use nostr_mls_storage::groups::GroupStorage;
    use nostr_mls_storage::test_utils::cross_storage::{
        create_test_group, create_test_processed_welcome, create_test_welcome,
    };
    use nostr_mls_storage::welcomes::types::ProcessedWelcomeState;
    use openmls::group::GroupId;

    use super::*;

    #[test]
    fn test_save_and_find_welcome() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();

        // First create a group (welcomes require a valid group foreign key)
        let mls_group_id = GroupId::from_slice(&[1, 2, 3, 4]);
        let group = create_test_group(mls_group_id.clone());

        // Save the group
        let result = storage.save_group(group);
        assert!(result.is_ok(), "{:?}", result);

        // Create a test welcome using the helper
        let event_id = EventId::all_zeros();
        let welcome = create_test_welcome(mls_group_id.clone(), event_id);

        // Save the welcome
        let result = storage.save_welcome(welcome.clone());
        assert!(result.is_ok(), "{:?}", result);

        // Find by event ID
        let found_welcome = storage
            .find_welcome_by_event_id(&event_id)
            .unwrap()
            .unwrap();
        assert_eq!(found_welcome.id, event_id);
        assert_eq!(found_welcome.mls_group_id, mls_group_id);
        assert_eq!(found_welcome.state, welcome.state);

        // Test pending welcomes
        let pending_welcomes = storage.pending_welcomes().unwrap();
        assert_eq!(pending_welcomes.len(), 1);
        assert_eq!(pending_welcomes[0].id, event_id);
    }

    #[test]
    fn test_processed_welcome() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();

        // Create test event IDs using helper methods
        let wrapper_event_id = EventId::all_zeros();
        let welcome_event_id =
            EventId::from_hex("1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap();

        // Create a test processed welcome using the helper
        let processed_welcome =
            create_test_processed_welcome(wrapper_event_id, Some(welcome_event_id));

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
