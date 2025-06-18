//! Implementation of GroupStorage trait for SQLite storage.

use std::collections::BTreeSet;

use nostr::PublicKey;
use nostr_mls_storage::groups::error::GroupError;
use nostr_mls_storage::groups::types::{Group, GroupExporterSecret, GroupRelay};
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::types::Message;
use openmls::group::GroupId;
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

    fn find_group_by_mls_group_id(
        &self,
        mls_group_id: &GroupId,
    ) -> Result<Option<Group>, GroupError> {
        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM groups WHERE mls_group_id = ?")
            .map_err(into_group_err)?;

        stmt.query_row([mls_group_id.as_slice()], db::row_to_group)
            .optional()
            .map_err(into_group_err)
    }

    fn find_group_by_nostr_group_id(
        &self,
        nostr_group_id: &[u8; 32],
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
                "INSERT INTO groups
             (mls_group_id, nostr_group_id, name, description, admin_pubkeys, last_message_id,
              last_message_at, group_type, epoch, state)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(mls_group_id) DO UPDATE SET
                nostr_group_id = excluded.nostr_group_id,
                name = excluded.name,
                description = excluded.description,
                admin_pubkeys = excluded.admin_pubkeys,
                last_message_id = excluded.last_message_id,
                last_message_at = excluded.last_message_at,
                group_type = excluded.group_type,
                epoch = excluded.epoch,
                state = excluded.state",
                params![
                    &group.mls_group_id.as_slice(),
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

    fn messages(&self, mls_group_id: &GroupId) -> Result<Vec<Message>, GroupError> {
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
            .query_map(params![mls_group_id.as_slice()], db::row_to_message)
            .map_err(into_group_err)?;

        let mut messages: Vec<Message> = Vec::new();

        for message_result in messages_iter {
            let message: Message = message_result.map_err(into_group_err)?;
            messages.push(message);
        }

        Ok(messages)
    }

    fn admins(&self, mls_group_id: &GroupId) -> Result<BTreeSet<PublicKey>, GroupError> {
        // Get the group which contains the admin_pubkeys
        match self.find_group_by_mls_group_id(mls_group_id)? {
            Some(group) => Ok(group.admin_pubkeys),
            None => Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                mls_group_id
            ))),
        }
    }

    fn group_relays(&self, mls_group_id: &GroupId) -> Result<BTreeSet<GroupRelay>, GroupError> {
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
            .query_map(params![mls_group_id.as_slice()], db::row_to_group_relay)
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
                params![
                    group_relay.mls_group_id.as_slice(),
                    group_relay.relay_url.as_str()
                ],
            )
            .map_err(into_group_err)?;

        Ok(())
    }

    fn get_group_exporter_secret(
        &self,
        mls_group_id: &GroupId,
        epoch: u64,
    ) -> Result<Option<GroupExporterSecret>, GroupError> {
        // First verify the group exists
        if self.find_group_by_mls_group_id(mls_group_id)?.is_none() {
            return Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                mls_group_id
            )));
        }

        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        let mut stmt = conn_guard
            .prepare("SELECT * FROM group_exporter_secrets WHERE mls_group_id = ? AND epoch = ?")
            .map_err(into_group_err)?;

        stmt.query_row(
            params![mls_group_id.as_slice(), epoch],
            db::row_to_group_exporter_secret,
        )
        .optional()
        .map_err(into_group_err)
    }

    fn save_group_exporter_secret(
        &self,
        group_exporter_secret: GroupExporterSecret,
    ) -> Result<(), GroupError> {
        if self
            .find_group_by_mls_group_id(&group_exporter_secret.mls_group_id)?
            .is_none()
        {
            return Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                group_exporter_secret.mls_group_id
            )));
        }

        let conn_guard = self.db_connection.lock().map_err(into_group_err)?;

        conn_guard.execute(
            "INSERT OR REPLACE INTO group_exporter_secrets (mls_group_id, epoch, secret) VALUES (?, ?, ?)",
            params![&group_exporter_secret.mls_group_id.as_slice(), &group_exporter_secret.epoch, &group_exporter_secret.secret],
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

        // Find by MLS group ID
        let found_group = storage
            .find_group_by_mls_group_id(&mls_group_id)
            .unwrap()
            .unwrap();
        assert_eq!(found_group.nostr_group_id[0..13], b"test_group_12"[..]);

        // Find by Nostr group ID
        let found_group = storage
            .find_group_by_nostr_group_id(&nostr_group_id)
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

        // Create a group relay
        let relay_url = RelayUrl::parse("wss://relay.example.com").unwrap();
        let group_relay = GroupRelay {
            mls_group_id: mls_group_id.clone(),
            relay_url,
        };

        // Save the group relay
        let result = storage.save_group_relay(group_relay);
        assert!(result.is_ok());

        // Get group relays
        let relays = storage.group_relays(&mls_group_id).unwrap();
        assert_eq!(relays.len(), 1);
        assert_eq!(
            relays.first().unwrap().relay_url.to_string(),
            "wss://relay.example.com"
        );
    }

    #[test]
    fn test_group_exporter_secret() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();

        // Create a test group
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
        storage.save_group(group).unwrap();

        // Create a group exporter secret
        let secret1 = GroupExporterSecret {
            mls_group_id: mls_group_id.clone(),
            epoch: 1,
            secret: [0u8; 32],
        };

        // Save the secret
        storage.save_group_exporter_secret(secret1).unwrap();

        // Get the secret and verify it was saved correctly
        let retrieved_secret = storage
            .get_group_exporter_secret(&mls_group_id, 1)
            .unwrap()
            .unwrap();
        assert_eq!(retrieved_secret.secret, [0u8; 32]);

        // Create a second secret with same group_id and epoch but different secret value
        let secret2 = GroupExporterSecret {
            mls_group_id: mls_group_id.clone(),
            epoch: 1,
            secret: [0u8; 32],
        };

        // Save the second secret - this should replace the first one due to the "OR REPLACE" in the SQL
        storage.save_group_exporter_secret(secret2).unwrap();

        // Get the secret again and verify it was updated
        let retrieved_secret = storage
            .get_group_exporter_secret(&mls_group_id, 1)
            .unwrap()
            .unwrap();
        assert_eq!(retrieved_secret.secret, [0u8; 32]);

        // Verify we can still save a different epoch
        let secret3 = GroupExporterSecret {
            mls_group_id: mls_group_id.clone(),
            epoch: 2,
            secret: [0u8; 32],
        };

        storage.save_group_exporter_secret(secret3).unwrap();

        // Verify both epochs exist
        let retrieved_secret1 = storage
            .get_group_exporter_secret(&mls_group_id, 1)
            .unwrap()
            .unwrap();
        let retrieved_secret2 = storage
            .get_group_exporter_secret(&mls_group_id, 2)
            .unwrap()
            .unwrap();

        assert_eq!(retrieved_secret1.secret, [0u8; 32]);
        assert_eq!(retrieved_secret2.secret, [0u8; 32]);
    }
}
