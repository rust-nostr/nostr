use std::sync::Arc;

use nostr::EventId;
use nostr_mls_storage::welcomes::error::WelcomeError;
use nostr_mls_storage::welcomes::types::*;
use nostr_mls_storage::welcomes::WelcomeStorage;

use crate::NostrMlsMemoryStorage;

impl WelcomeStorage for NostrMlsMemoryStorage {
    fn save_welcome(&self, welcome: Welcome) -> Result<Welcome, WelcomeError> {
        let welcome_arc = Arc::new(welcome.clone());

        {
            let mut cache = self.welcomes_cache.write();
            cache.put(welcome_arc.id, welcome_arc);
        }

        Ok(welcome)
    }

    fn pending_welcomes(&self) -> Result<Vec<Welcome>, WelcomeError> {
        let cache = self.welcomes_cache.read();
        let welcomes: Vec<Welcome> = cache
            .iter()
            .map(|(_, v)| (**v).clone())
            .filter(|welcome| welcome.state == WelcomeState::Pending)
            .collect();

        Ok(welcomes)
    }

    fn find_welcome_by_event_id(&self, event_id: EventId) -> Result<Welcome, WelcomeError> {
        let cache = self.welcomes_cache.read();
        if let Some(welcome_arc) = cache.peek(&event_id) {
            return Ok((**welcome_arc).clone());
        }

        Err(WelcomeError::NotFound)
    }

    fn find_processed_welcome_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedWelcome, WelcomeError> {
        let cache = self.processed_welcomes_cache.read();
        if let Some(processed_welcome_arc) = cache.peek(&event_id) {
            return Ok((**processed_welcome_arc).clone());
        }

        Err(WelcomeError::NotFound)
    }

    fn save_processed_welcome(
        &self,
        processed_welcome: ProcessedWelcome,
    ) -> Result<ProcessedWelcome, WelcomeError> {
        let processed_welcome_arc = Arc::new(processed_welcome.clone());

        {
            let mut cache = self.processed_welcomes_cache.write();
            cache.put(
                processed_welcome_arc.wrapper_event_id,
                processed_welcome_arc,
            );
        }

        Ok(processed_welcome)
    }
}
