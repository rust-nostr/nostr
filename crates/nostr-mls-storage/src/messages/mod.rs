pub mod error;
pub mod parser;
pub mod types;

use error::MessageError;
use nostr::EventId;
use types::*;

pub trait MessageStorage {
    fn create_message_for_group(
        &self,
        mls_group_id: &[u8],
        message: Message,
    ) -> Result<Message, MessageError>;

    fn find_message_by_event_id(&self, event_id: EventId) -> Result<Message, MessageError>;

    fn find_processed_message_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedMessage, MessageError>;

    fn create_processed_message_for_group_with_reason(
        &self,
        mls_group_id: &[u8],
        event_id: EventId,
        message_event_id: EventId,
        state: ProcessedMessageState,
        reason: String,
    ) -> Result<ProcessedMessage, MessageError>;
}
