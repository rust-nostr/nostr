use crate::NostrMlsMemoryStorage;
use crate::CURRENT_VERSION;
use nostr::PublicKey;
use nostr_mls_storage::groups::error::GroupError;
use nostr_mls_storage::groups::types::*;
use nostr_mls_storage::groups::GroupStorage;
use nostr_mls_storage::messages::types::Message;

use openmls_traits::storage::StorageProvider;
impl<S: StorageProvider<CURRENT_VERSION>> GroupStorage for NostrMlsMemoryStorage<S> {
    fn create_group(&self, group: Group) -> Result<Group, GroupError> {
        todo!()
    }

    fn all_groups(&self) -> Result<Vec<Group>, GroupError> {
        todo!()
    }

    fn find_group_by_mls_group_id(&self, mls_group_id: &[u8]) -> Result<Group, GroupError> {
        todo!()
    }

    fn find_group_by_nostr_group_id(&self, nostr_group_id: &str) -> Result<Group, GroupError> {
        todo!()
    }

    fn save_group(&self, group: Group) -> Result<Group, GroupError> {
        todo!()
    }

    fn delete_group(&self, mls_group_id: &[u8]) -> Result<(), GroupError> {
        todo!()
    }

    fn messages(&self, mls_group_id: &[u8]) -> Result<Vec<Message>, GroupError> {
        todo!()
    }

    fn members(&self, mls_group_id: &[u8]) -> Result<Vec<PublicKey>, GroupError> {
        todo!()
    }

    fn admins(&self, mls_group_id: &[u8]) -> Result<Vec<PublicKey>, GroupError> {
        todo!()
    }

    fn group_relays(&self, mls_group_id: &[u8]) -> Result<Vec<GroupRelay>, GroupError> {
        todo!()
    }

    fn self_update_keys(&self, mls_group_id: &[u8]) -> Result<Group, GroupError> {
        todo!()
    }

    fn create_group_relay(&self, group_relay: GroupRelay) -> Result<GroupRelay, GroupError> {
        todo!()
    }

    fn delete_group_relay(&self, group_relay: GroupRelay) -> Result<GroupRelay, GroupError> {
        todo!()
    }
}
