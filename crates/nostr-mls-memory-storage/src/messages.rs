//! Memory-based storage implementation of the NostrMlsStorageProvider trait for Nostr MLS messages

use nostr::EventId;
use nostr_mls_storage::messages::error::MessageError;
use nostr_mls_storage::messages::types::*;
use nostr_mls_storage::messages::MessageStorage;

use crate::NostrMlsMemoryStorage;

impl MessageStorage for NostrMlsMemoryStorage {
    fn save_message(&self, message: Message) -> Result<(), MessageError> {
        let mut cache = self.messages_cache.write();
        cache.put(message.id, message);
        Ok(())
    }

    fn find_message_by_event_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<Message>, MessageError> {
        let cache = self.messages_cache.read();
        Ok(cache.peek(event_id).cloned())
    }

    fn find_processed_message_by_event_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<ProcessedMessage>, MessageError> {
        let cache = self.processed_messages_cache.read();
        Ok(cache.peek(event_id).cloned())
    }

    fn save_processed_message(
        &self,
        processed_message: ProcessedMessage,
    ) -> Result<(), MessageError> {
        let mut cache = self.processed_messages_cache.write();
        cache.put(processed_message.wrapper_event_id, processed_message);

        Ok(())
    }
}
