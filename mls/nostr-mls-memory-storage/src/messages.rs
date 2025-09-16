//! Memory-based storage implementation of the NostrMlsStorageProvider trait for Nostr MLS messages

use nostr::EventId;
use nostr_mls_storage::messages::MessageStorage;
use nostr_mls_storage::messages::error::MessageError;
use nostr_mls_storage::messages::types::*;

use crate::NostrMlsMemoryStorage;

impl MessageStorage for NostrMlsMemoryStorage {
    fn save_message(&self, message: Message) -> Result<(), MessageError> {
        // Save in the messages cache
        let mut cache = self.messages_cache.write();
        cache.put(message.id, message.clone());

        // Save in the messages_by_group cache
        let mut group_cache = self.messages_by_group_cache.write();
        match group_cache.get_mut(&message.mls_group_id) {
            Some(group_messages) => {
                // TODO: time complexity here is O(n). We probably want to use another data struct here.

                // Find and update existing message or add new one
                match group_messages.iter().position(|m| m.id == message.id) {
                    Some(idx) => {
                        group_messages[idx] = message;
                    }
                    None => {
                        group_messages.push(message);
                    }
                }
            }
            // Not found, insert new
            None => {
                group_cache.put(message.mls_group_id.clone(), vec![message]);
            }
        }

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
