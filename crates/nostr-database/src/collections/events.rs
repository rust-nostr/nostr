// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::cmp::Ordering;
use std::collections::btree_set::IntoIter;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

use nostr::event::borrow::EventBorrow;
use nostr::{Event, Filter, Timestamp};

use super::tree::{BTreeCappedSet, Capacity, OverCapacityPolicy};

// Lookup ID: EVENT_ORD_IMPL
const POLICY: OverCapacityPolicy = OverCapacityPolicy::Last;

#[derive(Debug, Clone)]
pub enum QueryEvent<'a> {
    Owned(Event),
    Borrowed(EventBorrow<'a>),
}

impl<'a> From<Event> for QueryEvent<'a> {
    fn from(event: Event) -> Self {
        Self::Owned(event)
    }
}

impl<'a> From<EventBorrow<'a>> for QueryEvent<'a> {
    fn from(event: EventBorrow<'a>) -> Self {
        Self::Borrowed(event)
    }
}

impl<'a> QueryEvent<'a> {
    pub fn id(&'a self) -> &'a [u8; 32] {
        match self {
            Self::Owned(e) => e.id.as_bytes(),
            Self::Borrowed(e) => e.id,
        }
    }

    pub fn pubkey(&'a self) -> &'a [u8; 32] {
        match self {
            Self::Owned(e) => e.pubkey.as_bytes(),
            Self::Borrowed(e) => e.pubkey,
        }
    }

    pub fn created_at(&self) -> Timestamp {
        match self {
            Self::Owned(e) => e.created_at,
            Self::Borrowed(e) => e.created_at,
        }
    }

    pub fn content(&self) -> &str {
        match self {
            Self::Owned(e) => &e.content,
            Self::Borrowed(e) => e.content,
        }
    }

    pub fn into_owned(self) -> Self {
        match self {
            Self::Owned(e) => Self::Owned(e),
            Self::Borrowed(e) => Self::Owned(e.into_owned()),
        }
    }

    pub fn into_event(self) -> Event {
        match self {
            Self::Owned(e) => e,
            Self::Borrowed(e) => e.into_owned(),
        }
    }
}

impl PartialEq for QueryEvent<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for QueryEvent<'_> {}

impl PartialOrd for QueryEvent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueryEvent<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.created_at() != other.created_at() {
            // Descending order
            // NOT EDIT, will break many things!!
            // If the change is required, search for EVENT_ORD_IMPL comment
            // in the code and adj things.
            self.created_at().cmp(&other.created_at()).reverse()
        } else {
            self.id().cmp(other.id())
        }
    }
}

/// Query events
pub enum QueryEvents<'a> {
    /// Vector
    List(Vec<QueryEvent<'a>>),
    /// BTree
    Set(BTreeSet<QueryEvent<'a>>),
}

impl<'a> QueryEvents<'a> {
    /// Len
    pub fn len(&self) -> usize {
        match self {
            Self::List(events) => events.len(),
            Self::Set(events) => events.len(),
        }
    }

    /// Get first event
    #[inline]
    pub fn first(&self) -> Option<&QueryEvent> {
        match self {
            Self::List(events) => events.first(),
            Self::Set(events) => events.first(),
        }
    }

    /// Get first event
    #[inline]
    pub fn first_owned(self) -> Option<QueryEvent<'a>> {
        match self {
            Self::List(events) => events.into_iter().next(),
            Self::Set(events) => events.into_iter().next(),
        }
    }

    /// Into iterator
    pub fn into_iter(self) -> Box<dyn Iterator<Item = QueryEvent<'a>> + 'a> {
        match self {
            Self::List(events) => Box::new(events.into_iter()),
            Self::Set(events) => Box::new(events.into_iter()),
        }
    }

    /// Convert into [`Events`]
    #[inline]
    pub fn into_owned(self) -> Events {
        self.into_iter().collect()
    }
}

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
    pub fn new(filters: &[Filter]) -> Self {
        // Check how many filters are passed and return the limit
        let limit: Option<usize> = match (filters.len(), filters.first()) {
            (1, Some(filter)) => filter.limit,
            _ => None,
        };

        let mut hasher = DefaultHasher::new();
        filters.hash(&mut hasher);
        let hash: u64 = hasher.finish();

        let set: BTreeCappedSet<_> = match limit {
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

    /// Extend from [`QueryEvents`]
    pub fn extend_query_events(&mut self, events: QueryEvents) {
        self.extend(events.into_iter().map(|e| e.into_event()));
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

impl<'a> FromIterator<QueryEvent<'a>> for Events {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = QueryEvent<'a>>,
    {
        Self {
            set: iter.into_iter().map(|e| e.into_event()).collect(),
            hash: 0,
            prev_not_match: true,
        }
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
            let mut events1 = Events::new(&vec![Filter::new().kind(Kind::TextNote).limit(1)]);
            events1.insert(event1);

            let event2 = Event::from_json(r#"{"content":"Kind 10050 is for DMs, kind 10002 for the other stuff. But both have the same aim. So IMO both have to be under the `gossip` option.","created_at":1732738371,"id":"f2d71a515ce3576d238aaaeaa48fde97388162d08208f729b540a4c3f9723e6b","kind":1,"pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","sig":"d88d3ac21036cfb541809288c12844747dbf1d20a246133dbd37374254b281808c5582bade27c880477759491b2b964d7235142c8b80d233dfb9ae8a50252119","tags":[["e","8262a50cf7832351ae3f21c429e111bb31be0cf754ec437e015534bf5cc2eee8","","root"],["e","0f4bcc83ef2af2febbc7eb9aea5d615a29084ed9e65c467ef2a9387ff79b57e8"],["e","94469431e367b2c16e6d224a4ac2c369c18718a1abdf42759ff591d9816b5ff3","","reply"],["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"],["p","1739d937dc8c0c7370aa27585938c119e25c41f6c441a5d34c6d38503e3136ef"],["p","03f9cfd948e95aeb04f780382344f7c1cfc0210d9af3f4006bb6d451c7b08692"],["p","126103bfddc8df256b6e0abfd7f3797c80dcc4ea88f7c2f87dd4104220b4d65f"],["p","13a665157257e79d9dcc960deeb367fd79383be2d0babb3d861679a5701d463b"],["p","ee0d20b47fb298e8a9ed3609108fe7f2296bd71e8b82fb4f9ff8f61f62bbc7a6"],["p","1c71312fb45273956b078e27981dcc15b178db8d55bffd7ad57a8cfaed6b5ab4"],["p","800e0fe3d8638ce3f75a56ed865df9d96fc9d9cd2f75550df0d7f5c1d8468b0b"]]}"#).unwrap();
            let mut events2 = Events::new(&vec![Filter::new().kind(Kind::TextNote).limit(2)]); // Different filter from above
            events2.insert(event2);

            assert_eq!(events1, events2);
        }

        // NOT match
        {
            let event1 = Event::from_json(r#"{"content":"Kind 10050 is for DMs, kind 10002 for the other stuff. But both have the same aim. So IMO both have to be under the `gossip` option.","created_at":1732738371,"id":"f2d71a515ce3576d238aaaeaa48fde97388162d08208f729b540a4c3f9723e6b","kind":1,"pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","sig":"d88d3ac21036cfb541809288c12844747dbf1d20a246133dbd37374254b281808c5582bade27c880477759491b2b964d7235142c8b80d233dfb9ae8a50252119","tags":[["e","8262a50cf7832351ae3f21c429e111bb31be0cf754ec437e015534bf5cc2eee8","","root"],["e","0f4bcc83ef2af2febbc7eb9aea5d615a29084ed9e65c467ef2a9387ff79b57e8"],["e","94469431e367b2c16e6d224a4ac2c369c18718a1abdf42759ff591d9816b5ff3","","reply"],["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"],["p","1739d937dc8c0c7370aa27585938c119e25c41f6c441a5d34c6d38503e3136ef"],["p","03f9cfd948e95aeb04f780382344f7c1cfc0210d9af3f4006bb6d451c7b08692"],["p","126103bfddc8df256b6e0abfd7f3797c80dcc4ea88f7c2f87dd4104220b4d65f"],["p","13a665157257e79d9dcc960deeb367fd79383be2d0babb3d861679a5701d463b"],["p","ee0d20b47fb298e8a9ed3609108fe7f2296bd71e8b82fb4f9ff8f61f62bbc7a6"],["p","1c71312fb45273956b078e27981dcc15b178db8d55bffd7ad57a8cfaed6b5ab4"],["p","800e0fe3d8638ce3f75a56ed865df9d96fc9d9cd2f75550df0d7f5c1d8468b0b"]]}"#).unwrap();
            let mut events1 = Events::new(&vec![Filter::new().kind(Kind::TextNote).limit(1)]);
            events1.insert(event1);

            let event2 = Event::from_json(r#"{"content":"Thank you !","created_at":1732738224,"id":"035a18ba52a9b40137c0c60ed955eb1f1f93e12423082f6d8a83f62726462d21","kind":1,"pubkey":"1c71312fb45273956b078e27981dcc15b178db8d55bffd7ad57a8cfaed6b5ab4","sig":"54921c7a4f972428c67267a0d99df7d5094c7ca4d26fe9c08221de88ffafb0cab347939ff77129ecfdebad6b18cd2c4c229bf67ce8914fe778d24e19bc22be43","tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"],["p","1739d937dc8c0c7370aa27585938c119e25c41f6c441a5d34c6d38503e3136ef"],["p","03f9cfd948e95aeb04f780382344f7c1cfc0210d9af3f4006bb6d451c7b08692"],["p","126103bfddc8df256b6e0abfd7f3797c80dcc4ea88f7c2f87dd4104220b4d65f"],["p","13a665157257e79d9dcc960deeb367fd79383be2d0babb3d861679a5701d463b"],["p","ee0d20b47fb298e8a9ed3609108fe7f2296bd71e8b82fb4f9ff8f61f62bbc7a6"],["e","8262a50cf7832351ae3f21c429e111bb31be0cf754ec437e015534bf5cc2eee8","wss://nos.lol/","root"],["e","670303f9cbb24568c705b545c277be1f5172ad84795cc9e700aeea5bb248fd74","wss://n.ok0.org/","reply"]]}"#).unwrap();
            let mut events2 = Events::new(&vec![Filter::new().kind(Kind::TextNote).limit(2)]); // Different filter from above
            events2.insert(event2);

            assert_ne!(events1, events2);
        }
    }

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
