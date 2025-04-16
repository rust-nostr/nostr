use nostr::{EventId, PublicKey, Timestamp, UnsignedEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub failure_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Welcome {
    /// The event id of the kind 444 welcome
    pub id: EventId,
    /// The event that contains the welcome message
    pub event: UnsignedEvent,
    /// MLS group id
    pub mls_group_id: Vec<u8>,
    /// Nostr group id (from NostrGroupDataExtension)
    pub nostr_group_id: String,
    /// Group name (from NostrGroupDataExtension)
    pub group_name: String,
    /// Group description (from NostrGroupDataExtension)
    pub group_description: String,
    /// Group admin pubkeys (from NostrGroupDataExtension)
    pub group_admin_pubkeys: Vec<String>,
    /// Group relays (from NostrGroupDataExtension)
    pub group_relays: Vec<String>,
    /// Pubkey of the user that sent the welcome
    pub welcomer: PublicKey,
    /// Member count of the group
    pub member_count: u32,
    /// The state of the welcome
    pub state: WelcomeState,
    /// The event id of the 1059 event that contained the welcome
    pub wrapper_event_id: EventId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ProcessedWelcomeState {
    Processed,
    Failed,
}

impl From<String> for ProcessedWelcomeState {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "processed" => Self::Processed,
            "failed" => Self::Failed,
            _ => panic!("Invalid processed welcome state: {}", s),
        }
    }
}

impl From<ProcessedWelcomeState> for String {
    fn from(state: ProcessedWelcomeState) -> Self {
        match state {
            ProcessedWelcomeState::Processed => "processed".to_string(),
            ProcessedWelcomeState::Failed => "failed".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum WelcomeState {
    Pending,
    Accepted,
    Declined,
    Ignored,
}

impl From<String> for WelcomeState {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => Self::Pending,
            "accepted" => Self::Accepted,
            "declined" => Self::Declined,
            "ignored" => Self::Ignored,
            _ => panic!("Invalid welcome state: {}", s),
        }
    }
}

impl From<WelcomeState> for String {
    fn from(state: WelcomeState) -> Self {
        match state {
            WelcomeState::Pending => "pending".to_string(),
            WelcomeState::Accepted => "accepted".to_string(),
            WelcomeState::Declined => "declined".to_string(),
            WelcomeState::Ignored => "ignored".to_string(),
        }
    }
}
