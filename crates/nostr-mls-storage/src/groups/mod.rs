use nostr::PublicKey;

pub mod error;
pub mod types;

use self::error::GroupError;
use self::types::*;
use crate::messages::types::Message;

pub trait GroupStorage {
    fn all_groups(&self) -> Result<Vec<Group>, GroupError>;
    fn find_group_by_mls_group_id(&self, mls_group_id: &[u8]) -> Result<Group, GroupError>;
    fn find_group_by_nostr_group_id(&self, nostr_group_id: &str) -> Result<Group, GroupError>;
    fn save_group(&self, group: Group) -> Result<Group, GroupError>;
    fn messages(&self, mls_group_id: &[u8]) -> Result<Vec<Message>, GroupError>;
    fn admins(&self, mls_group_id: &[u8]) -> Result<Vec<PublicKey>, GroupError>;
    fn group_relays(&self, mls_group_id: &[u8]) -> Result<Vec<GroupRelay>, GroupError>;
    fn save_group_relay(&self, group_relay: GroupRelay) -> Result<GroupRelay, GroupError>;
}
