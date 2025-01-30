// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;

use nostr::prelude::IntoNostrSigner;
use nostr::NostrSigner;
use nostr_database::{IntoNostrDatabase, MemoryDatabase, NostrDatabase};
use tokio::sync::RwLock;

use crate::transport::websocket::{
    DefaultWebsocketTransport, IntoWebSocketTransport, WebSocketTransport,
};
use crate::{RelayFiltering, RelayFilteringMode};

#[derive(Debug)]
pub enum SharedStateError {
    SignerNotConfigured,
}

impl std::error::Error for SharedStateError {}

impl fmt::Display for SharedStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignerNotConfigured => write!(f, "signer not configured"),
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
    pub(crate) filtering: RelayFiltering,
    // TODO: add a semaphore to limit number of concurrent websocket connections attempts?
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            database: MemoryDatabase::new().into_nostr_database(),
            transport: DefaultWebsocketTransport.into_transport(),
            signer: Arc::new(RwLock::new(None)),
            nip42_auto_authentication: Arc::new(AtomicBool::new(true)),
            min_pow_difficulty: Arc::new(AtomicU8::new(0)),
            filtering: RelayFiltering::default(),
        }
    }
}

impl SharedState {
    pub fn new(
        database: Arc<dyn NostrDatabase>,
        transport: Arc<dyn WebSocketTransport>,
        signer: Option<Arc<dyn NostrSigner>>,
        filtering_mode: RelayFilteringMode,
        nip42_auto_authentication: bool,
        min_pow_difficulty: u8,
    ) -> Self {
        Self {
            database,
            transport,
            signer: Arc::new(RwLock::new(signer)),
            nip42_auto_authentication: Arc::new(AtomicBool::new(nip42_auto_authentication)),
            filtering: RelayFiltering::new(filtering_mode),
            min_pow_difficulty: Arc::new(AtomicU8::new(min_pow_difficulty)),
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

    /// Get relay filtering
    #[inline]
    pub fn filtering(&self) -> &RelayFiltering {
        &self.filtering
    }
}
