// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use nostr::{EventId, RelayUrl, SubscriptionId};

/// Output
///
/// Send or negentropy reconciliation output
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output<T>
where
    T: Debug,
{
    /// Value
    pub val: T,
    /// Set of relays that success
    pub success: HashSet<RelayUrl>,
    /// Map of relays that failed, with related errors.
    pub failed: HashMap<RelayUrl, String>,
}

impl<T> Deref for Output<T>
where
    T: Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T> DerefMut for Output<T>
where
    T: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl Output<EventId> {
    /// Get event ID
    #[inline]
    pub fn id(&self) -> &EventId {
        self.deref()
    }
}

impl Output<SubscriptionId> {
    /// Get subscription ID
    #[inline]
    pub fn id(&self) -> &SubscriptionId {
        self.deref()
    }
}
