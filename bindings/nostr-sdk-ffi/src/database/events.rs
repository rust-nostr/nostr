// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::prelude;
use uniffi::Object;

use crate::protocol::event::Event;

#[derive(Clone, Object)]
pub struct Events {
    inner: prelude::Events,
}

impl From<prelude::Events> for Events {
    fn from(inner: prelude::Events) -> Self {
        Self { inner }
    }
}

impl Deref for Events {
    type Target = prelude::Events;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Events {
    /// Returns the number of events in the collection.
    pub fn len(&self) -> u64 {
        self.inner.len() as u64
    }

    /// Returns the number of events in the collection.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if contains `Event`
    pub fn contains(&self, event: &Event) -> bool {
        self.inner.contains(event.deref())
    }

    /// Merge events collections into a single one.
    ///
    /// Collection is converted to unbounded if one of the merge `Events` have a different hash.
    /// In other words, the filters limit is respected only if the `Events` are related to the same
    /// list of filters.
    pub fn merge(&self, other: &Self) -> Self {
        self.inner.clone().merge(other.inner.clone()).into()
    }

    /// Get first `Event` (descending order)
    pub fn first(&self) -> Option<Arc<Event>> {
        self.inner.first().cloned().map(|e| Arc::new(e.into()))
    }

    /// Convert collection to vector of events.
    pub fn to_vec(&self) -> Vec<Arc<Event>> {
        self.inner
            .iter()
            .cloned()
            .map(|e| Arc::new(e.into()))
            .collect()
    }
}
