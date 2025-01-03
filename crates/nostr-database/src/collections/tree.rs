// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(dead_code)]

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::btree_set::{IntoIter, Iter};
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

/// Represents the possible options for removing a value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OverCapacityPolicy {
    /// Pop first value
    #[default]
    First,
    /// Pop last value
    Last,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Capacity {
    #[default]
    Unbounded,
    Bounded {
        max: usize,
        policy: OverCapacityPolicy,
    },
}

impl PartialOrd for Capacity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Capacity {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Unbounded, Self::Unbounded) => Ordering::Equal,
            (Self::Unbounded, Self::Bounded { .. }) => Ordering::Greater,
            (Self::Bounded { .. }, Self::Unbounded) => Ordering::Less,
            (Self::Bounded { max: this_max, .. }, Self::Bounded { max: other_max, .. }) => {
                this_max.cmp(other_max)
            }
        }
    }
}

impl Capacity {
    #[inline]
    pub fn bounded(max: usize) -> Self {
        Self::Bounded {
            max,
            policy: OverCapacityPolicy::default(),
        }
    }
}

pub struct InsertResult<T> {
    /// Return if the value was inserted or not
    pub inserted: bool,
    /// The removed value
    pub pop: Option<T>,
}

#[derive(Debug, Clone)]
pub struct BTreeCappedSet<T> {
    set: BTreeSet<T>,
    capacity: Capacity,
}

impl<T> Default for BTreeCappedSet<T> {
    #[inline]
    fn default() -> Self {
        Self {
            set: BTreeSet::new(),
            capacity: Capacity::default(),
        }
    }
}

impl<T> PartialEq for BTreeCappedSet<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set
    }
}

impl<T> Eq for BTreeCappedSet<T> where T: Eq {}

impl<T> PartialOrd for BTreeCappedSet<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.set.partial_cmp(&other.set)
    }
}

impl<T> Ord for BTreeCappedSet<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.set.cmp(&other.set)
    }
}

impl<T> Hash for BTreeCappedSet<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.set.hash(state);
    }
}

impl<T> BTreeCappedSet<T>
where
    T: Ord,
{
    #[inline]
    pub fn bounded(max: usize) -> Self {
        Self {
            set: BTreeSet::new(),
            capacity: Capacity::bounded(max),
        }
    }

    #[inline]
    pub fn bounded_with_policy(max: usize, policy: OverCapacityPolicy) -> Self {
        Self {
            set: BTreeSet::new(),
            capacity: Capacity::Bounded { max, policy },
        }
    }

    #[inline]
    pub fn unbounded() -> Self {
        Self {
            set: BTreeSet::new(),
            capacity: Capacity::Unbounded,
        }
    }

    /// Get capacity
    #[inline]
    pub fn capacity(&self) -> Capacity {
        self.capacity
    }

    /// Change capacity
    pub fn change_capacity(&mut self, capacity: Capacity) {
        match capacity {
            // Bounded capacity and limit reached
            Capacity::Bounded { max, policy } if self.set.len() > max => {
                while self.set.len() != max {
                    match policy {
                        OverCapacityPolicy::First => self.set.pop_first(),
                        OverCapacityPolicy::Last => self.set.pop_last(),
                    };
                }
            }
            // Unbounded capacity or bounded capacity not reached
            _ => self.capacity = capacity,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.set.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    #[inline]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.set.contains(value)
    }

    /// Insert value
    ///
    /// If the capacity is full, pop and return the last value.
    pub fn insert(&mut self, value: T) -> InsertResult<T> {
        // Check capacity
        match self.capacity {
            // Bounded capacity and limit reached
            Capacity::Bounded { max, policy } if self.set.len() >= max => {
                // Get the last value and compare it to the new value without popping
                let should_insert: bool = match policy {
                    OverCapacityPolicy::First => match self.set.first() {
                        Some(first) => &value > first,
                        None => true,
                    },
                    OverCapacityPolicy::Last => match self.set.last() {
                        Some(last) => &value < last,
                        None => true,
                    },
                };

                if should_insert {
                    // Pop the value if the new value should be inserted
                    InsertResult {
                        inserted: self.set.insert(value),
                        pop: match policy {
                            OverCapacityPolicy::First => self.set.pop_first(),
                            OverCapacityPolicy::Last => self.set.pop_last(),
                        },
                    }
                } else {
                    InsertResult {
                        inserted: false,
                        pop: None,
                    }
                }
            }
            // Unbounded capacity or bounded capacity not reached
            _ => {
                // Insert value
                InsertResult {
                    inserted: self.set.insert(value),
                    pop: None,
                }
            }
        }
    }

    /// Extend with values
    pub fn extend<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = T>,
    {
        match self.capacity {
            Capacity::Bounded { .. } => {
                // TODO: find more efficient way
                for value in values.into_iter() {
                    self.insert(value);
                }
            }
            Capacity::Unbounded => {
                self.set.extend(values);
            }
        }
    }

    #[inline]
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.set.remove(value)
    }

    /// Get first value
    #[inline]
    pub fn first(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.set.first()
    }

    /// Get last value
    #[inline]
    pub fn last(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.set.last()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.set.iter()
    }
}

impl<T> From<BTreeSet<T>> for BTreeCappedSet<T> {
    fn from(set: BTreeSet<T>) -> Self {
        Self {
            set,
            capacity: Capacity::Unbounded,
        }
    }
}

impl<T> FromIterator<T> for BTreeCappedSet<T>
where
    T: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            set: iter.into_iter().collect(),
            capacity: Capacity::Unbounded,
        }
    }
}

impl<T> IntoIterator for BTreeCappedSet<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut set = BTreeCappedSet::bounded(2);

        let res = set.insert(1);
        assert!(res.inserted);
        assert!(res.pop.is_none());
        assert_eq!(set.len(), 1);

        let res = set.insert(2);
        assert!(res.inserted);
        assert!(res.pop.is_none());
        assert_eq!(set.len(), 2);

        // exceeds capacity, 1 is removed
        let res = set.insert(3);
        assert!(res.inserted);
        assert_eq!(res.pop, Some(1));
        assert_eq!(set.len(), 2);

        // try to re-insert 1
        let res = set.insert(1);
        assert!(!res.inserted); // NOT inserted (cap reached)
        assert_eq!(res.pop, None);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_insert_inverted() {
        let mut set = BTreeCappedSet::bounded_with_policy(2, OverCapacityPolicy::Last);

        let res = set.insert(1);
        assert!(res.inserted);
        assert!(res.pop.is_none());
        assert_eq!(set.len(), 1);

        let res = set.insert(2);
        assert!(res.inserted);
        assert!(res.pop.is_none());
        assert_eq!(set.len(), 2);

        // exceeds capacity, 2 is removed
        let res = set.insert(0);
        assert!(res.inserted);
        assert_eq!(res.pop, Some(2));
        assert_eq!(set.len(), 2);

        // try to insert 3
        let res = set.insert(3);
        assert!(!res.inserted); // NOT inserted (cap reached and inverted policy)
        assert_eq!(res.pop, None);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_remove() {
        let mut set = BTreeCappedSet::bounded(3);
        set.insert(1);
        set.insert(2);
        set.insert(3);

        assert!(set.remove(&1));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_change_capacity() {
        let mut set = BTreeCappedSet::bounded(3);
        set.insert(1);
        set.insert(2);
        set.insert(3);

        // resize, discarding elements to cap the capacity
        set.change_capacity(Capacity::bounded(2));

        // 1 has been discarded due to resize
        assert_eq!(set.len(), 2);
        assert!(!set.remove(&1));
    }

    #[test]
    fn test_iter() {
        let mut set = BTreeCappedSet::bounded(3);
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let mut iter = set.iter();

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
    }

    #[test]
    fn test_cmp_capacity() {
        assert!(Capacity::Unbounded > Capacity::bounded(1000));
        assert!(Capacity::bounded(1) < Capacity::bounded(1000));
        assert_eq!(Capacity::Unbounded, Capacity::Unbounded);
    }
}
