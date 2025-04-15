use std::sync::Arc;

use nostr::EventId;
use nostr_mls_storage::messages::error::MessageError;
use nostr_mls_storage::messages::types::*;
use nostr_mls_storage::messages::MessageStorage;

use crate::NostrMlsMemoryStorage;

impl MessageStorage for NostrMlsMemoryStorage {
    fn save_message(&self, message: Message) -> Result<Message, MessageError> {
        let message_arc = Arc::new(message.clone());

        if let Ok(mut cache) = self.messages_cache.write() {
            cache.put(message_arc.id, Arc::clone(&message_arc));
        } else {
            return Err(MessageError::DatabaseError(
                "Failed to acquire write lock on messages cache".to_string(),
            ));
        }

        Ok(message)
    }

    fn find_message_by_event_id(&self, event_id: EventId) -> Result<Message, MessageError> {
        if let Ok(cache) = self.messages_cache.read() {
            if let Some(message_arc) = cache.peek(&event_id) {
                return Ok((**message_arc).clone());
            }
        } else {
            return Err(MessageError::DatabaseError(
                "Failed to acquire read lock on messages cache".to_string(),
            ));
        }

        Err(MessageError::NotFound)
    }

    fn find_processed_message_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedMessage, MessageError> {
        if let Ok(cache) = self.processed_messages_cache.read() {
            if let Some(processed_message_arc) = cache.peek(&event_id) {
                return Ok((**processed_message_arc).clone());
            }
        } else {
            return Err(MessageError::DatabaseError(
                "Failed to acquire read lock on processed messages cache".to_string(),
            ));
        }

        Err(MessageError::NotFound)
    }

    fn save_processed_message(
        &self,
        processed_message: ProcessedMessage,
    ) -> Result<ProcessedMessage, MessageError> {
        let processed_message_arc = Arc::new(processed_message.clone());

        if let Ok(mut cache) = self.processed_messages_cache.write() {
            cache.put(
                processed_message_arc.wrapper_event_id,
                processed_message_arc,
            );
        } else {
            return Err(MessageError::DatabaseError(
                "Failed to acquire write lock on processed messages cache".to_string(),
            ));
        }

        Ok(processed_message)
    }
}
