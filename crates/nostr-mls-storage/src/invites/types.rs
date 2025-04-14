use nostr::{EventId, PublicKey, Timestamp, UnsignedEvent};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessedInvite {
    /// The event id of the processed invite
    pub wrapper_event_id: EventId,
    /// The event id of the rumor event (kind 444 invite message)
    pub invite_event_id: Option<EventId>,
    /// The timestamp of when the invite was processed
    pub processed_at: Timestamp,
    /// The state of the invite
    pub state: ProcessedInviteState,
    /// The reason the invite failed to be processed
    pub failure_reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Invite {
    /// The event id of the kind 444 invite
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
    /// Pubkey of the user that sent the invite
    pub inviter: PublicKey,
    /// Member count of the group
    pub member_count: u32,
    /// The state of the invite
    pub state: InviteState,
    /// The event id of the 1059 event that contained the invite
    pub wrapper_event_id: EventId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ProcessedInviteState {
    Processed,
    Failed,
}

impl From<String> for ProcessedInviteState {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "processed" => Self::Processed,
            "failed" => Self::Failed,
            _ => panic!("Invalid processed invite state: {}", s),
        }
    }
}

impl From<ProcessedInviteState> for String {
    fn from(state: ProcessedInviteState) -> Self {
        match state {
            ProcessedInviteState::Processed => "processed".to_string(),
            ProcessedInviteState::Failed => "failed".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum InviteState {
    Pending,
    Accepted,
    Declined,
    Ignored,
}

impl From<String> for InviteState {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => Self::Pending,
            "accepted" => Self::Accepted,
            "declined" => Self::Declined,
            "ignored" => Self::Ignored,
            _ => panic!("Invalid invite state: {}", s),
        }
    }
}

impl From<InviteState> for String {
    fn from(state: InviteState) -> Self {
        match state {
            InviteState::Pending => "pending".to_string(),
            InviteState::Accepted => "accepted".to_string(),
            InviteState::Declined => "declined".to_string(),
            InviteState::Ignored => "ignored".to_string(),
        }
    }
}
