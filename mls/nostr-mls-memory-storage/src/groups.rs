//! Memory-based storage implementation of the NostrMlsStorageProvider trait for Nostr MLS groups

use std::collections::BTreeSet;

use nostr::{PublicKey, RelayUrl};
use nostr_mls_storage::groups::error::{GroupError, InvalidGroupState};
use nostr_mls_storage::groups::types::*;
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::types::Message;
use openmls::group::GroupId;

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
            cache.put(group.nostr_group_id, group);
        }

        Ok(())
    }

    fn all_groups(&self) -> Result<Vec<Group>, GroupError> {
        let cache = self.groups_cache.read();
        // Convert the values from the cache to a Vec
        let groups: Vec<Group> = cache.iter().map(|(_, v)| v.clone()).collect();
        Ok(groups)
    }

    fn find_group_by_mls_group_id(
        &self,
        mls_group_id: &GroupId,
    ) -> Result<Option<Group>, GroupError> {
        let cache = self.groups_cache.read();
        Ok(cache.peek(mls_group_id).cloned())
    }

    fn find_group_by_nostr_group_id(
        &self,
        nostr_group_id: &[u8; 32],
    ) -> Result<Option<Group>, GroupError> {
        let cache = self.groups_by_nostr_id_cache.read();
        Ok(cache.peek(nostr_group_id).cloned())
    }

    fn messages(&self, mls_group_id: &GroupId) -> Result<Vec<Message>, GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(mls_group_id)?;

        let cache = self.messages_by_group_cache.read();
        match cache.peek(mls_group_id).cloned() {
            Some(messages) => Ok(messages),
            // If not in cache but group exists, return empty vector
            None => Ok(Vec::new()),
        }
    }

    fn admins(&self, mls_group_id: &GroupId) -> Result<BTreeSet<PublicKey>, GroupError> {
        match self.find_group_by_mls_group_id(mls_group_id)? {
            Some(group) => Ok(group.admin_pubkeys),
            None => Err(GroupError::InvalidState(InvalidGroupState::NoAdmins)),
        }
    }

    fn group_relays(&self, mls_group_id: &GroupId) -> Result<BTreeSet<GroupRelay>, GroupError> {
        // Check if the group exists first
        if self.find_group_by_mls_group_id(mls_group_id)?.is_none() {
            return Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                mls_group_id
            )));
        }

        let cache = self.group_relays_cache.read();
        match cache.peek(mls_group_id).cloned() {
            Some(relays) => Ok(relays),
            // If not in cache but group exists, return empty set
            None => Ok(BTreeSet::new()),
        }
    }

    fn replace_group_relays(
        &self,
        group_id: &GroupId,
        relays: BTreeSet<RelayUrl>,
    ) -> Result<(), GroupError> {
        // Check if the group exists first
        if self.find_group_by_mls_group_id(group_id)?.is_none() {
            return Err(GroupError::InvalidParameters(format!(
                "Group with MLS ID {:?} not found",
                group_id
            )));
        }

        let mut cache = self.group_relays_cache.write();

        // Convert RelayUrl set to GroupRelay set
        let group_relays: BTreeSet<GroupRelay> = relays
            .into_iter()
            .map(|relay_url| GroupRelay {
                mls_group_id: group_id.clone(),
                relay_url,
            })
            .collect();

        // Replace the entire relay set for this group
        cache.put(group_id.clone(), group_relays);

        Ok(())
    }

    fn get_group_exporter_secret(
        &self,
        mls_group_id: &GroupId,
        epoch: u64,
    ) -> Result<Option<GroupExporterSecret>, GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(mls_group_id)?;

        let cache = self.group_exporter_secrets_cache.read();
        // Use tuple (GroupId, epoch) as key
        Ok(cache.peek(&(mls_group_id.clone(), epoch)).cloned())
    }

    fn save_group_exporter_secret(
        &self,
        group_exporter_secret: GroupExporterSecret,
    ) -> Result<(), GroupError> {
        // Check if the group exists first
        self.find_group_by_mls_group_id(&group_exporter_secret.mls_group_id)?;

        let mut cache = self.group_exporter_secrets_cache.write();
        // Use tuple (GroupId, epoch) as key
        let key = (
            group_exporter_secret.mls_group_id.clone(),
            group_exporter_secret.epoch,
        );
        cache.put(key, group_exporter_secret);

        Ok(())
    }
}
