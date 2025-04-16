//! Types for the groups module

use std::fmt;
use std::str::FromStr;

use nostr::{EventId, PublicKey, RelayUrl, Timestamp};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::error::GroupError;

/// The type of Nostr MLS group
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GroupType {
    /// A group with only two members
    DirectMessage,
    /// A group with more than two members
    Group,
}

impl fmt::Display for GroupType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl GroupType {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::DirectMessage => "direct_message",
            Self::Group => "group",
        }
    }
}

impl FromStr for GroupType {
    type Err = GroupError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "direct_message" => Ok(Self::DirectMessage),
            "group" => Ok(Self::Group),
            _ => Err(GroupError::InvalidParameters(format!(
                "Invalid group type: {}",
                s
            ))),
        }
    }
}

impl Serialize for GroupType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for GroupType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// The state of the group, this matches the MLS group state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GroupState {
    /// The group is active
    Active,
    /// The group is inactive
    Inactive,
}

impl fmt::Display for GroupState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl GroupState {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

impl FromStr for GroupState {
    type Err = GroupError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            _ => Err(GroupError::InvalidParameters(format!(
                "Invalid group state: {}",
                s
            ))),
        }
    }
}

impl Serialize for GroupState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for GroupState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// A Nostr MLS group
///
/// Stores metadata about the group
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A Nostr MLS group relay
///
/// Stores a relay URL and the MLS group ID it belongs to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRelay {
    /// The relay URL
    pub relay_url: RelayUrl,
    /// The MLS group ID
    pub mls_group_id: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_group_type_from_str() {
        assert_eq!(
            GroupType::from_str("direct_message").unwrap(),
            GroupType::DirectMessage
        );
        assert_eq!(GroupType::from_str("group").unwrap(), GroupType::Group);

        let err = GroupType::from_str("invalid").unwrap_err();
        match err {
            GroupError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid group type: invalid"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[test]
    fn test_group_type_to_string() {
        assert_eq!(GroupType::DirectMessage.to_string(), "direct_message");
        assert_eq!(GroupType::Group.to_string(), "group");
    }

    #[test]
    fn test_group_type_serialization() {
        let direct_message = GroupType::DirectMessage;
        let serialized = serde_json::to_string(&direct_message).unwrap();
        assert_eq!(serialized, r#""direct_message""#);

        let group = GroupType::Group;
        let serialized = serde_json::to_string(&group).unwrap();
        assert_eq!(serialized, r#""group""#);
    }

    #[test]
    fn test_group_type_deserialization() {
        let direct_message: GroupType = serde_json::from_str(r#""direct_message""#).unwrap();
        assert_eq!(direct_message, GroupType::DirectMessage);

        let group: GroupType = serde_json::from_str(r#""group""#).unwrap();
        assert_eq!(group, GroupType::Group);

        // Test snake_case works
        let direct_message: GroupType = serde_json::from_str(r#""direct_message""#).unwrap();
        assert_eq!(direct_message, GroupType::DirectMessage);
    }

    #[test]
    fn test_group_state_from_str() {
        assert_eq!(GroupState::from_str("active").unwrap(), GroupState::Active);
        assert_eq!(
            GroupState::from_str("inactive").unwrap(),
            GroupState::Inactive
        );

        let err = GroupState::from_str("invalid").unwrap_err();
        match err {
            GroupError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid group state: invalid"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[test]
    fn test_group_state_to_string() {
        assert_eq!(GroupState::Active.to_string(), "active");
        assert_eq!(GroupState::Inactive.to_string(), "inactive");
    }

    #[test]
    fn test_group_state_serialization() {
        let active = GroupState::Active;
        let serialized = serde_json::to_string(&active).unwrap();
        assert_eq!(serialized, r#""active""#);

        let inactive = GroupState::Inactive;
        let serialized = serde_json::to_string(&inactive).unwrap();
        assert_eq!(serialized, r#""inactive""#);
    }

    #[test]
    fn test_group_state_deserialization() {
        let active: GroupState = serde_json::from_str(r#""active""#).unwrap();
        assert_eq!(active, GroupState::Active);

        let inactive: GroupState = serde_json::from_str(r#""inactive""#).unwrap();
        assert_eq!(inactive, GroupState::Inactive);
    }

    #[test]
    fn test_group_serialization() {
        // Simple test to ensure Group can be serialized
        let group = Group {
            mls_group_id: vec![1, 2, 3],
            nostr_group_id: "test_id".to_string(),
            name: "Test Group".to_string(),
            description: "Test Description".to_string(),
            admin_pubkeys: Vec::new(),
            last_message_id: None,
            last_message_at: None,
            group_type: GroupType::Group,
            epoch: 0,
            state: GroupState::Active,
        };

        let serialized = serde_json::to_value(&group).unwrap();
        assert_eq!(serialized["mls_group_id"], json!([1, 2, 3]));
        assert_eq!(serialized["nostr_group_id"], json!("test_id"));
        assert_eq!(serialized["name"], json!("Test Group"));
        assert_eq!(serialized["description"], json!("Test Description"));
        assert_eq!(serialized["group_type"], json!("group"));
        assert_eq!(serialized["state"], json!("active"));
    }
}
