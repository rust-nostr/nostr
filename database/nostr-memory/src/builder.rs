//! Memory Database Builder

use core::num::NonZeroUsize;

use crate::MemoryDatabase;

/// Memory Database Builder
#[derive(Debug)]
pub struct MemoryDatabaseBuilder {
    /// Max number of events to store in memory. If None, there is no limit.
    pub(crate) max_events: Option<NonZeroUsize>,
    /// Whether to process event deletion request (NIP-09) events.
    ///
    /// Defaults to `true`
    pub(crate) process_nip09: bool,
}

impl Default for MemoryDatabaseBuilder {
    fn default() -> Self {
        Self {
            max_events: None,
            process_nip09: true,
        }
    }
}

impl MemoryDatabaseBuilder {
    /// Set a maximum number of events to store in memory.
    ///
    /// When the limit is reached, the oldest events will be removed to make space for new ones.
    #[inline]
    pub fn max_events(mut self, max_events: NonZeroUsize) -> Self {
        self.max_events = Some(max_events);
        self
    }

    /// Whether to process event deletion request (NIP-09) events.
    ///
    /// Defaults to `true`
    #[inline]
    pub fn process_nip09(mut self, process_nip09: bool) -> Self {
        self.process_nip09 = process_nip09;
        self
    }

    /// Build the in-memory database.
    #[inline]
    pub fn build(self) -> MemoryDatabase {
        MemoryDatabase::from_builder(self)
    }
}
