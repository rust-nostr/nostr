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

impl<T> Output<T>
where
    T: Debug,
{
    /// Create a new output
    #[must_use]
    pub fn new(val: T) -> Self {
        Self {
            val,
            success: HashSet::new(),
            failed: HashMap::new(),
        }
    }

    /// Get inner value
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.val
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

/// Result of sending an event to relays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendEventOutput {
    /// ID of the sent event.
    pub event_id: EventId,
    /// Relays that accepted the event, with an optional OK message.
    pub success: HashMap<RelayUrl, Option<String>>,
    /// Relays that rejected the event, with related errors.
    pub failed: HashMap<RelayUrl, String>,
}

impl SendEventOutput {
    /// Creates an empty result for the given event ID.
    #[must_use]
    pub fn new(event_id: EventId) -> Self {
        Self {
            event_id,
            success: HashMap::new(),
            failed: HashMap::new(),
        }
    }

    /// Returns a reference to the event ID.
    #[inline]
    pub fn id(&self) -> &EventId {
        &self.event_id
    }
}

impl Deref for SendEventOutput {
    type Target = EventId;

    fn deref(&self) -> &Self::Target {
        &self.event_id
    }
}

impl DerefMut for SendEventOutput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event_id
    }
}
