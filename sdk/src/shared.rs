use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use nostr::EventId;
use nostr_database::NostrDatabase;
use tokio::sync::Mutex;

use crate::authenticator::Authenticator;
use crate::monitor::Monitor;
use crate::policy::AdmitPolicy;
use crate::transport::websocket::WebSocketTransport;

// LruCache pre-allocate, so keep this at a reasonable value.
// A good value may be <= 128k, considering that stored values are the 64-bit hashes of the event IDs.
const MAX_VERIFICATION_CACHE_SIZE: usize = 128_000;

#[derive(Debug, Clone)]
pub(crate) struct SharedState {
    pub(crate) database: Arc<dyn NostrDatabase>,
    pub(crate) transport: Arc<dyn WebSocketTransport>,
    verification_cache: Arc<Mutex<LruCache<u64, ()>>>,
    pub(crate) admit_policy: Option<Arc<dyn AdmitPolicy>>,
    pub(crate) authenticator: Option<Arc<dyn Authenticator>>,
    pub(crate) monitor: Option<Monitor>,
}

impl SharedState {
    pub(crate) fn new(
        database: Arc<dyn NostrDatabase>,
        transport: Arc<dyn WebSocketTransport>,
        admit_policy: Option<Arc<dyn AdmitPolicy>>,
        authenticator: Option<Arc<dyn Authenticator>>,
        monitor: Option<Monitor>,
    ) -> Self {
        let max_verification_cache_size: NonZeroUsize =
            NonZeroUsize::new(MAX_VERIFICATION_CACHE_SIZE)
                .expect("MAX_VERIFICATION_CACHE_SIZE must be greater than 0");

        Self {
            database,
            transport,
            verification_cache: Arc::new(Mutex::new(LruCache::new(max_verification_cache_size))),
            admit_policy,
            authenticator,
            monitor,
        }
    }

    #[inline]
    pub(crate) fn database(&self) -> &Arc<dyn NostrDatabase> {
        &self.database
    }

    #[inline]
    pub(crate) fn is_authenticator_available(&self) -> bool {
        self.authenticator.is_some()
    }

    pub(crate) async fn verified(&self, id: &EventId) -> bool {
        let mut cache = self.verification_cache.lock().await;

        // Hash event ID
        let id: u64 = hash(&id);

        // Returns `Some(T)` if the key already exists
        cache.put(id, ()).is_some()
    }
}

fn hash<T>(val: &T) -> u64
where
    T: Hash,
{
    let mut hasher: DefaultHasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}
