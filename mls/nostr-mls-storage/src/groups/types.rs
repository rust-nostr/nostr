//! Types for the groups module

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use nostr::{EventId, PublicKey, RelayUrl, Timestamp};
use openmls::group::GroupId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::error::GroupError;

/// The state of the group, this matches the MLS group state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GroupState {
    /// The group is active
    Active,
    /// The group is inactive, this is used for groups that users have left or for welcome messages that have been declined
    Inactive,
    /// The group is pending, this is used for groups that users are invited to but haven't joined yet
    Pending,
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
            Self::Pending => "pending",
        }
    }
}

impl FromStr for GroupState {
    type Err = GroupError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "pending" => Ok(Self::Pending),
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Group {
    /// This is the MLS group ID, this will serve as the PK in the DB and doesn't change
    pub mls_group_id: GroupId,
    /// This is the group_id used in published Nostr events, it can change over time
    pub nostr_group_id: [u8; 32],
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub name: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub description: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub image_url: Option<String>,
    /// Secret key of the image
    pub image_key: Option<Vec<u8>>,
    /// Hex encoded (same value as the NostrGroupDataExtension)
    pub admin_pubkeys: BTreeSet<PublicKey>,
    /// Hex encoded Nostr event ID of the last message in the group
    pub last_message_id: Option<EventId>,
    /// Timestamp of the last message in the group
    pub last_message_at: Option<Timestamp>,
    /// Epoch of the group
    pub epoch: u64,
    /// The state of the group
    pub state: GroupState,
}

/// A Nostr MLS group relay
///
/// Stores a relay URL and the MLS group ID it belongs to
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GroupRelay {
    /// The relay URL
    pub relay_url: RelayUrl,
    /// The MLS group ID
    pub mls_group_id: GroupId,
}

/// Exporter secrets for each epoch of a group
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GroupExporterSecret {
    /// The MLS group ID
    pub mls_group_id: GroupId,
    /// The epoch
    pub epoch: u64,
    /// The secret
    pub secret: [u8; 32],
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

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
            mls_group_id: GroupId::from_slice(&[1, 2, 3]),
            nostr_group_id: [0u8; 32],
            name: "Test Group".to_string(),
            description: "Test Description".to_string(),
            image_url: None,
            image_key: None,
            admin_pubkeys: BTreeSet::new(),
            last_message_id: None,
            last_message_at: None,
            epoch: 0,
            state: GroupState::Active,
        };

        let serialized = serde_json::to_value(&group).unwrap();
        assert_eq!(serialized["mls_group_id"]["value"]["vec"], json!([1, 2, 3]));
        assert_eq!(
            serialized["nostr_group_id"],
            json!([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ])
        );
        assert_eq!(serialized["name"], json!("Test Group"));
        assert_eq!(serialized["description"], json!("Test Description"));
        assert_eq!(serialized["state"], json!("active"));
    }

    #[test]
    fn test_group_exporter_secret_serialization() {
        let secret = GroupExporterSecret {
            mls_group_id: GroupId::from_slice(&[1, 2, 3]),
            epoch: 42,
            secret: [0u8; 32],
        };

        let serialized = serde_json::to_value(&secret).unwrap();
        assert_eq!(serialized["mls_group_id"]["value"]["vec"], json!([1, 2, 3]));
        assert_eq!(serialized["epoch"], json!(42));
        assert_eq!(
            serialized["secret"],
            json!([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ])
        );

        // Test deserialization
        let deserialized: GroupExporterSecret = serde_json::from_value(serialized).unwrap();
        assert_eq!(deserialized.epoch, 42);
        assert_eq!(deserialized.secret, [0u8; 32]);
    }

    #[test]
    fn test_group_relay_serialization() {
        let relay = GroupRelay {
            relay_url: RelayUrl::from_str("wss://relay.example.com").unwrap(),
            mls_group_id: GroupId::from_slice(&[1, 2, 3]),
        };

        let serialized = serde_json::to_value(&relay).unwrap();
        assert_eq!(serialized["relay_url"], json!("wss://relay.example.com"));
        assert_eq!(serialized["mls_group_id"]["value"]["vec"], json!([1, 2, 3]));

        // Test deserialization
        let deserialized: GroupRelay = serde_json::from_value(serialized).unwrap();
        assert_eq!(
            deserialized.relay_url.to_string(),
            "wss://relay.example.com"
        );
    }
}
