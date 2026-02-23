//! Memory Database Builder

use core::num::NonZeroUsize;

use crate::error::Error;
use crate::MemoryDatabase;

/// Memory Database Builder
#[derive(Debug, Default)]
pub struct MemoryDatabaseBuilder {
    /// Max number of events to store in memory. If None, there is no limit.
    pub max_events: Option<NonZeroUsize>,
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

    /// Build the in-memory database.
    #[inline]
    pub fn build(self) -> Result<MemoryDatabase, Error> {
        MemoryDatabase::from_builder(self)
    }
}
