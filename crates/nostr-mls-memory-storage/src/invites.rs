use crate::NostrMlsMemoryStorage;
use crate::CURRENT_VERSION;
use nostr::EventId;
use nostr_mls_storage::invites::error::InviteError;
use nostr_mls_storage::invites::types::*;
use nostr_mls_storage::invites::InviteStorage;

use openmls_traits::storage::StorageProvider;

impl<S: StorageProvider<CURRENT_VERSION>> InviteStorage for NostrMlsMemoryStorage<S> {
    fn create_invite(&self, invite: Invite) -> Result<Invite, InviteError> {
        todo!()
    }

    fn pending_invites(&self) -> Result<Vec<Invite>, InviteError> {
        todo!()
    }

    fn find_invite_by_event_id(&self, event_id: EventId) -> Result<Invite, InviteError> {
        todo!()
    }

    fn find_processed_invite_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedInvite, InviteError> {
        todo!()
    }

    fn create_processed_invite_for_group_with_reason(
        &self,
        mls_group_id: &[u8],
        event_id: EventId,
        message_event_id: EventId,
        state: ProcessedInviteState,
        reason: String,
    ) -> Result<ProcessedInvite, InviteError> {
        todo!()
    }
}
