//! Memory-based storage implementation of the NostrMlsStorageProvider trait for Nostr MLS messages

use nostr::EventId;
use nostr_mls_storage::messages::error::MessageError;
use nostr_mls_storage::messages::types::*;
use nostr_mls_storage::messages::MessageStorage;

use crate::NostrMlsMemoryStorage;

impl MessageStorage for NostrMlsMemoryStorage {
    fn save_message(&self, message: Message) -> Result<Message, MessageError> {
        {
            let mut cache = self.messages_cache.write();
            cache.put(message.id, message.clone());
        }

        Ok(message)
    }

    fn find_message_by_event_id(&self, event_id: EventId) -> Result<Message, MessageError> {
        let cache = self.messages_cache.read();
        if let Some(message) = cache.peek(&event_id) {
            return Ok(message.clone());
        }

        Err(MessageError::NotFound)
    }

    fn find_processed_message_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedMessage, MessageError> {
        let cache = self.processed_messages_cache.read();
        if let Some(processed_message) = cache.peek(&event_id) {
            return Ok(processed_message.clone());
        }

        Err(MessageError::NotFound)
    }

    fn save_processed_message(
        &self,
        processed_message: ProcessedMessage,
    ) -> Result<ProcessedMessage, MessageError> {
        {
            let mut cache = self.processed_messages_cache.write();
            cache.put(
                processed_message.wrapper_event_id,
                processed_message.clone(),
            );
        }

        Ok(processed_message)
    }
}
