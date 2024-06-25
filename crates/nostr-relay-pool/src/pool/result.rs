// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;

use nostr::{EventId, SubscriptionId, Url};

/// Output
///
/// Send or negentropy reconciliation output
// TODO: use a better name?
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output {
    /// Set of relays that success
    pub success: HashSet<Url>,
    /// Map of relays that failed, with related errors.
    pub failed: HashMap<Url, Option<String>>,
}

impl Output {
    pub(super) fn success(url: Url) -> Self {
        let mut success: HashSet<Url> = HashSet::with_capacity(1);
        success.insert(url);
        Self {
            success,
            failed: HashMap::new(),
        }
    }
}

/// Send event output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendEventOutput {
    /// Event ID
    pub id: EventId,
    /// Output
    pub output: Output,
}

impl Deref for SendEventOutput {
    type Target = EventId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl fmt::Display for SendEventOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Subscribe output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeOutput {
    /// Subscription ID
    pub id: SubscriptionId,
    /// Output
    pub output: Output,
}

impl Deref for SubscribeOutput {
    type Target = SubscriptionId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl fmt::Display for SubscribeOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
