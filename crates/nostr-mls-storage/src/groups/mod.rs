//! Groups module
//!
//! This module is responsible for storing and retrieving groups
//! It also handles the parsing of group content
//!
//! The groups are stored in the database and can be retrieved by MLS group ID or Nostr group ID
//!
//! Here we also define the storage traits that are used to store and retrieve groups

use std::collections::BTreeSet;

use nostr::PublicKey;

pub mod error;
pub mod types;

use self::error::GroupError;
use self::types::*;
use crate::messages::types::Message;

/// Storage traits for the groups module
pub trait GroupStorage {
    /// Get all groups
    fn all_groups(&self) -> Result<Vec<Group>, GroupError>;

    /// Find a group by MLS group ID
    fn find_group_by_mls_group_id(&self, mls_group_id: &[u8]) -> Result<Option<Group>, GroupError>;

    /// Find a group by Nostr group ID
    fn find_group_by_nostr_group_id(
        &self,
        nostr_group_id: &str,
    ) -> Result<Option<Group>, GroupError>;

    /// Save a group
    fn save_group(&self, group: Group) -> Result<(), GroupError>;

    /// Get all messages for a group
    fn messages(&self, mls_group_id: &[u8]) -> Result<Vec<Message>, GroupError>;

    /// Get all admins for a group
    fn admins(&self, mls_group_id: &[u8]) -> Result<BTreeSet<PublicKey>, GroupError>;

    /// Get all relays for a group
    fn group_relays(&self, mls_group_id: &[u8]) -> Result<BTreeSet<GroupRelay>, GroupError>;

    /// Save a group relay
    fn save_group_relay(&self, group_relay: GroupRelay) -> Result<(), GroupError>;
}
