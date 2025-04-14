use crate::NostrMlsMemoryStorage;
use crate::CURRENT_VERSION;
use nostr::PublicKey;
use nostr_mls_storage::groups::error::GroupError;
use nostr_mls_storage::groups::types::*;
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::types::Message;
use std::sync::Arc;

use openmls_traits::storage::StorageProvider;
impl<S: StorageProvider<CURRENT_VERSION>> GroupStorage for NostrMlsMemoryStorage<S> {
    fn save_group(&self, group: Group) -> Result<Group, GroupError> {
        // Create Arc for the group
        let group_arc = Arc::new(group.clone());

        // Store in the MLS group ID cache
        if let Ok(mut cache) = self.groups_cache.write() {
            cache.put(group_arc.mls_group_id.clone(), Arc::clone(&group_arc));
        } else {
            return Err(GroupError::DatabaseError(
                "Failed to acquire write lock on groups cache".to_string(),
            ));
        }

        // Store in the Nostr group ID cache
        if let Ok(mut cache) = self.groups_by_nostr_id_cache.write() {
            cache.put(group_arc.nostr_group_id.clone(), Arc::clone(&group_arc));
        } else {
            return Err(GroupError::DatabaseError(
                "Failed to acquire write lock on groups by nostr ID cache".to_string(),
            ));
        }

        Ok(group)
    }

    fn all_groups(&self) -> Result<Vec<Group>, GroupError> {
        if let Ok(cache) = self.groups_cache.read() {
            // Convert the values from the cache to a Vec
            let groups: Vec<Group> = cache.iter().map(|(_, v)| (**v).clone()).collect();
            return Ok(groups);
        }

        Err(GroupError::DatabaseError(
            "Failed to acquire read lock on groups cache".to_string(),
        ))
    }

    fn find_group_by_mls_group_id(&self, mls_group_id: &[u8]) -> Result<Group, GroupError> {
        if let Ok(cache) = self.groups_cache.read() {
            if let Some(group_arc) = cache.peek(mls_group_id) {
                // Return a clone of the found group
                return Ok((**group_arc).clone());
            }
        } else {
            return Err(GroupError::DatabaseError(
                "Failed to acquire read lock on groups cache".to_string(),
            ));
        }

        Err(GroupError::NotFound)
    }

    fn find_group_by_nostr_group_id(&self, nostr_group_id: &str) -> Result<Group, GroupError> {
        if let Ok(cache) = self.groups_by_nostr_id_cache.read() {
            if let Some(group_arc) = cache.peek(nostr_group_id) {
                // Return a clone of the found group
                return Ok((**group_arc).clone());
            }
        } else {
            return Err(GroupError::DatabaseError(
                "Failed to acquire read lock on groups by nostr ID cache".to_string(),
            ));
        }

        Err(GroupError::NotFound)
    }

    fn messages(&self, mls_group_id: &[u8]) -> Result<Vec<Message>, GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(mls_group_id)?;

        if let Ok(cache) = self.messages_by_group_cache.read() {
            if let Some(messages_arc) = cache.peek(mls_group_id) {
                return Ok((**messages_arc).clone());
            }
        } else {
            return Err(GroupError::DatabaseError(
                "Failed to acquire read lock on messages by group cache".to_string(),
            ));
        }

        // If not in cache but group exists, return empty vector
        Ok(Vec::new())
    }

    fn admins(&self, mls_group_id: &[u8]) -> Result<Vec<PublicKey>, GroupError> {
        // Find the group first
        if let Ok(group) = self.find_group_by_mls_group_id(mls_group_id) {
            // Return the admin pubkeys from the group
            return Ok(group.admin_pubkeys);
        }

        Err(GroupError::NotFound)
    }

    fn group_relays(&self, mls_group_id: &[u8]) -> Result<Vec<GroupRelay>, GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(mls_group_id)?;

        if let Ok(cache) = self.group_relays_cache.read() {
            if let Some(relays_arc) = cache.peek(mls_group_id) {
                return Ok((**relays_arc).clone());
            }
        } else {
            return Err(GroupError::DatabaseError(
                "Failed to acquire read lock on group relays cache".to_string(),
            ));
        }

        // If not in cache but group exists, return empty vector
        Ok(Vec::new())
    }

    fn save_group_relay(&self, group_relay: GroupRelay) -> Result<GroupRelay, GroupError> {
        let mls_group_id = group_relay.mls_group_id.clone();

        // Check if the group exists first
        self.find_group_by_mls_group_id(&mls_group_id)?;

        let group_relay_clone = group_relay.clone();

        if let Ok(mut cache) = self.group_relays_cache.write() {
            // Get existing relays or create new vector
            let relays = match cache.get(&mls_group_id) {
                Some(existing_relays_arc) => {
                    let mut new_relays = (**existing_relays_arc).clone();
                    // Add the new relay if it doesn't already exist
                    if !new_relays
                        .iter()
                        .any(|r| r.relay_url == group_relay.relay_url)
                    {
                        new_relays.push(group_relay_clone);
                    }
                    new_relays
                }
                None => vec![group_relay_clone],
            };

            // Update the cache with the new vector
            cache.put(mls_group_id, Arc::new(relays));
        } else {
            return Err(GroupError::DatabaseError(
                "Failed to acquire write lock on group relays cache".to_string(),
            ));
        }

        Ok(group_relay)
    }
}
