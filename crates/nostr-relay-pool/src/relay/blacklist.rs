// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay blacklist

use std::collections::HashSet;
use std::sync::Arc;

use nostr::{EventId, PublicKey};
use tokio::sync::RwLock;

/// Blacklist
#[derive(Debug, Clone, Default)]
pub struct RelayBlacklist {
    ids: Arc<RwLock<HashSet<EventId>>>,
    public_keys: Arc<RwLock<HashSet<PublicKey>>>,
    //words: Arc<RwLock<HashSet<String>>>,
}

impl RelayBlacklist {
    /// Construct blacklist
    pub fn new<I, P>(ids: I, public_keys: P) -> Self
    where
        I: IntoIterator<Item = EventId>,
        P: IntoIterator<Item = PublicKey>,
        // W: IntoIterator<Item = S>,
        // S: Into<String>
    {
        Self {
            ids: Arc::new(RwLock::new(ids.into_iter().collect())),
            public_keys: Arc::new(RwLock::new(public_keys.into_iter().collect())),
            //words: Arc::new(RwLock::new(words.into_iter().map(|w| w.into()).collect())),
        }
    }

    /// Construct new empty blacklist
    #[inline]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add [EventId]s to blacklist
    pub async fn add_ids<I>(&self, i: I)
    where
        I: IntoIterator<Item = EventId>,
    {
        let mut ids = self.ids.write().await;
        ids.extend(i);
    }

    /// Remove [EventId]s from blacklist
    pub async fn remove_ids<'a, I>(&self, iter: I)
    where
        I: IntoIterator<Item = &'a EventId>,
    {
        let mut ids = self.ids.write().await;
        for id in iter.into_iter() {
            ids.remove(id);
        }
    }

    /// Remove [EventId] from blacklist
    pub async fn remove_id(&self, id: &EventId) {
        let mut ids = self.ids.write().await;
        ids.remove(id);
    }

    /// Check if blacklist contains event ID
    pub async fn has_id(&self, id: &EventId) -> bool {
        let ids = self.ids.read().await;
        ids.contains(id)
    }

    /// Add [PublicKey]s to blacklist
    pub async fn add_public_keys<I>(&self, iter: I)
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let mut public_keys = self.public_keys.write().await;
        public_keys.extend(iter);
    }

    /// Remove [PublicKey]s from blacklist
    pub async fn remove_public_keys<'a, I>(&self, iter: I)
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut public_keys = self.public_keys.write().await;
        for public_key in iter.into_iter() {
            public_keys.remove(public_key);
        }
    }

    /// Remove [PublicKey] from blacklist
    pub async fn remove_public_key(&self, public_key: &PublicKey) {
        let mut public_keys = self.public_keys.write().await;
        public_keys.remove(public_key);
    }

    /// Check if blacklist contains public key
    pub async fn has_public_key(&self, public_key: &PublicKey) -> bool {
        let public_keys = self.public_keys.read().await;
        public_keys.contains(public_key)
    }

    // /// Add word to blacklist
    // pub async fn add_words<I, S>(&self, iter: I)
    // where
    //     I: IntoIterator<Item = S>,
    //     S: Into<String>
    // {
    //     let mut words = self.words.write().await;
    //     words.extend(iter.into_iter().map(|w| w.into()));
    // }
    //
    // /// Remove word from blacklist
    // pub async fn remove_word<S>(&self, word: S)
    // where
    //     S: AsRef<str>
    // {
    //     let mut words = self.words.write().await;
    //     words.remove(word.as_ref());
    // }
    //
    // /// Check if blacklist contains word
    // pub async fn has_word<S>(&self, word: S) -> bool
    // where
    //     S: AsRef<str>
    // {
    //     let words = self.words.read().await;
    //     words.contains(word.as_ref())
    // }

    /// Remove everything
    pub async fn clear(&self) {
        let mut ids = self.ids.write().await;
        ids.clear();

        let mut public_keys = self.public_keys.write().await;
        public_keys.clear();

        // let mut words = self.words.write().await;
        // words.clear();
    }
}
