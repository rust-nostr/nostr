// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::hash_map::Entry as HashMapEntry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};

use nostr::{EventId, RelayUrl, SubscriptionId};

/// Output
///
/// Send, fetch or negentropy reconciliation output
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
    pub failed: HashMap<RelayUrl, Vec<String>>,
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

    pub(crate) fn push_failed<E>(&mut self, relay: RelayUrl, error: E)
    where
        E: Display,
    {
        match self.failed.entry(relay) {
            HashMapEntry::Vacant(entry) => {
                entry.insert(vec![error.to_string()]);
            }
            HashMapEntry::Occupied(mut entry) => {
                entry.get_mut().push(error.to_string());
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_failed() {
        let relay1 = RelayUrl::parse("wss://relay1.example.com").unwrap();
        let relay2 = RelayUrl::parse("wss://relay2.example.com").unwrap();

        let mut output: Output<()> = Output::default();

        assert!(output.failed.is_empty());

        output.push_failed(relay1.clone(), "error1");
        output.push_failed(relay2.clone(), "error2");

        assert_eq!(output.failed.len(), 2);
        assert_eq!(output.failed.get(&relay1).unwrap(), &vec!["error1"]);
        assert_eq!(output.failed.get(&relay2).unwrap(), &vec!["error2"]);

        output.push_failed(relay1.clone(), "error3");
        assert_eq!(
            output.failed.get(&relay1).unwrap(),
            &vec!["error1", "error3"]
        );
    }
}
