//! Types for the welcomes module

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use nostr::{EventId, PublicKey, RelayUrl, Timestamp, UnsignedEvent};
use openmls::group::GroupId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::error::WelcomeError;

/// A processed welcome, this stores data about whether we have processed a welcome or not
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProcessedWelcome {
    /// The event id of the processed welcome
    pub wrapper_event_id: EventId,
    /// The event id of the rumor event (kind 444 welcome message)
    pub welcome_event_id: Option<EventId>,
    /// The timestamp of when the welcome was processed
    pub processed_at: Timestamp,
    /// The state of the welcome
    pub state: ProcessedWelcomeState,
    /// The reason the welcome failed to be processed
    pub failure_reason: Option<String>,
}

/// A welcome message
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Welcome {
    /// The event id of the kind 444 welcome
    pub id: EventId,
    /// The event that contains the welcome message
    pub event: UnsignedEvent,
    /// MLS group id
    pub mls_group_id: GroupId,
    /// Nostr group id (from NostrGroupDataExtension)
    pub nostr_group_id: [u8; 32],
    /// Group name (from NostrGroupDataExtension)
    pub group_name: String,
    /// Group description (from NostrGroupDataExtension)
    pub group_description: String,
    /// Group image hash (from NostrGroupDataExtension)
    pub group_image_hash: Option<Vec<u8>>,
    /// Group image key (from NostrGroupDataExtension)
    pub group_image_key: Option<Vec<u8>>,
    /// Group image nonce (from NostrGroupDataExtension)
    pub group_image_nonce: Option<Vec<u8>>,
    /// Group admin pubkeys (from NostrGroupDataExtension)
    pub group_admin_pubkeys: BTreeSet<PublicKey>,
    /// Group relays (from NostrGroupDataExtension)
    pub group_relays: BTreeSet<RelayUrl>,
    /// Pubkey of the user that sent the welcome
    pub welcomer: PublicKey,
    /// Member count of the group
    pub member_count: u32,
    /// The state of the welcome
    pub state: WelcomeState,
    /// The event id of the 1059 event that contained the welcome
    pub wrapper_event_id: EventId,
}

/// The processing state of a welcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessedWelcomeState {
    /// The welcome was successfully processed and stored in the database
    Processed,
    /// The welcome failed to be processed and stored in the database
    Failed,
}

impl fmt::Display for ProcessedWelcomeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ProcessedWelcomeState {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Processed => "processed",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for ProcessedWelcomeState {
    type Err = WelcomeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "processed" => Ok(Self::Processed),
            "failed" => Ok(Self::Failed),
            _ => Err(WelcomeError::InvalidParameters(format!(
                "Invalid processed welcome state: {}",
                s
            ))),
        }
    }
}

impl Serialize for ProcessedWelcomeState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ProcessedWelcomeState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// The state of a welcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WelcomeState {
    /// The welcome is pending
    Pending,
    /// The welcome was accepted
    Accepted,
    /// The welcome was declined
    Declined,
    /// The welcome was ignored
    Ignored,
}

impl fmt::Display for WelcomeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl WelcomeState {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Declined => "declined",
            Self::Ignored => "ignored",
        }
    }
}

impl FromStr for WelcomeState {
    type Err = WelcomeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "declined" => Ok(Self::Declined),
            "ignored" => Ok(Self::Ignored),
            _ => Err(WelcomeError::InvalidParameters(format!(
                "Invalid welcome state: {}",
                s
            ))),
        }
    }
}

impl Serialize for WelcomeState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for WelcomeState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_processed_welcome_state_from_str() {
        assert_eq!(
            ProcessedWelcomeState::from_str("processed").unwrap(),
            ProcessedWelcomeState::Processed
        );
        assert_eq!(
            ProcessedWelcomeState::from_str("failed").unwrap(),
            ProcessedWelcomeState::Failed
        );

        let err = ProcessedWelcomeState::from_str("invalid").unwrap_err();
        match err {
            WelcomeError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid processed welcome state: invalid"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[test]
    fn test_processed_welcome_state_to_string() {
        assert_eq!(ProcessedWelcomeState::Processed.to_string(), "processed");
        assert_eq!(ProcessedWelcomeState::Failed.to_string(), "failed");
    }

    #[test]
    fn test_processed_welcome_state_serialization() {
        let processed = ProcessedWelcomeState::Processed;
        let serialized = serde_json::to_string(&processed).unwrap();
        assert_eq!(serialized, r#""processed""#);

        let failed = ProcessedWelcomeState::Failed;
        let serialized = serde_json::to_string(&failed).unwrap();
        assert_eq!(serialized, r#""failed""#);
    }

    #[test]
    fn test_processed_welcome_state_deserialization() {
        let processed: ProcessedWelcomeState = serde_json::from_str(r#""processed""#).unwrap();
        assert_eq!(processed, ProcessedWelcomeState::Processed);

        let failed: ProcessedWelcomeState = serde_json::from_str(r#""failed""#).unwrap();
        assert_eq!(failed, ProcessedWelcomeState::Failed);
    }

    #[test]
    fn test_welcome_state_from_str() {
        assert_eq!(
            WelcomeState::from_str("pending").unwrap(),
            WelcomeState::Pending
        );
        assert_eq!(
            WelcomeState::from_str("accepted").unwrap(),
            WelcomeState::Accepted
        );
        assert_eq!(
            WelcomeState::from_str("declined").unwrap(),
            WelcomeState::Declined
        );
        assert_eq!(
            WelcomeState::from_str("ignored").unwrap(),
            WelcomeState::Ignored
        );

        let err = WelcomeState::from_str("invalid").unwrap_err();
        match err {
            WelcomeError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid welcome state: invalid"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[test]
    fn test_welcome_state_to_string() {
        assert_eq!(WelcomeState::Pending.to_string(), "pending");
        assert_eq!(WelcomeState::Accepted.to_string(), "accepted");
        assert_eq!(WelcomeState::Declined.to_string(), "declined");
        assert_eq!(WelcomeState::Ignored.to_string(), "ignored");
    }

    #[test]
    fn test_welcome_state_serialization() {
        let pending = WelcomeState::Pending;
        let serialized = serde_json::to_string(&pending).unwrap();
        assert_eq!(serialized, r#""pending""#);

        let accepted = WelcomeState::Accepted;
        let serialized = serde_json::to_string(&accepted).unwrap();
        assert_eq!(serialized, r#""accepted""#);

        let declined = WelcomeState::Declined;
        let serialized = serde_json::to_string(&declined).unwrap();
        assert_eq!(serialized, r#""declined""#);

        let ignored = WelcomeState::Ignored;
        let serialized = serde_json::to_string(&ignored).unwrap();
        assert_eq!(serialized, r#""ignored""#);
    }

    #[test]
    fn test_welcome_state_deserialization() {
        let pending: WelcomeState = serde_json::from_str(r#""pending""#).unwrap();
        assert_eq!(pending, WelcomeState::Pending);

        let accepted: WelcomeState = serde_json::from_str(r#""accepted""#).unwrap();
        assert_eq!(accepted, WelcomeState::Accepted);

        let declined: WelcomeState = serde_json::from_str(r#""declined""#).unwrap();
        assert_eq!(declined, WelcomeState::Declined);

        let ignored: WelcomeState = serde_json::from_str(r#""ignored""#).unwrap();
        assert_eq!(ignored, WelcomeState::Ignored);
    }

    #[test]
    fn test_processed_welcome_serialization() {
        // Create a processed welcome to test serialization
        let processed_welcome = ProcessedWelcome {
            wrapper_event_id: EventId::all_zeros(), // Using all_zeros for testing
            welcome_event_id: None,
            processed_at: Timestamp::now(),
            state: ProcessedWelcomeState::Processed,
            failure_reason: None,
        };

        let serialized = serde_json::to_value(&processed_welcome).unwrap();
        assert_eq!(serialized["state"], json!("processed"));
        assert_eq!(serialized["failure_reason"], json!(null));
    }
}
