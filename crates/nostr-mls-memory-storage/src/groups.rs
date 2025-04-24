//! Memory-based storage implementation of the NostrMlsStorageProvider trait for Nostr MLS groups

use std::collections::BTreeSet;

use nostr::PublicKey;
use nostr_mls_storage::groups::error::{GroupError, InvalidGroupState};
use nostr_mls_storage::groups::types::*;
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::types::Message;

use crate::NostrMlsMemoryStorage;

impl GroupStorage for NostrMlsMemoryStorage {
    fn save_group(&self, group: Group) -> Result<(), GroupError> {
        // Store in the MLS group ID cache
        {
            let mut cache = self.groups_cache.write();
            cache.put(group.mls_group_id.clone(), group.clone());
        }

        // Store in the Nostr group ID cache
        {
            let mut cache = self.groups_by_nostr_id_cache.write();
            cache.put(group.nostr_group_id.clone(), group);
        }

        Ok(())
    }

    fn all_groups(&self) -> Result<Vec<Group>, GroupError> {
        let cache = self.groups_cache.read();
        // Convert the values from the cache to a Vec
        let groups: Vec<Group> = cache.iter().map(|(_, v)| v.clone()).collect();
        Ok(groups)
    }

    fn find_group_by_mls_group_id(&self, mls_group_id: &[u8]) -> Result<Option<Group>, GroupError> {
        let cache = self.groups_cache.read();
        Ok(cache.peek(mls_group_id).cloned())
    }

    fn find_group_by_nostr_group_id(
        &self,
        nostr_group_id: &str,
    ) -> Result<Option<Group>, GroupError> {
        let cache = self.groups_by_nostr_id_cache.read();
        Ok(cache.peek(nostr_group_id).cloned())
    }

    fn messages(&self, mls_group_id: &[u8]) -> Result<Vec<Message>, GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(mls_group_id)?;

        let cache = self.messages_by_group_cache.read();
        match cache.peek(mls_group_id).cloned() {
            Some(messages) => Ok(messages),
            // If not in cache but group exists, return empty vector
            None => Ok(Vec::new()),
        }
    }

    fn admins(&self, mls_group_id: &[u8]) -> Result<BTreeSet<PublicKey>, GroupError> {
        match self.find_group_by_mls_group_id(mls_group_id)? {
            Some(group) => Ok(group.admin_pubkeys),
            None => Err(GroupError::InvalidState(InvalidGroupState::NoAdmins)),
        }
    }

    fn group_relays(&self, mls_group_id: &[u8]) -> Result<BTreeSet<GroupRelay>, GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(mls_group_id)?;

        let cache = self.group_relays_cache.read();
        match cache.peek(mls_group_id).cloned() {
            Some(relays) => Ok(relays),
            None => Err(GroupError::InvalidState(InvalidGroupState::NoRelays)),
        }
    }

    fn save_group_relay(&self, group_relay: GroupRelay) -> Result<(), GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(&group_relay.mls_group_id)?;

        let mut cache = self.group_relays_cache.write();

        // Try to get the existing relays for the group
        match cache.get_mut(&group_relay.mls_group_id) {
            // If the group exists, add the new relay to the vector
            Some(existing_relays) => {
                // Add the new relay if it doesn't already exist
                existing_relays.insert(group_relay);
            }
            // If the group doesn't exist, create a new vector with the new relay
            None => {
                // Update the cache with the new vector
                cache.put(
                    group_relay.mls_group_id.clone(),
                    BTreeSet::from([group_relay]),
                );
            }
        };

        Ok(())
    }
}
