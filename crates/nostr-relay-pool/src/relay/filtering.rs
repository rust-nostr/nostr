// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay filtering

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nostr::{EventId, PartialEvent, PublicKey};
use tokio::sync::RwLock;

pub(crate) enum CheckFiltering {
    Allow,
    EventIdBlacklisted(EventId),
    PublicKeyBlacklisted(PublicKey),
    PublicKeyNotInWhitelist(PublicKey),
}

/// Filtering mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayFilteringMode {
    /// Only the matching values will be allowed
    Whitelist,
    /// All matching values will be discarded
    #[default]
    Blacklist,
}

impl RelayFilteringMode {
    /// Check if is whitelist mode
    #[inline]
    pub fn is_whitelist(&self) -> bool {
        matches!(self, RelayFilteringMode::Whitelist)
    }

    /// Check if is blacklist mode
    #[inline]
    pub fn is_blacklist(&self) -> bool {
        matches!(self, RelayFilteringMode::Blacklist)
    }
}

#[derive(Debug, Default)]
struct AtomicRelayFilteringMode {
    /// Value
    ///
    /// * true -> whitelist
    /// * false -> blacklist
    value: AtomicBool,
}

impl AtomicRelayFilteringMode {
    #[inline]
    fn new(mode: RelayFilteringMode) -> Self {
        Self {
            value: AtomicBool::new(mode.is_whitelist()),
        }
    }

    fn load(&self) -> RelayFilteringMode {
        let val: bool = self.value.load(Ordering::SeqCst);
        if val {
            RelayFilteringMode::Whitelist
        } else {
            RelayFilteringMode::Blacklist
        }
    }

    fn update(&self, mode: RelayFilteringMode) {
        let val: bool = mode.is_whitelist();
        self.value.store(val, Ordering::SeqCst);
    }
}

#[derive(Debug, Default)]
struct InnerRelayFiltering {
    mode: AtomicRelayFilteringMode,
    ids: RwLock<HashSet<EventId>>,
    public_keys: RwLock<HashSet<PublicKey>>,
}

impl InnerRelayFiltering {
    #[inline]
    fn new(mode: RelayFilteringMode) -> Self {
        Self {
            mode: AtomicRelayFilteringMode::new(mode),
            ..Default::default()
        }
    }
}

/// Relay filtering
#[derive(Debug, Clone, Default)]
pub struct RelayFiltering {
    inner: Arc<InnerRelayFiltering>,
}

impl RelayFiltering {
    /// Construct new filtering
    pub fn new(mode: RelayFilteringMode) -> Self {
        Self {
            inner: Arc::new(InnerRelayFiltering::new(mode)),
        }
    }

    /// Construct new filtering in whitelist mode
    #[inline]
    pub fn whitelist() -> Self {
        Self::new(RelayFilteringMode::Whitelist)
    }

    /// Construct new filtering in blacklist mode
    #[inline]
    pub fn blacklist() -> Self {
        Self::new(RelayFilteringMode::Blacklist)
    }

    /// Get mode
    #[inline]
    pub fn mode(&self) -> RelayFilteringMode {
        self.inner.mode.load()
    }

    /// Update filtering mode
    #[inline]
    pub fn update_mode(&self, mode: RelayFilteringMode) {
        self.inner.mode.update(mode);
    }

    /// Add event IDs
    ///
    /// Note: IDs are ignored in whitelist mode!
    pub async fn add_ids<I>(&self, i: I)
    where
        I: IntoIterator<Item = EventId>,
    {
        let mut ids = self.inner.ids.write().await;
        ids.extend(i);
    }

    /// Remove event IDs
    ///
    /// Note: IDs are ignored in whitelist mode!
    pub async fn remove_ids<'a, I>(&self, iter: I)
    where
        I: IntoIterator<Item = &'a EventId>,
    {
        let mut ids = self.inner.ids.write().await;
        for id in iter.into_iter() {
            ids.remove(id);
        }
    }

    /// Remove event ID
    ///
    /// Note: IDs are ignored in whitelist mode!
    pub async fn remove_id(&self, id: &EventId) {
        let mut ids = self.inner.ids.write().await;
        ids.remove(id);
    }

    /// Check if has event ID
    pub async fn has_id(&self, id: &EventId) -> bool {
        let ids = self.inner.ids.read().await;
        ids.contains(id)
    }

    /// Add public keys
    pub async fn add_public_keys<I>(&self, iter: I)
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let mut public_keys = self.inner.public_keys.write().await;
        public_keys.extend(iter);
    }

    /// Remove public keys
    pub async fn remove_public_keys<'a, I>(&self, iter: I)
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut public_keys = self.inner.public_keys.write().await;
        for public_key in iter.into_iter() {
            public_keys.remove(public_key);
        }
    }

    /// Remove public key
    pub async fn remove_public_key(&self, public_key: &PublicKey) {
        let mut public_keys = self.inner.public_keys.write().await;
        public_keys.remove(public_key);
    }

    /// Overwrite public keys set
    pub async fn overwrite_public_keys<I>(&self, public_keys: I)
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let mut p = self.inner.public_keys.write().await;
        *p = public_keys.into_iter().collect();
    }

    /// Check if has public key
    pub async fn has_public_key(&self, public_key: &PublicKey) -> bool {
        let public_keys = self.inner.public_keys.read().await;
        public_keys.contains(public_key)
    }

    pub(crate) async fn check_partial_event(&self, partial_event: &PartialEvent) -> CheckFiltering {
        match self.inner.mode.load() {
            RelayFilteringMode::Whitelist => {
                if !self.has_public_key(&partial_event.pubkey).await {
                    return CheckFiltering::PublicKeyNotInWhitelist(partial_event.pubkey);
                }
            }
            RelayFilteringMode::Blacklist => {
                if self.has_id(&partial_event.id).await {
                    return CheckFiltering::EventIdBlacklisted(partial_event.id);
                }

                if self.has_public_key(&partial_event.pubkey).await {
                    return CheckFiltering::PublicKeyBlacklisted(partial_event.pubkey);
                }
            }
        };

        CheckFiltering::Allow
    }

    /// Remove everything
    pub async fn clear(&self) {
        let mut ids = self.inner.ids.write().await;
        ids.clear();

        let mut public_keys = self.inner.public_keys.write().await;
        public_keys.clear();
    }
}
