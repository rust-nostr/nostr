// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::pool::relay;
use uniffi::{Enum, Object};

use crate::protocol::event::EventId;
use crate::protocol::key::PublicKey;

#[derive(Enum)]
pub enum RelayFilteringMode {
    /// Only the matching values will be allowed
    Whitelist,
    /// All matching values will be discarded
    Blacklist,
}

impl From<RelayFilteringMode> for relay::RelayFilteringMode {
    fn from(value: RelayFilteringMode) -> Self {
        match value {
            RelayFilteringMode::Whitelist => Self::Whitelist,
            RelayFilteringMode::Blacklist => Self::Blacklist,
        }
    }
}

impl From<relay::RelayFilteringMode> for RelayFilteringMode {
    fn from(value: relay::RelayFilteringMode) -> Self {
        match value {
            relay::RelayFilteringMode::Whitelist => Self::Whitelist,
            relay::RelayFilteringMode::Blacklist => Self::Blacklist,
        }
    }
}

#[derive(Object)]
pub struct RelayFiltering {
    inner: relay::RelayFiltering,
}

impl Deref for RelayFiltering {
    type Target = relay::RelayFiltering;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<relay::RelayFiltering> for RelayFiltering {
    fn from(inner: relay::RelayFiltering) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RelayFiltering {
    /// Construct new filtering in whitelist mode
    #[uniffi::constructor]
    pub fn whitelist() -> Self {
        Self {
            inner: relay::RelayFiltering::whitelist(),
        }
    }

    /// Construct new filtering in blacklist mode
    #[uniffi::constructor]
    pub fn blacklist() -> Self {
        Self {
            inner: relay::RelayFiltering::blacklist(),
        }
    }

    /// Get filtering mode
    pub fn mode(&self) -> RelayFilteringMode {
        self.inner.mode().into()
    }

    /// Update filtering mode
    pub fn update_mode(&self, mode: RelayFilteringMode) {
        self.inner.update_mode(mode.into());
    }

    /// Add event IDs
    ///
    /// Note: IDs are ignored in whitelist mode!
    pub async fn add_ids(&self, ids: Vec<Arc<EventId>>) {
        self.inner.add_ids(ids.into_iter().map(|id| **id)).await
    }

    /// Remove event IDs
    ///
    /// Note: IDs are ignored in whitelist mode!
    pub async fn remove_ids(&self, ids: &[Arc<EventId>]) {
        self.inner
            .remove_ids(ids.iter().map(|id| id.as_ref().deref()))
            .await
    }

    /// Remove event ID
    ///
    /// Note: IDs are ignored in whitelist mode!
    pub async fn remove_id(&self, id: &EventId) {
        self.inner.remove_id(id.deref()).await
    }

    /// Check if has event ID
    pub async fn has_id(&self, id: &EventId) -> bool {
        self.inner.has_id(id.deref()).await
    }

    /// Add public keys
    pub async fn add_public_keys(&self, public_keys: Vec<Arc<PublicKey>>) {
        self.inner
            .add_public_keys(public_keys.into_iter().map(|p| **p))
            .await
    }

    /// Remove public keys
    pub async fn remove_public_keys(&self, ids: &[Arc<PublicKey>]) {
        self.inner
            .remove_public_keys(ids.iter().map(|p| p.as_ref().deref()))
            .await
    }

    /// Remove public key
    pub async fn remove_public_key(&self, public_key: &PublicKey) {
        self.inner.remove_public_key(public_key.deref()).await
    }

    /// Overwrite public keys set
    pub async fn overwrite_public_keys(&self, public_keys: Vec<Arc<PublicKey>>) {
        self.inner
            .overwrite_public_keys(public_keys.into_iter().map(|p| **p))
            .await
    }

    /// Check if has public key
    pub async fn has_public_key(&self, public_key: &PublicKey) -> bool {
        self.inner.has_public_key(public_key.deref()).await
    }

    /// Remove everything
    pub async fn clear(&self) {
        self.inner.clear().await
    }
}
