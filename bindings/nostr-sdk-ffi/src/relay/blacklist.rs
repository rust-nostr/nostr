// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::{EventId, PublicKey};
use nostr_sdk::block_on;
use nostr_sdk::pool::relay;
use uniffi::Object;

#[derive(Object)]
pub struct RelayBlacklist {
    inner: relay::RelayBlacklist,
}

impl Deref for RelayBlacklist {
    type Target = relay::RelayBlacklist;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<relay::RelayBlacklist> for RelayBlacklist {
    fn from(inner: relay::RelayBlacklist) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl RelayBlacklist {
    #[uniffi::constructor(default(ids = [], public_keys = []))]
    pub fn new(ids: Vec<Arc<EventId>>, public_keys: Vec<Arc<PublicKey>>) -> Self {
        Self {
            inner: relay::RelayBlacklist::new(
                ids.into_iter().map(|id| **id),
                public_keys.into_iter().map(|p| **p),
            ),
        }
    }

    /// construct new empty blacklist
    #[uniffi::constructor]
    pub fn empty() -> Self {
        Self {
            inner: relay::RelayBlacklist::empty(),
        }
    }

    /// Add event IDs to blacklist
    pub fn add_ids(&self, ids: Vec<Arc<EventId>>) {
        block_on(async { self.inner.add_ids(ids.into_iter().map(|id| **id)).await })
    }

    /// Remove event IDs from blacklist
    pub fn remove_ids(&self, ids: &[Arc<EventId>]) {
        block_on(async {
            self.inner
                .remove_ids(ids.iter().map(|id| id.as_ref().deref()))
                .await
        })
    }

    /// Remove event ID from blacklist
    pub fn remove_id(&self, id: &EventId) {
        block_on(async { self.inner.remove_id(id.deref()).await })
    }

    /// Check if blacklist contains event ID
    pub fn has_id(&self, id: &EventId) -> bool {
        block_on(async { self.inner.has_id(id.deref()).await })
    }

    /// Add public keys to blacklist
    pub fn add_public_keys(&self, public_keys: Vec<Arc<PublicKey>>) {
        block_on(async {
            self.inner
                .add_public_keys(public_keys.into_iter().map(|p| **p))
                .await
        })
    }

    /// Remove event IDs from blacklist
    pub fn remove_public_keys(&self, ids: &[Arc<PublicKey>]) {
        block_on(async {
            self.inner
                .remove_public_keys(ids.iter().map(|p| p.as_ref().deref()))
                .await
        })
    }

    /// Remove public key from blacklist
    pub fn remove_public_key(&self, public_key: &PublicKey) {
        block_on(async { self.inner.remove_public_key(public_key.deref()).await })
    }

    /// Check if blacklist contains public key
    pub fn has_public_key(&self, public_key: &PublicKey) -> bool {
        block_on(async { self.inner.has_public_key(public_key.deref()).await })
    }

    /// Remove everything
    pub fn clear(&self) {
        block_on(async { self.inner.clear().await })
    }
}
