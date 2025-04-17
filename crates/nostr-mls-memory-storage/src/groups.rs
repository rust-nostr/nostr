//! Memory-based storage implementation of the NostrMlsStorageProvider trait for Nostr MLS groups

use nostr::PublicKey;
use nostr_mls_storage::groups::error::GroupError;
use nostr_mls_storage::groups::types::*;
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::types::Message;

use crate::NostrMlsMemoryStorage;

impl GroupStorage for NostrMlsMemoryStorage {
    fn save_group(&self, group: Group) -> Result<Group, GroupError> {
        // Store in the MLS group ID cache
        {
            let mut cache = self.groups_cache.write();
            cache.put(group.mls_group_id.clone(), group.clone());
        }

        // Store in the Nostr group ID cache
        {
            let mut cache = self.groups_by_nostr_id_cache.write();
            cache.put(group.nostr_group_id.clone(), group.clone());
        }

        Ok(group)
    }

    fn all_groups(&self) -> Result<Vec<Group>, GroupError> {
        let cache = self.groups_cache.read();
        // Convert the values from the cache to a Vec
        let groups: Vec<Group> = cache.iter().map(|(_, v)| v.clone()).collect();
        Ok(groups)
    }

    fn find_group_by_mls_group_id(&self, mls_group_id: &[u8]) -> Result<Group, GroupError> {
        let cache = self.groups_cache.read();
        if let Some(group) = cache.peek(mls_group_id) {
            // Return a clone of the found group
            return Ok(group.clone());
        }

        Err(GroupError::NotFound)
    }

    fn find_group_by_nostr_group_id(&self, nostr_group_id: &str) -> Result<Group, GroupError> {
        let cache = self.groups_by_nostr_id_cache.read();
        if let Some(group) = cache.peek(nostr_group_id) {
            // Return a clone of the found group
            return Ok(group.clone());
        }

        Err(GroupError::NotFound)
    }

    fn messages(&self, mls_group_id: &[u8]) -> Result<Vec<Message>, GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(mls_group_id)?;

        let cache = self.messages_by_group_cache.read();
        if let Some(messages) = cache.peek(mls_group_id) {
            return Ok(messages.clone());
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

        let cache = self.group_relays_cache.read();
        if let Some(relays) = cache.peek(mls_group_id) {
            return Ok(relays.clone());
        }

        // If not in cache but group exists, return empty vector
        Ok(Vec::new())
    }

    fn save_group_relay(&self, group_relay: GroupRelay) -> Result<GroupRelay, GroupError> {
        let mls_group_id = group_relay.mls_group_id.clone();

        // Check if the group exists first
        self.find_group_by_mls_group_id(&mls_group_id)?;

        let group_relay_clone = group_relay.clone();

        {
            let mut cache = self.group_relays_cache.write();
            // Get existing relays or create new vector
            let relays = match cache.get(&mls_group_id) {
                Some(existing_relays) => {
                    let mut new_relays = existing_relays.clone();
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
            cache.put(mls_group_id, relays);
        }

        Ok(group_relay)
    }
}
