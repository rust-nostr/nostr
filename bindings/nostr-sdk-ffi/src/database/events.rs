// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::{Arc, Mutex};

use nostr_sdk::prelude;
use uniffi::Object;

use crate::error::{NostrSdkError, Result};
use crate::protocol::event::Event;

#[derive(Object)]
pub struct Events {
    inner: Mutex<Option<prelude::Events>>,
}

impl From<prelude::Events> for Events {
    fn from(inner: prelude::Events) -> Self {
        Self {
            inner: Mutex::new(Some(inner)),
        }
    }
}

impl Events {
    fn lock_with<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut prelude::Events) -> T,
    {
        let mut inner = self.inner.lock()?;
        match inner.as_mut() {
            Some(inner) => Ok(f(inner)),
            None => Err(NostrSdkError::Generic(
                "Events object already consumed".to_string(),
            )),
        }
    }

    fn take(&self) -> Result<prelude::Events> {
        let mut inner = self.inner.lock()?;
        match inner.take() {
            Some(inner) => Ok(inner),
            None => Err(NostrSdkError::Generic(
                "Events object already consumed".to_string(),
            )),
        }
    }
}

#[uniffi::export]
impl Events {
    /// Returns the number of events in the collection.
    pub fn len(&self) -> u64 {
        self.lock_with(|inner| inner.len() as u64).unwrap_or(0)
    }

    /// Returns the number of events in the collection.
    pub fn is_empty(&self) -> bool {
        self.lock_with(|inner| inner.is_empty()).unwrap_or(true)
    }

    /// Check if contains `Event`
    pub fn contains(&self, event: &Event) -> bool {
        self.lock_with(|inner| inner.contains(event.deref()))
            .unwrap_or(false)
    }

    /// Merge events collections into a single one.
    ///
    /// This method consumes the object, making it unavailable for further use.
    ///
    /// Collection is converted to unbounded if one of the merge `Events` has a different hash.
    /// In other words, the filter limit is respected only if the `Events` are related to the same
    /// list of filters.
    pub fn merge(&self, other: &Self) -> Result<Self> {
        let inner: prelude::Events = self.take()?;
        let other: prelude::Events = other.take()?;
        Ok(inner.merge(other).into())
    }

    /// Get first `Event` (descending order)
    pub fn first(&self) -> Option<Arc<Event>> {
        self.lock_with(|inner| inner.first().cloned().map(|e| Arc::new(e.into())))
            .ok()?
    }

    /// Convert the collection to vector of events.
    ///
    /// This method consumes the object, making it unavailable for further use.
    pub fn to_vec(&self) -> Result<Vec<Arc<Event>>> {
        let inner: prelude::Events = self.take()?;
        Ok(inner.into_iter().map(|e| Arc::new(e.into())).collect())
    }
}
