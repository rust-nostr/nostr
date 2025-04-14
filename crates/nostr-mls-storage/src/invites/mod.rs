pub mod error;
pub mod types;

use error::InviteError;
use nostr::EventId;
use types::*;

pub trait InviteStorage {
    fn create_invite(&self, invite: Invite) -> Result<Invite, InviteError>;

    fn pending_invites(&self) -> Result<Vec<Invite>, InviteError>;

    fn find_invite_by_event_id(&self, event_id: EventId) -> Result<Invite, InviteError>;

    fn find_processed_invite_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedInvite, InviteError>;

    fn create_processed_invite_for_group_with_reason(
        &self,
        mls_group_id: &[u8],
        event_id: EventId,
        message_event_id: EventId,
        state: ProcessedInviteState,
        reason: String,
    ) -> Result<ProcessedInvite, InviteError>;
}
