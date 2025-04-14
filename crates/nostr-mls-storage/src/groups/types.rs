use nostr::{EventId, PublicKey, RelayUrl, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GroupType {
    /// A group with only two members
    DirectMessage,
    /// A group with more than two members
    Group,
}

impl From<String> for GroupType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "DirectMessage" => Self::DirectMessage,
            "Group" => Self::Group,
            _ => panic!("Invalid group type: {}", s),
        }
    }
}

impl From<GroupType> for String {
    fn from(group_type: GroupType) -> Self {
        match group_type {
            GroupType::DirectMessage => "DirectMessage".to_string(),
            GroupType::Group => "Group".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GroupState {
    Active,
    Inactive,
}

impl From<String> for GroupState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Active" => Self::Active,
            "Inactive" => Self::Inactive,
            _ => panic!("Invalid group state: {}", s),
        }
    }
}

impl From<GroupState> for String {
    fn from(state: GroupState) -> Self {
        match state {
            GroupState::Active => "Active".to_string(),
            GroupState::Inactive => "Inactive".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    /// This is the MLS group ID, this will serve as the PK in the DB and doesn't change
    pub mls_group_id: Vec<u8>,
    /// Hex encoded (same value as the NostrGroupDataExtension) this is the group_id used in Nostr events
    pub nostr_group_id: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub name: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub description: String,
    /// Hex encoded (same value as the NostrGroupDataExtension)
    pub admin_pubkeys: Vec<PublicKey>,
    /// Hex encoded Nostr event ID of the last message in the group
    pub last_message_id: Option<EventId>,
    /// Timestamp of the last message in the group
    pub last_message_at: Option<Timestamp>,
    /// Type of Nostr MLS group
    pub group_type: GroupType,
    /// Epoch of the group
    pub epoch: u64,
    /// The state of the group
    pub state: GroupState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupRelay {
    pub relay_url: RelayUrl,
    pub mls_group_id: Vec<u8>,
}
