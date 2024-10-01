// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::cmp;
use std::collections::btree_set::IntoIter;
use std::collections::BTreeSet;

use nostr::{Event, Filter};

use crate::tree::{BTreeCappedSet, Capacity, OverCapacityPolicy};

// Lookup ID: EVENT_ORD_IMPL
const POLICY: OverCapacityPolicy = OverCapacityPolicy::Last;

/// Descending sorted collection of events
#[derive(Debug, Clone)]
pub struct Events {
    set: BTreeCappedSet<Event>,
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

        match limit {
            Some(limit) => Self::bounded(limit),
            None => Self::unbounded(),
        }
    }

    /// New bounded collection
    #[inline]
    pub fn bounded(limit: usize) -> Self {
        Self {
            set: BTreeCappedSet::bounded_with_policy(limit, POLICY),
        }
    }

    /// New unbounded collection
    #[inline]
    pub fn unbounded() -> Self {
        Self {
            set: BTreeCappedSet::unbounded(),
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
    /// If one of the collections is bounded, the minimum capacity will be used.
    pub fn merge(mut self, other: Self) -> Self {
        // Get min capacity
        let mut min: Capacity = cmp::min(self.set.capacity(), other.set.capacity());

        // Check over capacity policy
        // Lookup ID: EVENT_ORD_IMPL
        if let Capacity::Bounded {
            max,
            policy: OverCapacityPolicy::First,
        } = min
        {
            min = Capacity::Bounded {
                max,
                policy: POLICY,
            };
        };

        // Update capacity
        self.set.change_capacity(min);

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

impl From<BTreeSet<Event>> for Events {
    fn from(set: BTreeSet<Event>) -> Self {
        Self {
            set: BTreeCappedSet::from(set),
        }
    }
}

// TODO: add unit tests
