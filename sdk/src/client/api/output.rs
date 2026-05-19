// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use nostr::{EventId, RelayUrl, SubscriptionId};

/// Output
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output<T, S = (), F = String> {
    /// The output value
    pub value: T,
    /// Successful relays and their operation-specific output.
    pub success: HashMap<RelayUrl, S>,
    /// Map of relays that failed with related errors.
    pub failed: HashMap<RelayUrl, F>,
}

impl<T, S, F> Deref for Output<T, S, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, S, F> DerefMut for Output<T, S, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, S, F> Output<T, S, F> {
    /// Create a new output
    #[inline]
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            value,
            success: HashMap::new(),
            failed: HashMap::new(),
        }
    }
}

impl<S, F> Output<EventId, S, F> {
    /// Get event ID
    #[inline]
    #[must_use]
    pub fn id(&self) -> &EventId {
        self.deref()
    }
}

impl<S, F> Output<SubscriptionId, S, F> {
    /// Get subscription ID
    #[inline]
    #[must_use]
    pub fn id(&self) -> &SubscriptionId {
        self.deref()
    }
}
