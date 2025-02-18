// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use lru::LruCache;
use nostr::prelude::IntoNostrSigner;
use nostr::{EventId, NostrSigner};
use nostr_database::{IntoNostrDatabase, MemoryDatabase, NostrDatabase};
use tokio::sync::RwLock;

use crate::policy::AdmitPolicy;
use crate::transport::websocket::{
    DefaultWebsocketTransport, IntoWebSocketTransport, WebSocketTransport,
};

// LruCache pre-allocate, so keep this at a reasonable value.
// A good value may be <= 128k, considering that stored values are the 64-bit hashes of the event IDs.
const MAX_VERIFICATION_CACHE_SIZE: usize = 128_000;

#[derive(Debug)]
pub enum SharedStateError {
    SignerNotConfigured,
    /// Admit policy already set
    AdmitPolicyAlreadySet,
    MutexPoisoned,
}

impl std::error::Error for SharedStateError {}

impl fmt::Display for SharedStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignerNotConfigured => write!(f, "signer not configured"),
            Self::AdmitPolicyAlreadySet => write!(f, "admission policy already set"),
            Self::MutexPoisoned => write!(f, "mutex poisoned"),
        }
    }
}

// TODO: reduce atomic operations
#[derive(Debug, Clone)]
pub struct SharedState {
    pub(crate) database: Arc<dyn NostrDatabase>,
    pub(crate) transport: Arc<dyn WebSocketTransport>,
    signer: Arc<RwLock<Option<Arc<dyn NostrSigner>>>>,
    nip42_auto_authentication: Arc<AtomicBool>,
    min_pow_difficulty: Arc<AtomicU8>,
    verification_cache: Arc<Mutex<LruCache<u64, ()>>>,
    pub(crate) admit_policy: OnceLock<Arc<dyn AdmitPolicy>>,
    // TODO: add a semaphore to limit number of concurrent websocket connections attempts?
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new(
            MemoryDatabase::new().into_nostr_database(),
            DefaultWebsocketTransport.into_transport(),
            None,
            None,
            true,
            0,
        )
    }
}

impl SharedState {
    pub fn new(
        database: Arc<dyn NostrDatabase>,
        transport: Arc<dyn WebSocketTransport>,
        signer: Option<Arc<dyn NostrSigner>>,
        admit_policy: Option<Arc<dyn AdmitPolicy>>,
        nip42_auto_authentication: bool,
        min_pow_difficulty: u8,
    ) -> Self {
        let max_verification_cache_size: NonZeroUsize =
            NonZeroUsize::new(MAX_VERIFICATION_CACHE_SIZE)
                .expect("MAX_VERIFICATION_CACHE_SIZE must be greater than 0");

        Self {
            database,
            transport,
            signer: Arc::new(RwLock::new(signer)),
            nip42_auto_authentication: Arc::new(AtomicBool::new(nip42_auto_authentication)),
            min_pow_difficulty: Arc::new(AtomicU8::new(min_pow_difficulty)),
            verification_cache: Arc::new(Mutex::new(LruCache::new(max_verification_cache_size))),
            admit_policy: match admit_policy {
                Some(policy) => OnceLock::from(policy),
                None => OnceLock::new(),
            },
        }
    }

    /// Set a custom transport
    pub fn custom_transport<T>(mut self, transport: T) -> Self
    where
        T: IntoWebSocketTransport,
    {
        self.transport = transport.into_transport();
        self
    }

    /// Set an admission policy
    #[inline]
    pub(crate) fn set_admit_policy<T>(&self, policy: T) -> Result<(), SharedStateError>
    where
        T: AdmitPolicy + 'static,
    {
        self.admit_policy
            .set(Arc::new(policy))
            .map_err(|_| SharedStateError::AdmitPolicyAlreadySet)
    }

    /// Check if auto authentication to relays is enabled
    #[inline]
    pub fn is_auto_authentication_enabled(&self) -> bool {
        self.nip42_auto_authentication.load(Ordering::SeqCst)
    }

    /// Auto authenticate to relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    pub fn automatic_authentication(&self, enable: bool) {
        self.nip42_auto_authentication
            .store(enable, Ordering::SeqCst);
    }

    /// Minimum POW difficulty for received events
    ///
    /// All received events must have a difficulty equal or greater than the set one.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn set_pow(&self, difficulty: u8) {
        self.min_pow_difficulty.store(difficulty, Ordering::SeqCst);
    }

    #[inline]
    pub(crate) fn minimum_pow_difficulty(&self) -> u8 {
        self.min_pow_difficulty.load(Ordering::SeqCst)
    }

    /// Get database
    #[inline]
    pub fn database(&self) -> &Arc<dyn NostrDatabase> {
        &self.database
    }

    /// Check if signer is configured
    pub async fn has_signer(&self) -> bool {
        let signer = self.signer.read().await;
        signer.is_some()
    }

    /// Get current nostr signer
    ///
    /// Rise error if it not set.
    pub async fn signer(&self) -> Result<Arc<dyn NostrSigner>, SharedStateError> {
        let signer = self.signer.read().await;
        signer.clone().ok_or(SharedStateError::SignerNotConfigured)
    }

    /// Set nostr signer
    pub async fn set_signer<T>(&self, signer: T)
    where
        T: IntoNostrSigner,
    {
        let mut s = self.signer.write().await;
        *s = Some(signer.into_nostr_signer());
    }

    /// Unset nostr signer
    pub async fn unset_signer(&self) {
        let mut s = self.signer.write().await;
        *s = None;
    }

    pub(crate) fn verified(&self, id: &EventId) -> Result<bool, SharedStateError> {
        let mut cache = self
            .verification_cache
            .lock()
            .map_err(|_| SharedStateError::MutexPoisoned)?;

        // Hash event ID
        let id: u64 = hash(&id);

        // Returns `Some(T)` if the key already exists
        Ok(cache.put(id, ()).is_some())
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
