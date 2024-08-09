// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(dead_code)]

use std::borrow::Borrow;
use std::collections::btree_set::Iter;
use std::collections::BTreeSet;

/// Represents the possible options for removing a value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OverCapacityPolicy {
    /// Pop first value
    #[default]
    First,
    /// Pop last value
    Last,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capacity {
    #[default]
    Unbounded,
    Bounded {
        max: usize,
        policy: OverCapacityPolicy,
    },
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    #[inline]
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.set.remove(value)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.set.iter()
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
}
