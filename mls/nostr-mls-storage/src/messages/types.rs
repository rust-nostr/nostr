//! Types for the messages module

use std::fmt;
use std::str::FromStr;

use nostr::event::Kind;
use nostr::{EventId, PublicKey, Tags, Timestamp, UnsignedEvent};
use openmls::group::GroupId;
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
    pub mls_group_id: GroupId,
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
    /// The state of the message
    pub state: MessageState,
}

/// The state of the message
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MessageState {
    /// The message was created successfully and stored but we don't yet know if it was published to relays.
    Created,
    /// The message was successfully processed and stored in the database
    Processed,
    /// The message was deleted by the original sender - via a delete event
    Deleted,
}

impl fmt::Display for MessageState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl MessageState {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Created => "created",
            Self::Processed => "processed",
            Self::Deleted => "deleted",
        }
    }
}

impl FromStr for MessageState {
    type Err = MessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(Self::Created),
            "processed" => Ok(Self::Processed),
            "deleted" => Ok(Self::Deleted),
            _ => Err(MessageError::InvalidParameters(format!(
                "Invalid message state: {}",
                s
            ))),
        }
    }
}

impl Serialize for MessageState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for MessageState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// The Processing State of the message,
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessedMessageState {
    /// The processed message (and message) was created successfully and stored but we don't yet know if it was published to relays.
    /// This state only happens when you are sending a message. Since we can't decrypt messages from ourselves in MLS groups,
    /// once we see this message we mark it as processed but skip the rest of the processing.
    Created,
    /// The message was successfully processed and stored in the database
    Processed,
    /// The message was a commit message and we have already processed it. We can't decrypt messages from ourselves in MLS groups so we need to skip this processing.
    ProcessedCommit,
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
            Self::Created => "created",
            Self::Processed => "processed",
            Self::ProcessedCommit => "processed_commit",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for ProcessedMessageState {
    type Err = MessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(Self::Created),
            "processed" => Ok(Self::Processed),
            "processed_commit" => Ok(Self::ProcessedCommit),
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
    fn test_message_state_from_str() {
        assert_eq!(
            MessageState::from_str("created").unwrap(),
            MessageState::Created
        );
        assert_eq!(
            MessageState::from_str("processed").unwrap(),
            MessageState::Processed
        );
        assert_eq!(
            MessageState::from_str("deleted").unwrap(),
            MessageState::Deleted
        );

        let err = MessageState::from_str("invalid").unwrap_err();
        match err {
            MessageError::InvalidParameters(msg) => {
                assert!(msg.contains("Invalid message state: invalid"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[test]
    fn test_message_state_to_string() {
        assert_eq!(MessageState::Created.to_string(), "created");
        assert_eq!(MessageState::Processed.to_string(), "processed");
        assert_eq!(MessageState::Deleted.to_string(), "deleted");
    }

    #[test]
    fn test_message_state_serialization() {
        let created = MessageState::Created;
        let serialized = serde_json::to_string(&created).unwrap();
        assert_eq!(serialized, r#""created""#);

        let processed = MessageState::Processed;
        let serialized = serde_json::to_string(&processed).unwrap();
        assert_eq!(serialized, r#""processed""#);

        let deleted = MessageState::Deleted;
        let serialized = serde_json::to_string(&deleted).unwrap();
        assert_eq!(serialized, r#""deleted""#);
    }

    #[test]
    fn test_message_state_deserialization() {
        let created: MessageState = serde_json::from_str(r#""created""#).unwrap();
        assert_eq!(created, MessageState::Created);

        let processed: MessageState = serde_json::from_str(r#""processed""#).unwrap();
        assert_eq!(processed, MessageState::Processed);

        let deleted: MessageState = serde_json::from_str(r#""deleted""#).unwrap();
        assert_eq!(deleted, MessageState::Deleted);

        // Test invalid state
        let result = serde_json::from_str::<MessageState>(r#""invalid""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_serialization() {
        // Create a message to test serialization
        let pubkey =
            PublicKey::from_hex("8a9de562cbbed225b6ea0118dd3997a02df92c0bffd2224f71081a7450c3e549")
                .unwrap();
        let message = Message {
            id: EventId::all_zeros(),
            pubkey,
            kind: Kind::MlsGroupMessage,
            mls_group_id: GroupId::from_slice(&[1, 2, 3, 4]),
            created_at: Timestamp::now(),
            content: "Test message".to_string(),
            tags: Tags::new(),
            event: UnsignedEvent::new(
                pubkey,
                Timestamp::now(),
                Kind::MlsGroupMessage,
                Tags::new(),
                "Test message".to_string(),
            ),
            wrapper_event_id: EventId::all_zeros(),
            state: MessageState::Created,
        };

        let serialized = serde_json::to_value(&message).unwrap();
        assert_eq!(serialized["state"], json!("created"));
        assert_eq!(serialized["content"], json!("Test message"));
    }

    #[test]
    fn test_processed_message_state_from_str() {
        assert_eq!(
            ProcessedMessageState::from_str("created").unwrap(),
            ProcessedMessageState::Created
        );
        assert_eq!(
            ProcessedMessageState::from_str("processed").unwrap(),
            ProcessedMessageState::Processed
        );
        assert_eq!(
            ProcessedMessageState::from_str("processed_commit").unwrap(),
            ProcessedMessageState::ProcessedCommit
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
        assert_eq!(ProcessedMessageState::Created.to_string(), "created");
        assert_eq!(ProcessedMessageState::Processed.to_string(), "processed");
        assert_eq!(
            ProcessedMessageState::ProcessedCommit.to_string(),
            "processed_commit"
        );
        assert_eq!(ProcessedMessageState::Failed.to_string(), "failed");
    }

    #[test]
    fn test_processed_message_state_serialization() {
        let created = ProcessedMessageState::Created;
        let serialized = serde_json::to_string(&created).unwrap();
        assert_eq!(serialized, r#""created""#);

        let processed = ProcessedMessageState::Processed;
        let serialized = serde_json::to_string(&processed).unwrap();
        assert_eq!(serialized, r#""processed""#);

        let processed_commit = ProcessedMessageState::ProcessedCommit;
        let serialized = serde_json::to_string(&processed_commit).unwrap();
        assert_eq!(serialized, r#""processed_commit""#);

        let failed = ProcessedMessageState::Failed;
        let serialized = serde_json::to_string(&failed).unwrap();
        assert_eq!(serialized, r#""failed""#);
    }

    #[test]
    fn test_processed_message_state_deserialization() {
        let created: ProcessedMessageState = serde_json::from_str(r#""created""#).unwrap();
        assert_eq!(created, ProcessedMessageState::Created);

        let processed: ProcessedMessageState = serde_json::from_str(r#""processed""#).unwrap();
        assert_eq!(processed, ProcessedMessageState::Processed);

        let processed_commit: ProcessedMessageState =
            serde_json::from_str(r#""processed_commit""#).unwrap();
        assert_eq!(processed_commit, ProcessedMessageState::ProcessedCommit);

        let failed: ProcessedMessageState = serde_json::from_str(r#""failed""#).unwrap();
        assert_eq!(failed, ProcessedMessageState::Failed);

        // Test invalid state
        let result = serde_json::from_str::<ProcessedMessageState>(r#""invalid""#);
        assert!(result.is_err());
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
        assert_eq!(serialized["failure_reason"], json!(null));

        // Create a failed message with a reason
        let failed_message = ProcessedMessage {
            wrapper_event_id: EventId::all_zeros(),
            message_event_id: Some(EventId::all_zeros()),
            processed_at: Timestamp::now(),
            state: ProcessedMessageState::Failed,
            failure_reason: Some("Decryption failed".to_string()),
        };

        let serialized = serde_json::to_value(&failed_message).unwrap();
        assert_eq!(serialized["state"], json!("failed"));
        assert_eq!(serialized["failure_reason"], json!("Decryption failed"));
        assert!(serialized["message_event_id"].is_string());
    }

    #[test]
    fn test_processed_message_deserialization() {
        let json_str = r#"{
            "wrapper_event_id": "0000000000000000000000000000000000000000000000000000000000000000",
            "message_event_id": null,
            "processed_at": 1677721600,
            "state": "processed",
            "failure_reason": null
        }"#;

        let processed_message: ProcessedMessage = serde_json::from_str(json_str).unwrap();
        assert_eq!(processed_message.state, ProcessedMessageState::Processed);
        assert_eq!(processed_message.failure_reason, None);

        let json_str = r#"{
            "wrapper_event_id": "0000000000000000000000000000000000000000000000000000000000000000",
            "message_event_id": "0000000000000000000000000000000000000000000000000000000000000000",
            "processed_at": 1677721600,
            "state": "failed",
            "failure_reason": "Decryption failed"
        }"#;

        let failed_message: ProcessedMessage = serde_json::from_str(json_str).unwrap();
        assert_eq!(failed_message.state, ProcessedMessageState::Failed);
        assert_eq!(
            failed_message.failure_reason,
            Some("Decryption failed".to_string())
        );
    }
}
