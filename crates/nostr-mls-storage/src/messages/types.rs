use nostr::event::Kind;
use nostr::{EventId, PublicKey, Tags, Timestamp, UnsignedEvent};
use serde::{Deserialize, Serialize};

use super::parser::SerializableToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub failure_reason: String,
}

/// This is the processed rumor message that represents a message in a group
/// We store the deconstructed messages but also the UnsignedEvent.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// The tokenized content of the message
    pub tokens: Vec<SerializableToken>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ProcessedMessageState {
    Processed,
    Failed,
}

impl From<String> for ProcessedMessageState {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "processed" => Self::Processed,
            "failed" => Self::Failed,
            _ => panic!("Invalid processed message state: {}", s),
        }
    }
}

impl From<ProcessedMessageState> for String {
    fn from(state: ProcessedMessageState) -> Self {
        match state {
            ProcessedMessageState::Processed => "processed".to_string(),
            ProcessedMessageState::Failed => "failed".to_string(),
        }
    }
}
