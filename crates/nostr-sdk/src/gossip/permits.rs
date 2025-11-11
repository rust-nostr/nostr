use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use nostr::PublicKey;
use nostr_gossip::GossipListKind;
use tokio::sync::{AcquireError, Mutex, OwnedSemaphorePermit, Semaphore};

const CACHE_LIMIT: usize = 35_000;

type Key = (PublicKey, GossipListKind);

#[derive(Debug)]
pub(crate) struct GossipSyncPermits {
    permits: Mutex<LruCache<Key, Arc<Semaphore>>>,
}

impl Default for GossipSyncPermits {
    fn default() -> Self {
        let limit: NonZeroUsize = NonZeroUsize::new(CACHE_LIMIT).expect("CACHE_LIMIT must be > 0");

        Self {
            permits: Mutex::new(LruCache::new(limit)),
        }
    }
}

/// NOTE: don't acquire the Mutex lock in the same function as the semaphore permit,
/// or most likely will cause a deadlock!
impl GossipSyncPermits {
    /// Lock the mutex and get the cloned semaphore.
    async fn get_semaphore(&self, key: Key) -> Arc<Semaphore> {
        let mut permits = self.permits.lock().await;
        permits
            .get_or_insert(key, || Arc::new(Semaphore::new(1)))
            .clone()
    }

    /// Acquire a permit for a specific public key and gossip kind
    pub(crate) async fn acquire(
        &self,
        public_key: PublicKey,
        kind: GossipListKind,
    ) -> Result<OwnedSemaphorePermit, AcquireError> {
        let key: Key = (public_key, kind);

        // Get the semaphore
        let semaphore: Arc<Semaphore> = self.get_semaphore(key).await;

        // Acquire the permit
        semaphore.acquire_owned().await
    }
}
