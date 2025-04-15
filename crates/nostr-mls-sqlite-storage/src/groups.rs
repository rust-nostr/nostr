/// Implementation of GroupStorage trait for SQLite storage.
use crate::db;
use crate::NostrMlsSqliteStorage;
use nostr::PublicKey;
use nostr_mls_storage::groups::error::GroupError;
use nostr_mls_storage::groups::types::{Group, GroupRelay};
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::types::Message;
use rusqlite::params;

impl GroupStorage for NostrMlsSqliteStorage {
    fn all_groups(&self) -> Result<Vec<Group>, GroupError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            GroupError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM groups")
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        let groups_iter = stmt
            .query_map([], db::row_to_group)
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        let mut groups = Vec::new();
        for group_result in groups_iter {
            match group_result {
                Ok(group) => groups.push(group),
                Err(e) => {
                    return Err(GroupError::DatabaseError(format!(
                        "Error parsing group: {}",
                        e
                    )))
                }
            }
        }

        Ok(groups)
    }

    fn find_group_by_mls_group_id(&self, mls_group_id: &[u8]) -> Result<Group, GroupError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            GroupError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM groups WHERE mls_group_id = ?")
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        let group = stmt
            .query_row(params![mls_group_id], db::row_to_group)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => GroupError::NotFound,
                _ => GroupError::DatabaseError(e.to_string()),
            })?;

        Ok(group)
    }

    fn find_group_by_nostr_group_id(&self, nostr_group_id: &str) -> Result<Group, GroupError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            GroupError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM groups WHERE nostr_group_id = ?")
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        let group = stmt
            .query_row(params![nostr_group_id], db::row_to_group)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => GroupError::NotFound,
                _ => GroupError::DatabaseError(e.to_string()),
            })?;

        Ok(group)
    }

    fn save_group(&self, group: Group) -> Result<Group, GroupError> {
        let conn_guard = self.db_connection.lock().map_err(|_| {
            GroupError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        // Convert admin_pubkeys to JSON
        let admin_pubkeys_json = serde_json::to_string(
            &group
                .admin_pubkeys
                .iter()
                .map(|pk| pk.to_string())
                .collect::<Vec<String>>(),
        )
        .map_err(|e| {
            GroupError::DatabaseError(format!("Failed to serialize admin pubkeys: {}", e))
        })?;

        // Convert last_message_id to string if it exists
        let last_message_id = group.last_message_id.as_ref().map(|id| id.to_string());

        // Convert last_message_at to i64 if it exists
        let last_message_at = group.last_message_at.as_ref().map(|ts| ts.as_u64());

        // Convert group_type and state to strings
        let group_type_str: String = group.group_type.clone().into();
        let state_str: String = group.state.clone().into();

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO groups
             (mls_group_id, nostr_group_id, name, description, admin_pubkeys, last_message_id,
              last_message_at, group_type, epoch, state)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    &group.mls_group_id,
                    &group.nostr_group_id,
                    &group.name,
                    &group.description,
                    &admin_pubkeys_json,
                    &last_message_id,
                    &last_message_at,
                    &group_type_str,
                    &(group.epoch as i64),
                    &state_str
                ],
            )
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        Ok(group)
    }

    fn messages(&self, mls_group_id: &[u8]) -> Result<Vec<Message>, GroupError> {
        // First verify the group exists
        self.find_group_by_mls_group_id(mls_group_id)?;

        let conn_guard = self.db_connection.lock().map_err(|_| {
            GroupError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM messages WHERE mls_group_id = ? ORDER BY created_at DESC")
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        let messages_iter = stmt
            .query_map(params![mls_group_id], db::row_to_message)
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        let mut messages = Vec::new();
        for message_result in messages_iter {
            match message_result {
                Ok(message) => messages.push(message),
                Err(e) => {
                    return Err(GroupError::DatabaseError(format!(
                        "Error parsing message: {}",
                        e
                    )))
                }
            }
        }

        Ok(messages)
    }

    fn admins(&self, mls_group_id: &[u8]) -> Result<Vec<PublicKey>, GroupError> {
        // Get the group which contains the admin_pubkeys
        let group = self.find_group_by_mls_group_id(mls_group_id)?;

        // Return the admin pubkeys
        Ok(group.admin_pubkeys)
    }

    fn group_relays(&self, mls_group_id: &[u8]) -> Result<Vec<GroupRelay>, GroupError> {
        // First verify the group exists
        self.find_group_by_mls_group_id(mls_group_id)?;

        let conn_guard = self.db_connection.lock().map_err(|_| {
            GroupError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM group_relays WHERE mls_group_id = ?")
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        let relays_iter = stmt
            .query_map(params![mls_group_id], db::row_to_group_relay)
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        let mut relays = Vec::new();
        for relay_result in relays_iter {
            match relay_result {
                Ok(relay) => relays.push(relay),
                Err(e) => {
                    return Err(GroupError::DatabaseError(format!(
                        "Error parsing group relay: {}",
                        e
                    )))
                }
            }
        }

        Ok(relays)
    }

    fn save_group_relay(&self, group_relay: GroupRelay) -> Result<GroupRelay, GroupError> {
        // First verify the group exists
        self.find_group_by_mls_group_id(&group_relay.mls_group_id)?;

        let conn_guard = self.db_connection.lock().map_err(|_| {
            GroupError::DatabaseError("Failed to acquire database lock".to_string())
        })?;

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO group_relays (mls_group_id, relay_url) VALUES (?, ?)",
                params![
                    &group_relay.mls_group_id,
                    &group_relay.relay_url.to_string()
                ],
            )
            .map_err(|e| GroupError::DatabaseError(e.to_string()))?;

        Ok(group_relay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::RelayUrl;
    use nostr_mls_storage::groups::types::{GroupState, GroupType};

    #[test]
    fn test_save_and_find_group() {
        let storage = crate::NostrMlsSqliteStorage::new_in_memory().unwrap();

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
        let result = storage.save_group(group.clone());
        assert!(result.is_ok());

        // Find by MLS group ID
        let found_group = storage.find_group_by_mls_group_id(&mls_group_id).unwrap();
        assert_eq!(found_group.nostr_group_id, "test_group_123");

        // Find by Nostr group ID
        let found_group = storage
            .find_group_by_nostr_group_id("test_group_123")
            .unwrap();
        assert_eq!(found_group.mls_group_id, mls_group_id);

        // Get all groups
        let all_groups = storage.all_groups().unwrap();
        assert_eq!(all_groups.len(), 1);
    }

    #[test]
    fn test_group_relay() {
        let storage = crate::NostrMlsSqliteStorage::new_in_memory().unwrap();

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
        let result = storage.save_group(group.clone());
        assert!(result.is_ok());

        // Create a group relay
        let relay_url = RelayUrl::parse("wss://relay.example.com").unwrap();
        let group_relay = GroupRelay {
            mls_group_id: mls_group_id.clone(),
            relay_url,
        };

        // Save the group relay
        let result = storage.save_group_relay(group_relay.clone());
        assert!(result.is_ok());

        // Get group relays
        let relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(relays.len(), 1);
        assert_eq!(relays[0].relay_url.to_string(), "wss://relay.example.com");
    }
}
