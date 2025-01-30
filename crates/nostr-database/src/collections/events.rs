// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::btree_set::IntoIter;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use nostr::{Event, Filter};

use super::tree::{BTreeCappedSet, Capacity, OverCapacityPolicy};

// Lookup ID: EVENT_ORD_IMPL
const POLICY: OverCapacityPolicy = OverCapacityPolicy::Last;

/// Descending sorted collection of events
#[derive(Debug, Clone)]
pub struct Events {
    set: BTreeCappedSet<Event>,
    hash: u64,
    prev_not_match: bool,
}

impl PartialEq for Events {
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set
    }
}

impl Eq for Events {}

impl Events {
    /// New collection
    #[inline]
    pub fn new(filter: &Filter) -> Self {
        let mut hasher = DefaultHasher::new();
        filter.hash(&mut hasher);
        let hash: u64 = hasher.finish();

        let set: BTreeCappedSet<Event> = match filter.limit {
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

    /// Get first [`Event`] (descending order)
    #[inline]
    pub fn first_owned(self) -> Option<Event> {
        // Lookup ID: EVENT_ORD_IMPL
        self.into_iter().next()
    }

    /// Get last [`Event`] (descending order)
    #[inline]
    pub fn last(&self) -> Option<&Event> {
        // Lookup ID: EVENT_ORD_IMPL
        self.set.last()
    }

    /// Get last [`Event`] (descending order)
    #[inline]
    pub fn last_owned(self) -> Option<Event> {
        // Lookup ID: EVENT_ORD_IMPL
        self.into_iter().next_back()
    }

    /// Iterate events in descending order
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        // Lookup ID: EVENT_ORD_IMPL
        self.set.iter()
    }

    /// Convert the collection to vector of events.
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
    use nostr::{JsonUtil, Kind};

    use super::*;

    #[test]
    fn test_events_equality() {
        // Match
        {
            let event1 = Event::from_json(r#"{"content":"Kind 10050 is for DMs, kind 10002 for the other stuff. But both have the same aim. So IMO both have to be under the `gossip` option.","created_at":1732738371,"id":"f2d71a515ce3576d238aaaeaa48fde97388162d08208f729b540a4c3f9723e6b","kind":1,"pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","sig":"d88d3ac21036cfb541809288c12844747dbf1d20a246133dbd37374254b281808c5582bade27c880477759491b2b964d7235142c8b80d233dfb9ae8a50252119","tags":[["e","8262a50cf7832351ae3f21c429e111bb31be0cf754ec437e015534bf5cc2eee8","","root"],["e","0f4bcc83ef2af2febbc7eb9aea5d615a29084ed9e65c467ef2a9387ff79b57e8"],["e","94469431e367b2c16e6d224a4ac2c369c18718a1abdf42759ff591d9816b5ff3","","reply"],["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"],["p","1739d937dc8c0c7370aa27585938c119e25c41f6c441a5d34c6d38503e3136ef"],["p","03f9cfd948e95aeb04f780382344f7c1cfc0210d9af3f4006bb6d451c7b08692"],["p","126103bfddc8df256b6e0abfd7f3797c80dcc4ea88f7c2f87dd4104220b4d65f"],["p","13a665157257e79d9dcc960deeb367fd79383be2d0babb3d861679a5701d463b"],["p","ee0d20b47fb298e8a9ed3609108fe7f2296bd71e8b82fb4f9ff8f61f62bbc7a6"],["p","1c71312fb45273956b078e27981dcc15b178db8d55bffd7ad57a8cfaed6b5ab4"],["p","800e0fe3d8638ce3f75a56ed865df9d96fc9d9cd2f75550df0d7f5c1d8468b0b"]]}"#).unwrap();
            let mut events1 = Events::new(&Filter::new().kind(Kind::TextNote).limit(1));
            events1.insert(event1);

            let event2 = Event::from_json(r#"{"content":"Kind 10050 is for DMs, kind 10002 for the other stuff. But both have the same aim. So IMO both have to be under the `gossip` option.","created_at":1732738371,"id":"f2d71a515ce3576d238aaaeaa48fde97388162d08208f729b540a4c3f9723e6b","kind":1,"pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","sig":"d88d3ac21036cfb541809288c12844747dbf1d20a246133dbd37374254b281808c5582bade27c880477759491b2b964d7235142c8b80d233dfb9ae8a50252119","tags":[["e","8262a50cf7832351ae3f21c429e111bb31be0cf754ec437e015534bf5cc2eee8","","root"],["e","0f4bcc83ef2af2febbc7eb9aea5d615a29084ed9e65c467ef2a9387ff79b57e8"],["e","94469431e367b2c16e6d224a4ac2c369c18718a1abdf42759ff591d9816b5ff3","","reply"],["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"],["p","1739d937dc8c0c7370aa27585938c119e25c41f6c441a5d34c6d38503e3136ef"],["p","03f9cfd948e95aeb04f780382344f7c1cfc0210d9af3f4006bb6d451c7b08692"],["p","126103bfddc8df256b6e0abfd7f3797c80dcc4ea88f7c2f87dd4104220b4d65f"],["p","13a665157257e79d9dcc960deeb367fd79383be2d0babb3d861679a5701d463b"],["p","ee0d20b47fb298e8a9ed3609108fe7f2296bd71e8b82fb4f9ff8f61f62bbc7a6"],["p","1c71312fb45273956b078e27981dcc15b178db8d55bffd7ad57a8cfaed6b5ab4"],["p","800e0fe3d8638ce3f75a56ed865df9d96fc9d9cd2f75550df0d7f5c1d8468b0b"]]}"#).unwrap();
            let mut events2 = Events::new(&Filter::new().kind(Kind::TextNote).limit(2)); // Different filter from above
            events2.insert(event2);

            assert_eq!(events1, events2);
        }

        // NOT match
        {
            let event1 = Event::from_json(r#"{"content":"Kind 10050 is for DMs, kind 10002 for the other stuff. But both have the same aim. So IMO both have to be under the `gossip` option.","created_at":1732738371,"id":"f2d71a515ce3576d238aaaeaa48fde97388162d08208f729b540a4c3f9723e6b","kind":1,"pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","sig":"d88d3ac21036cfb541809288c12844747dbf1d20a246133dbd37374254b281808c5582bade27c880477759491b2b964d7235142c8b80d233dfb9ae8a50252119","tags":[["e","8262a50cf7832351ae3f21c429e111bb31be0cf754ec437e015534bf5cc2eee8","","root"],["e","0f4bcc83ef2af2febbc7eb9aea5d615a29084ed9e65c467ef2a9387ff79b57e8"],["e","94469431e367b2c16e6d224a4ac2c369c18718a1abdf42759ff591d9816b5ff3","","reply"],["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"],["p","1739d937dc8c0c7370aa27585938c119e25c41f6c441a5d34c6d38503e3136ef"],["p","03f9cfd948e95aeb04f780382344f7c1cfc0210d9af3f4006bb6d451c7b08692"],["p","126103bfddc8df256b6e0abfd7f3797c80dcc4ea88f7c2f87dd4104220b4d65f"],["p","13a665157257e79d9dcc960deeb367fd79383be2d0babb3d861679a5701d463b"],["p","ee0d20b47fb298e8a9ed3609108fe7f2296bd71e8b82fb4f9ff8f61f62bbc7a6"],["p","1c71312fb45273956b078e27981dcc15b178db8d55bffd7ad57a8cfaed6b5ab4"],["p","800e0fe3d8638ce3f75a56ed865df9d96fc9d9cd2f75550df0d7f5c1d8468b0b"]]}"#).unwrap();
            let mut events1 = Events::new(&Filter::new().kind(Kind::TextNote).limit(1));
            events1.insert(event1);

            let event2 = Event::from_json(r#"{"content":"Thank you !","created_at":1732738224,"id":"035a18ba52a9b40137c0c60ed955eb1f1f93e12423082f6d8a83f62726462d21","kind":1,"pubkey":"1c71312fb45273956b078e27981dcc15b178db8d55bffd7ad57a8cfaed6b5ab4","sig":"54921c7a4f972428c67267a0d99df7d5094c7ca4d26fe9c08221de88ffafb0cab347939ff77129ecfdebad6b18cd2c4c229bf67ce8914fe778d24e19bc22be43","tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"],["p","1739d937dc8c0c7370aa27585938c119e25c41f6c441a5d34c6d38503e3136ef"],["p","03f9cfd948e95aeb04f780382344f7c1cfc0210d9af3f4006bb6d451c7b08692"],["p","126103bfddc8df256b6e0abfd7f3797c80dcc4ea88f7c2f87dd4104220b4d65f"],["p","13a665157257e79d9dcc960deeb367fd79383be2d0babb3d861679a5701d463b"],["p","ee0d20b47fb298e8a9ed3609108fe7f2296bd71e8b82fb4f9ff8f61f62bbc7a6"],["e","8262a50cf7832351ae3f21c429e111bb31be0cf754ec437e015534bf5cc2eee8","wss://nos.lol/","root"],["e","670303f9cbb24568c705b545c277be1f5172ad84795cc9e700aeea5bb248fd74","wss://n.ok0.org/","reply"]]}"#).unwrap();
            let mut events2 = Events::new(&Filter::new().kind(Kind::TextNote).limit(2)); // Different filter from above
            events2.insert(event2);

            assert_ne!(events1, events2);
        }
    }

    #[test]
    fn test_merge() {
        // Same filter
        let filter = Filter::new().kind(Kind::TextNote).limit(100);

        let events1 = Events::new(&filter);
        assert_eq!(
            events1.set.capacity(),
            Capacity::Bounded {
                max: 100,
                policy: POLICY
            }
        );

        let events2 = Events::new(&filter);
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
        let filter1 = Filter::new().kind(Kind::TextNote).limit(100);
        let filter2 = Filter::new().kind(Kind::Metadata).limit(10);
        let filter3 = Filter::new().kind(Kind::ContactList).limit(1);

        let events1 = Events::new(&filter1);
        assert_eq!(
            events1.set.capacity(),
            Capacity::Bounded {
                max: 100,
                policy: POLICY
            }
        );

        let events2 = Events::new(&filter2);
        assert_eq!(
            events2.set.capacity(),
            Capacity::Bounded {
                max: 10,
                policy: POLICY
            }
        );

        let events3 = Events::new(&filter3);
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
