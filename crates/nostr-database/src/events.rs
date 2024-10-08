// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::btree_set::IntoIter;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use nostr::{Event, Filter};

use crate::tree::{BTreeCappedSet, Capacity, OverCapacityPolicy};

// Lookup ID: EVENT_ORD_IMPL
const POLICY: OverCapacityPolicy = OverCapacityPolicy::Last;

/// Descending sorted collection of events
#[derive(Debug, Clone)]
pub struct Events {
    set: BTreeCappedSet<Event>,
    hash: u64,
    prev_not_match: bool,
}

impl Events {
    /// New collection
    #[inline]
    pub fn new(filters: &[Filter]) -> Self {
        // Check how many filters are passed and return the limit
        let limit: Option<usize> = match (filters.len(), filters.first()) {
            (1, Some(filter)) => filter.limit,
            _ => None,
        };

        let mut hasher = DefaultHasher::new();
        filters.hash(&mut hasher);
        let hash: u64 = hasher.finish();

        let set: BTreeCappedSet<Event> = match limit {
            Some(limit) => BTreeCappedSet::bounded_with_policy(limit, POLICY),
            None => BTreeCappedSet::unbounded(),
        };

        Self {
            set,
            hash,
            prev_not_match: false,
        }
    }

    /// Returns the number of events in the collection.
    #[inline]
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// Returns the number of events in the collection.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Check if contains [`Event`]
    #[inline]
    pub fn contains(&self, event: &Event) -> bool {
        self.set.contains(event)
    }

    /// Insert [`Event`]
    ///
    /// If the set did not previously contain an equal value, `true` is returned.
    #[inline]
    pub fn insert(&mut self, event: Event) -> bool {
        self.set.insert(event).inserted
    }

    /// Insert events
    #[inline]
    pub fn extend<I>(&mut self, events: I)
    where
        I: IntoIterator<Item = Event>,
    {
        self.set.extend(events);
    }

    /// Merge events collections into a single one.
    ///
    /// Collection is converted to unbounded if one of the merge [`Events`] have a different hash.
    /// In other words, the filters limit is respected only if the [`Events`] are related to the same
    /// list of filters.
    pub fn merge(mut self, other: Self) -> Self {
        // Hash not match -> change capacity to unbounded
        if self.hash != other.hash || self.prev_not_match || other.prev_not_match {
            self.set.change_capacity(Capacity::Unbounded);
            self.hash = 0;
            self.prev_not_match = true;
        }

        // Extend
        self.extend(other.set);

        self
    }

    /// Get first [`Event`] (descending order)
    #[inline]
    pub fn first(&self) -> Option<&Event> {
        // Lookup ID: EVENT_ORD_IMPL
        self.set.first()
    }

    /// Get last [`Event`] (descending order)
    #[inline]
    pub fn last(&self) -> Option<&Event> {
        // Lookup ID: EVENT_ORD_IMPL
        self.set.last()
    }

    /// Iterate events in descending order
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        // Lookup ID: EVENT_ORD_IMPL
        self.set.iter()
    }

    /// Convert collection to vector of events.
    #[inline]
    pub fn to_vec(self) -> Vec<Event> {
        self.into_iter().collect()
    }
}

impl IntoIterator for Events {
    type Item = Event;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // Lookup ID: EVENT_ORD_IMPL
        self.set.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use nostr::Kind;

    use super::*;

    #[test]
    fn test_merge() {
        // Same filter
        let filters = vec![Filter::new().kind(Kind::TextNote).limit(100)];

        let events1 = Events::new(&filters);
        assert_eq!(
            events1.set.capacity(),
            Capacity::Bounded {
                max: 100,
                policy: POLICY
            }
        );

        let events2 = Events::new(&filters);
        assert_eq!(
            events2.set.capacity(),
            Capacity::Bounded {
                max: 100,
                policy: POLICY
            }
        );

        let hash1 = events1.hash;

        assert_eq!(events1.hash, events2.hash);

        let events = events1.merge(events2);
        assert_eq!(events.hash, hash1);
        assert!(!events.prev_not_match);
        assert_eq!(
            events.set.capacity(),
            Capacity::Bounded {
                max: 100,
                policy: POLICY
            }
        );

        // Different filters
        let filters1 = vec![Filter::new().kind(Kind::TextNote).limit(100)];
        let filters2 = vec![Filter::new().kind(Kind::Metadata).limit(10)];
        let filters3 = vec![Filter::new().kind(Kind::ContactList).limit(1)];

        let events1 = Events::new(&filters1);
        assert_eq!(
            events1.set.capacity(),
            Capacity::Bounded {
                max: 100,
                policy: POLICY
            }
        );

        let events2 = Events::new(&filters2);
        assert_eq!(
            events2.set.capacity(),
            Capacity::Bounded {
                max: 10,
                policy: POLICY
            }
        );

        let events3 = Events::new(&filters3);
        assert_eq!(
            events3.set.capacity(),
            Capacity::Bounded {
                max: 1,
                policy: POLICY
            }
        );

        assert_ne!(events1.hash, events2.hash);

        let events = events1.merge(events2);
        assert_eq!(events.hash, 0);
        assert!(events.prev_not_match);
        assert_eq!(events.set.capacity(), Capacity::Unbounded);

        let events = events.merge(events3);
        assert_eq!(events.hash, 0);
        assert!(events.prev_not_match);
        assert_eq!(events.set.capacity(), Capacity::Unbounded);
    }
}
