//! Types for the messages module

use std::fmt;
use std::str::FromStr;

use nostr::event::Kind;
use nostr::{EventId, PublicKey, Tags, Timestamp, UnsignedEvent};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::error::MessageError;

/// A processed message, this stores data about whether we have processed a message or not
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProcessedMessage {
    /// The event id of the processed message
    pub wrapper_event_id: EventId,
    /// The event id of the rumor event (kind 445 group message)
    pub message_event_id: Option<EventId>,
    /// The timestamp of when the message was processed
    pub processed_at: Timestamp,
    /// The state of the message
    pub state: ProcessedMessageState,
    /// The reason the message failed to be processed
    pub failure_reason: Option<String>,
}

/// This is the processed rumor message that represents a message in a group
/// We store the deconstructed messages but also the UnsignedEvent.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Message {
    /// The event id of the message
    pub id: EventId,
    /// The pubkey of the author of the message
    pub pubkey: PublicKey,
    /// The kind of the message
    pub kind: Kind,
    /// The MLS group id of the message
    pub mls_group_id: Vec<u8>,
    /// The created at timestamp of the message
    pub created_at: Timestamp,
    /// The content of the message
    pub content: String,
    /// The tags of the message
    pub tags: Tags,
    /// The event that contains the message
    pub event: UnsignedEvent,
    /// The event id of the 1059 event that contained the message
    pub wrapper_event_id: EventId,
}

/// The Processing State of the message,
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessedMessageState {
    /// The message was successfully processed and stored in the database
    Processed,
    /// The message failed to be processed and stored in the database
    Failed,
}

impl fmt::Display for ProcessedMessageState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ProcessedMessageState {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Processed => "processed",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for ProcessedMessageState {
    type Err = MessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "processed" => Ok(Self::Processed),
            "failed" => Ok(Self::Failed),
            _ => Err(MessageError::InvalidParameters(format!(
                "Invalid processed message state: {}",
                s
            ))),
        }
    }
}

impl Serialize for ProcessedMessageState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ProcessedMessageState {
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
    fn test_processed_message_state_from_str() {
        assert_eq!(
            ProcessedMessageState::from_str("processed").unwrap(),
            ProcessedMessageState::Processed
        );
        assert_eq!(
            ProcessedMessageState::from_str("failed").unwrap(),
            ProcessedMessageState::Failed
        );

        let err = ProcessedMessageState::from_str("invalid").unwrap_err();
        match err {
            MessageError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid processed message state: invalid"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[test]
    fn test_processed_message_state_to_string() {
        assert_eq!(ProcessedMessageState::Processed.to_string(), "processed");
        assert_eq!(ProcessedMessageState::Failed.to_string(), "failed");
    }

    #[test]
    fn test_processed_message_state_serialization() {
        let processed = ProcessedMessageState::Processed;
        let serialized = serde_json::to_string(&processed).unwrap();
        assert_eq!(serialized, r#""processed""#);

        let failed = ProcessedMessageState::Failed;
        let serialized = serde_json::to_string(&failed).unwrap();
        assert_eq!(serialized, r#""failed""#);
    }

    #[test]
    fn test_processed_message_state_deserialization() {
        let processed: ProcessedMessageState = serde_json::from_str(r#""processed""#).unwrap();
        assert_eq!(processed, ProcessedMessageState::Processed);

        let failed: ProcessedMessageState = serde_json::from_str(r#""failed""#).unwrap();
        assert_eq!(failed, ProcessedMessageState::Failed);
    }

    #[test]
    fn test_processed_message_serialization() {
        // Create a processed message to test serialization
        let processed_message = ProcessedMessage {
            wrapper_event_id: EventId::all_zeros(),
            message_event_id: None,
            processed_at: Timestamp::now(),
            state: ProcessedMessageState::Processed,
            failure_reason: None,
        };

        let serialized = serde_json::to_value(&processed_message).unwrap();
        assert_eq!(serialized["state"], json!("processed"));
        assert_eq!(serialized["failure_reason"], json!(""));
    }
}
