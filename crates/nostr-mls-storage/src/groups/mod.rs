pub mod error;
pub mod types;

use error::GroupError;
use nostr::PublicKey;
use types::*;

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

// TODO: MOVE TO nostr-mls
impl Group {
    /// Validates the members and admins of a group during creation
    ///
    /// # Arguments
    /// * `creator_pubkey` - The public key of the group creator
    /// * `member_pubkeys` - List of public keys for group members
    /// * `admin_pubkeys` - List of public keys for group admins
    ///
    /// # Returns
    /// * `Ok(true)` if validation passes
    /// * `Err(GroupError::InvalidParameters)` if validation fails
    ///
    /// # Validation Rules
    /// - Creator must be an admin but not included in member list
    /// - Creator must have a valid public key
    /// - All member public keys must be valid
    /// - All admin public keys must be valid
    /// - All admins must also be members (except creator)
    ///
    /// # Errors
    /// Returns `GroupError::InvalidParameters` with descriptive message if:
    /// - Creator is not an admin
    /// - Creator is in member list
    /// - Creator has invalid public key
    /// - Any member has invalid public key
    /// - Any admin has invalid public key
    /// - Any admin is not a member
    pub fn validate_group_members(
        creator_pubkey: &String,
        member_pubkeys: &[String],
        admin_pubkeys: &[String],
    ) -> Result<bool, GroupError> {
        // Creator must be an admin
        if !admin_pubkeys.contains(creator_pubkey) {
            return Err(GroupError::InvalidParameters(
                "Creator must be an admin".to_string(),
            ));
        }

        // Creator must not be included as a member
        if member_pubkeys.contains(creator_pubkey) {
            return Err(GroupError::InvalidParameters(
                "Creator must not be included as a member".to_string(),
            ));
        }

        // Creator must be valid pubkey
        if let Err(e) = PublicKey::parse(creator_pubkey) {
            return Err(GroupError::InvalidParameters(format!(
                "{} is not a valid creator pubkey: {}",
                creator_pubkey, e
            )));
        }

        // Check that members are valid pubkeys
        for pubkey in member_pubkeys.iter() {
            if let Err(e) = PublicKey::parse(pubkey) {
                return Err(GroupError::InvalidParameters(format!(
                    "{} is not a valid member pubkey: {}",
                    pubkey, e
                )));
            }
        }

        // Check that admins are valid pubkeys and are members
        for pubkey in admin_pubkeys.iter() {
            if let Err(e) = PublicKey::parse(pubkey) {
                return Err(GroupError::InvalidParameters(format!(
                    "{} is not a valid admin pubkey: {}",
                    pubkey, e
                )));
            }
            if !member_pubkeys.contains(pubkey) && creator_pubkey != pubkey {
                return Err(GroupError::InvalidParameters(
                    "Admin must be a member".to_string(),
                ));
            }
        }
        Ok(true)
    }
}
