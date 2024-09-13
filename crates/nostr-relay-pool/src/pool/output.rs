// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use nostr::{EventId, SubscriptionId, Url};

/// Output
///
/// Send or negentropy reconciliation output
// TODO: use a better name?
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output<T>
where
    T: Debug,
{
    /// Value
    pub val: T,
    /// Set of relays that success
    pub success: HashSet<Url>,
    /// Map of relays that failed, with related errors.
    pub failed: HashMap<Url, Option<String>>,
}

impl<T> Output<T>
where
    T: Debug,
{
    pub(super) fn success(url: Url, val: T) -> Self {
        let mut success: HashSet<Url> = HashSet::with_capacity(1);
        success.insert(url);
        Self {
            val,
            success,
            failed: HashMap::new(),
        }
    }
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
