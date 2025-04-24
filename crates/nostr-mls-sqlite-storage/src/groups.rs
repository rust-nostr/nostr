//! Implementation of GroupStorage trait for SQLite storage.

use std::collections::BTreeSet;

use nostr::PublicKey;
use nostr_mls_storage::groups::error::GroupError;
use nostr_mls_storage::groups::types::{Group, GroupRelay};
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::types::Message;
use rusqlite::{params, OptionalExtension};

use crate::{db, NostrMlsSqliteStorage};

#[inline]
fn into_group_err<T>(e: T) -> GroupError
where
    T: std::error::Error,
{
    GroupError::DatabaseError(e.to_string())
}

impl GroupStorage for NostrMlsSqliteStorage {
    fn all_groups(&self) -> Result<Vec<Group>, GroupError> {
        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM groups")
            .map_err(into_group_err)?;

        let groups_iter = stmt
            .query_map([], db::row_to_group)
            .map_err(into_group_err)?;

        let mut groups: Vec<Group> = Vec::new();

        for group_result in groups_iter {
            // TODO: simply skip parsing errors? Or log them? Instead of block the whole request
            let group: Group = group_result.map_err(into_group_err)?;
            groups.push(group);
        }

        Ok(groups)
    }

    fn find_group_by_mls_group_id(&self, mls_group_id: &[u8]) -> Result<Option<Group>, GroupError> {
        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM groups WHERE mls_group_id = ?")
            .map_err(into_group_err)?;

        stmt.query_row([mls_group_id], db::row_to_group)
            .optional()
            .map_err(into_group_err)
    }

    fn find_group_by_nostr_group_id(
        &self,
        nostr_group_id: &str,
    ) -> Result<Option<Group>, GroupError> {
        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM groups WHERE nostr_group_id = ?")
            .map_err(into_group_err)?;

        stmt.query_row(params![nostr_group_id], db::row_to_group)
            .optional()
            .map_err(into_group_err)
    }

    fn save_group(&self, group: Group) -> Result<(), GroupError> {
        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        let admin_pubkeys_json: String =
            serde_json::to_string(&group.admin_pubkeys).map_err(|e| {
                GroupError::DatabaseError(format!("Failed to serialize admin pubkeys: {}", e))
            })?;

        let last_message_id: Option<&[u8; 32]> =
            group.last_message_id.as_ref().map(|id| id.as_bytes());
        let last_message_at: Option<u64> = group.last_message_at.as_ref().map(|ts| ts.as_u64());

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
                    last_message_id,
                    &last_message_at,
                    group.group_type.as_str(),
                    &(group.epoch as i64),
                    group.state.as_str()
                ],
            )
            .map_err(into_group_err)?;

        Ok(())
    }

    fn messages(&self, mls_group_id: &[u8]) -> Result<Vec<Message>, GroupError> {
        // First verify the group exists
        if self.find_group_by_mls_group_id(mls_group_id)?.is_none() {
            return Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                mls_group_id
            )));
        }

        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM messages WHERE mls_group_id = ? ORDER BY created_at DESC")
            .map_err(into_group_err)?;

        let messages_iter = stmt
            .query_map(params![mls_group_id], db::row_to_message)
            .map_err(into_group_err)?;

        let mut messages: Vec<Message> = Vec::new();

        for message_result in messages_iter {
            let message: Message = message_result.map_err(into_group_err)?;
            messages.push(message);
        }

        Ok(messages)
    }

    fn admins(&self, mls_group_id: &[u8]) -> Result<BTreeSet<PublicKey>, GroupError> {
        // Get the group which contains the admin_pubkeys
        match self.find_group_by_mls_group_id(mls_group_id)? {
            Some(group) => Ok(group.admin_pubkeys),
            None => Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                mls_group_id
            ))),
        }
    }

    fn group_relays(&self, mls_group_id: &[u8]) -> Result<BTreeSet<GroupRelay>, GroupError> {
        // First verify the group exists
        if self.find_group_by_mls_group_id(mls_group_id)?.is_none() {
            return Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                mls_group_id
            )));
        }

        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM group_relays WHERE mls_group_id = ?")
            .map_err(into_group_err)?;

        let relays_iter = stmt
            .query_map(params![mls_group_id], db::row_to_group_relay)
            .map_err(into_group_err)?;

        let mut relays: BTreeSet<GroupRelay> = BTreeSet::new();

        for relay_result in relays_iter {
            let relay: GroupRelay = relay_result.map_err(into_group_err)?;
            relays.insert(relay);
        }

        Ok(relays)
    }

    fn save_group_relay(&self, group_relay: GroupRelay) -> Result<(), GroupError> {
        // First verify the group exists
        if self
            .find_group_by_mls_group_id(&group_relay.mls_group_id)?
            .is_none()
        {
            return Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                group_relay.mls_group_id
            )));
        }

        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO group_relays (mls_group_id, relay_url) VALUES (?, ?)",
                params![group_relay.mls_group_id, group_relay.relay_url.as_str()],
            )
            .map_err(into_group_err)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use nostr::RelayUrl;
    use nostr_mls_storage::groups::types::{GroupState, GroupType};

    use super::*;

    #[test]
    fn test_save_and_find_group() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();

        // Create a test group
        let mls_group_id = vec![1, 2, 3, 4];
        let group = Group {
            mls_group_id: mls_group_id.clone(),
            nostr_group_id: "test_group_123".to_string(),
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
        let result = storage.save_group(group.clone());
        assert!(result.is_ok());

        // Find by MLS group ID
        let found_group = storage
            .find_group_by_mls_group_id(&mls_group_id)
            .unwrap()
            .unwrap();
        assert_eq!(found_group.nostr_group_id, "test_group_123");

        // Find by Nostr group ID
        let found_group = storage
            .find_group_by_nostr_group_id("test_group_123")
            .unwrap()
            .unwrap();
        assert_eq!(found_group.mls_group_id, mls_group_id);

        // Get all groups
        let all_groups = storage.all_groups().unwrap();
        assert_eq!(all_groups.len(), 1);
    }

    #[test]
    fn test_group_relay() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();

        // Create a test group
        let mls_group_id = vec![1, 2, 3, 4];
        let group = Group {
            mls_group_id: mls_group_id.clone(),
            nostr_group_id: "test_group_123".to_string(),
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
        assert_eq!(
            relays.first().unwrap().relay_url.to_string(),
            "wss://relay.example.com"
        );
    }
}
