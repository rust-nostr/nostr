use nostr::EventId;

pub mod error;
pub mod parser;
pub mod types;

use self::error::MessageError;
use self::types::*;

pub trait MessageStorage {
    fn save_message(&self, message: Message) -> Result<Message, MessageError>;

    fn find_message_by_event_id(&self, event_id: EventId) -> Result<Message, MessageError>;

    fn find_processed_message_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedMessage, MessageError>;

    fn save_processed_message(
        &self,
        processed_message: ProcessedMessage,
    ) -> Result<ProcessedMessage, MessageError>;
}
