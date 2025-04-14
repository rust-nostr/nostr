use crate::NostrMlsMemoryStorage;
use crate::CURRENT_VERSION;
use nostr::EventId;
use nostr_mls_storage::messages::error::MessageError;
use nostr_mls_storage::messages::types::*;
use nostr_mls_storage::messages::MessageStorage;

use openmls_traits::storage::StorageProvider;

impl<S: StorageProvider<CURRENT_VERSION>> MessageStorage for NostrMlsMemoryStorage<S> {
    fn create_message_for_group(
        &self,
        group_id: &[u8],
        message: Message,
    ) -> Result<Message, MessageError> {
        todo!()
    }

    fn find_message_by_event_id(&self, event_id: EventId) -> Result<Message, MessageError> {
        todo!()
    }

    fn find_processed_message_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedMessage, MessageError> {
        todo!()
    }

    fn create_processed_message_for_group_with_reason(
        &self,
        mls_group_id: &[u8],
        event_id: EventId,
        message_event_id: EventId,
        state: ProcessedMessageState,
        reason: String,
    ) -> Result<ProcessedMessage, MessageError> {
        todo!()
    }
}
