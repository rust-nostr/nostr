//! Memory-based storage implementation of the NostrMlsStorageProvider trait for Nostr MLS welcomes

use nostr::EventId;
use nostr_mls_storage::welcomes::error::WelcomeError;
use nostr_mls_storage::welcomes::types::*;
use nostr_mls_storage::welcomes::WelcomeStorage;

use crate::NostrMlsMemoryStorage;

impl WelcomeStorage for NostrMlsMemoryStorage {
    fn save_welcome(&self, welcome: Welcome) -> Result<(), WelcomeError> {
        {
            let mut cache = self.welcomes_cache.write();
            cache.put(welcome.id, welcome.clone());
        }

        Ok(())
    }

    fn pending_welcomes(&self) -> Result<Vec<Welcome>, WelcomeError> {
        let cache = self.welcomes_cache.read();
        let welcomes: Vec<Welcome> = cache
            .iter()
            .map(|(_, v)| v.clone())
            .filter(|welcome| welcome.state == WelcomeState::Pending)
            .collect();

        Ok(welcomes)
    }

    fn find_welcome_by_event_id(&self, event_id: EventId) -> Result<Option<Welcome>, WelcomeError> {
        let cache = self.welcomes_cache.read();
        if let Some(welcome) = cache.peek(&event_id) {
            return Ok(Some(welcome.clone()));
        }

        Ok(None)
    }

    fn find_processed_welcome_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<Option<ProcessedWelcome>, WelcomeError> {
        let cache = self.processed_welcomes_cache.read();
        if let Some(processed_welcome) = cache.peek(&event_id) {
            return Ok(Some(processed_welcome.clone()));
        }

        Ok(None)
    }

    fn save_processed_welcome(
        &self,
        processed_welcome: ProcessedWelcome,
    ) -> Result<(), WelcomeError> {
        {
            let mut cache = self.processed_welcomes_cache.write();
            cache.put(
                processed_welcome.wrapper_event_id,
                processed_welcome.clone(),
            );
        }

        Ok(())
    }
}
