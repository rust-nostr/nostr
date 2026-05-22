use std::num::NonZeroUsize;

use lru::LruCache;
use nostr::prelude::*;
use nostr_database::error::Error;
use nostr_database::prelude::*;
use tokio::sync::RwLock;

const MAX_EVENTS: NonZeroUsize = NonZeroUsize::new(35_000).unwrap();

/// Memory Database (RAM)
#[derive(Debug)]
pub(crate) struct MemoryEventsTracker {
    tracker: RwLock<LruCache<EventId, ()>>,
}

impl Default for MemoryEventsTracker {
    fn default() -> Self {
        Self {
            tracker: RwLock::new(LruCache::new(MAX_EVENTS)),
        }
    }
}

impl NostrDatabase for MemoryEventsTracker {
    fn backend(&self) -> Backend {
        Backend::Custom(String::from("nostr-memory-events-tracker"))
    }

    fn features(&self) -> Features {
        Features {
            persistent: false,
            event_expiration: false,
            full_text_search: false,
            request_to_vanish: false,
        }
    }

    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, Error>> {
        Box::pin(async move {
            // Mark it as seen
            let mut seen_event_ids = self.tracker.write().await;
            seen_event_ids.put(event.id, ());

            Ok(SaveEventStatus::Success)
        })
    }

    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, Error>> {
        Box::pin(async move {
            let seen_event_ids = self.tracker.read().await;

            Ok(if seen_event_ids.contains(event_id) {
                DatabaseEventStatus::Saved
            } else {
                DatabaseEventStatus::NotExistent
            })
        })
    }

    fn event_by_id<'a>(
        &'a self,
        _event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, Error>> {
        Box::pin(async move { Ok(None) })
    }

    fn count(&self, _filter: Filter) -> BoxedFuture<'_, Result<usize, Error>> {
        Box::pin(async move { Ok(0) })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<'_, Result<Events, Error>> {
        Box::pin(async move { Ok(Events::new(&filter)) })
    }

    fn negentropy_items(
        &self,
        _filter: Filter,
    ) -> BoxedFuture<'_, Result<Vec<(EventId, Timestamp)>, Error>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn delete(&self, _filter: Filter) -> BoxedFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            Err(Error::unsupported(
                "delete is not supported for the in-memory events tracker",
            ))
        })
    }

    fn wipe(&self) -> BoxedFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let mut seen_event_ids = self.tracker.write().await;
            seen_event_ids.clear();
            Ok(())
        })
    }
}
