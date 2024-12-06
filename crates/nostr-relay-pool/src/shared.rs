// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nostr::prelude::IntoNostrSigner;
use nostr::NostrSigner;
use nostr_database::{IntoNostrDatabase, MemoryDatabase, NostrDatabase};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::{RelayFiltering, RelayFilteringMode};

#[derive(Debug, Error)]
pub enum SharedStateError {
    #[error("signer not configured")]
    SignerNotConfigured,
}

// TODO: add SharedStateBuilder?

#[derive(Debug, Clone)]
pub struct SharedState {
    pub(crate) database: Arc<dyn NostrDatabase>,
    signer: Arc<RwLock<Option<Arc<dyn NostrSigner>>>>,
    nip42_auto_authentication: Arc<AtomicBool>,
    pub(crate) filtering: RelayFiltering,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            database: MemoryDatabase::new().into_nostr_database(),
            signer: Arc::new(RwLock::new(None)),
            nip42_auto_authentication: Arc::new(AtomicBool::new(true)),
            filtering: RelayFiltering::default(),
        }
    }
}

impl SharedState {
    pub fn new(
        database: Arc<dyn NostrDatabase>,
        signer: Option<Arc<dyn NostrSigner>>,
        filtering_mode: RelayFilteringMode,
        nip42_auto_authentication: bool,
    ) -> Self {
        Self {
            database,
            signer: Arc::new(RwLock::new(signer)),
            nip42_auto_authentication: Arc::new(AtomicBool::new(nip42_auto_authentication)),
            filtering: RelayFiltering::new(filtering_mode),
        }
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
