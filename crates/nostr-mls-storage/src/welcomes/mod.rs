pub mod error;
pub mod types;

use error::WelcomeError;
use nostr::EventId;
use types::*;

pub trait WelcomeStorage {
    fn save_welcome(&self, welcome: Welcome) -> Result<Welcome, WelcomeError>;
    fn find_welcome_by_event_id(&self, event_id: EventId) -> Result<Welcome, WelcomeError>;
    fn pending_welcomes(&self) -> Result<Vec<Welcome>, WelcomeError>;

    fn save_processed_welcome(
        &self,
        processed_welcome: ProcessedWelcome,
    ) -> Result<ProcessedWelcome, WelcomeError>;
    fn find_processed_welcome_by_event_id(
        &self,
        event_id: EventId,
    ) -> Result<ProcessedWelcome, WelcomeError>;
}
