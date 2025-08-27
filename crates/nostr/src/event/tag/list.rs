// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Tags (tag list)

#[cfg(not(feature = "std"))]
use alloc::collections::btree_map::Entry;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::{IntoIter, Vec};
#[cfg(not(feature = "std"))]
use core::cell::OnceCell;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::slice::Iter;
#[cfg(feature = "std")]
use std::collections::hash_map::{Entry, HashMap};
#[cfg(feature = "std")]
use std::sync::OnceLock as OnceCell;

use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Error, Tag};
use crate::nips::nip01::Coordinate;
use crate::{EventId, PublicKey, SingleLetterTag, TagKind, TagStandard, Timestamp};

/// Tags Indexes
pub type TagsIndexes = BTreeMap<SingleLetterTag, BTreeSet<String>>;

struct DedupVal {
    // First index where the tag was seen
    first_idx: usize,
    // The best index, so in this case the longest one
    best_idx: usize,
}

impl DedupVal {
    #[inline]
    fn new(idx: usize) -> Self {
        Self {
            first_idx: idx,
            best_idx: idx,
        }
    }
}

/// Tags collection
#[derive(Clone, Default)]
pub struct Tags {
    list: Vec<Tag>,
    indexes: OnceCell<TagsIndexes>,
}

impl fmt::Debug for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.list)
    }
}

impl PartialEq for Tags {
    fn eq(&self, other: &Self) -> bool {
        self.list == other.list
    }
}

impl Eq for Tags {}

impl PartialOrd for Tags {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tags {
    fn cmp(&self, other: &Self) -> Ordering {
        self.list.cmp(&other.list)
    }
}

impl Hash for Tags {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.list.hash(state);
    }
}

impl Tags {
    /// Construct a new empty collection.
    #[inline]
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            indexes: OnceCell::new(),
        }
    }

    /// Constructs a new, empty collection with at least the specified capacity.
    ///
    /// Check [`Vec::with_capacity`] doc to learn more.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            list: Vec::with_capacity(capacity),
            indexes: OnceCell::new(),
        }
    }

    /// Construct the collection from a list of tags.
    pub fn from_list(list: Vec<Tag>) -> Self {
        Self {
            list,
            indexes: OnceCell::new(),
        }
    }

    /// Parse tags
    pub fn parse<I1, I2, S>(tags: I1) -> Result<Self, Error>
    where
        I1: IntoIterator<Item = I2>,
        I2: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut list: Vec<Tag> = Vec::new();

        for tag in tags.into_iter() {
            let tag: Tag = Tag::parse(tag)?;
            list.push(tag);
        }

        Ok(Self::from_list(list))
    }

    /// Get number of tags.
    #[inline]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Check if contains no tags.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Appends a [`Tag`] to the back of the collection.
    ///
    /// Check [`Vec::push`] doc to learn more.
    ///
    /// This erases the [`TagsIndexes`].
    pub fn push(&mut self, tag: Tag) {
        // Erase indexes
        self.erase_indexes();

        // Append
        self.list.push(tag);
    }

    /// Removes the last [`Tag`] and returns it, or `None` if it's empty.
    ///
    /// Check [`Vec::pop`] doc to learn more.
    ///
    /// This erases the [`TagsIndexes`].
    pub fn pop(&mut self) -> Option<Tag> {
        // Erase indexes
        self.erase_indexes();

        // Pop last item
        self.list.pop()
    }

    /// Inserts a [`Tag`] at position `index` within the vector,
    /// shifting all tags after it to the right.
    ///
    /// Returns `true` if the [`Tag`] is inserted successfully.
    /// Returns `false` if `index > len`.
    ///
    /// Check [`Vec::insert`] doc to learn more.
    ///
    /// This erases the [`TagsIndexes`].
    pub fn insert(&mut self, index: usize, tag: Tag) -> bool {
        // Check if `index` is bigger than collection len
        if index > self.list.len() {
            return false;
        }

        // Erase indexes
        self.erase_indexes();

        // Insert at position
        self.list.insert(index, tag);

        // Inserted successfully
        true
    }

    /// Removes and returns the [`Tag`] at position `index` within the vector,
    /// shifting all tags after it to the left.
    ///
    /// Check [`Vec::remove`] doc to learn more.
    ///
    /// This erases the [`TagsIndexes`].
    pub fn remove(&mut self, index: usize) -> Option<Tag> {
        // Check if `index` is bigger than collection len
        if index > self.list.len() {
            return None;
        }

        // Erase indexes
        self.erase_indexes();

        // Remove from collection
        Some(self.list.remove(index))
    }

    /// Extends the collection.
    ///
    /// Check [`Vec::extend`] doc to learn more.
    ///
    /// This erases the [`TagsIndexes`].
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Tag>,
    {
        // Erase indexes
        self.erase_indexes();

        // Extend list
        self.list.extend(iter);
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e for which `f(&e)` returns `false`.
    ///
    /// Check [`Vec::retain`] doc to learn more.
    ///
    /// This erases the [`TagsIndexes`].
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Tag) -> bool,
    {
        // Erase indexes
        self.erase_indexes();

        // Retain tags
        self.list.retain(|t| f(t));
    }

    /// Deduplicate tags
    ///
    /// # Policy
    ///
    /// - Two tags are considered duplicates if:
    ///   1) They have the same [`TagKind`]
    ///   2) They contain the same content (if applicable)
    ///
    /// - Among duplicates, the longest tag is retained; shorter ones are discarded.
    ///
    /// # Time complexity
    ///
    /// In a `no_std` env takes `O(N log N)` time, otherwise `O(N)`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nostr::Tags;
    /// let tags = [
    ///     vec!["t", "test"], // This will be discarded since an item with the same kind + content and longer len exists.
    ///     vec!["t", "test1"],
    ///     vec!["t", "test", "wss://relay.damus.io"],
    /// ];
    /// let mut tags = Tags::parse(tags).unwrap();
    ///
    /// let expected_tags = [
    ///     vec!["t", "test", "wss://relay.damus.io"], // Replaced the previous shorted tag
    ///     vec!["t", "test1"],
    /// ];
    /// let mut expected_tags = Tags::parse(expected_tags).unwrap();
    ///
    /// assert_eq!(tags, expected_tags);
    /// ```
    pub fn dedup(&mut self) {
        // Erase indexes
        self.erase_indexes();

        // If there are no tags, nothing to do
        if self.list.is_empty() {
            return;
        }

        // Construct the dedup map
        #[cfg(feature = "std")]
        let mut map: HashMap<(TagKind, Option<&str>), DedupVal> =
            HashMap::with_capacity(self.list.len());
        #[cfg(not(feature = "std"))]
        let mut map: BTreeMap<(TagKind, Option<&str>), DedupVal> = BTreeMap::new();

        // Figure out which tags to keep
        for (idx, tag) in self.list.iter().enumerate() {
            // Construct dedup key
            let key: (TagKind, Option<&str>) = (tag.kind(), tag.content());

            // Check if key exists or not
            match map.entry(key) {
                // The key already exists
                Entry::Occupied(mut entry) => {
                    // Get entry value
                    let val: &mut DedupVal = entry.get_mut();

                    // Compare lengths and keep whichever is longer
                    if tag.len() > self.list[val.best_idx].len() {
                        // The current tag is longer -> update the best_idx with the current one
                        val.best_idx = idx;
                    }
                }
                // The key doesn't exist, insert the current index
                Entry::Vacant(entry) => {
                    entry.insert(DedupVal::new(idx));
                }
            }
        }

        // Build a new list, placing the best duplicate at the earliest index
        let mut new_list: Vec<Option<Tag>> = vec![None; self.list.len()];
        for DedupVal {
            first_idx,
            best_idx,
        } in map.into_values()
        {
            new_list[first_idx] = Some(self.list[best_idx].clone()); // TODO: avoid clone here
        }

        // Flatten out the resulting list, skipping positions that are `None`
        self.list = new_list.into_iter().flatten().collect();
    }

    /// Get first tag
    #[inline]
    pub fn first(&self) -> Option<&Tag> {
        self.list.first()
    }

    /// Get last tag
    #[inline]
    pub fn last(&self) -> Option<&Tag> {
        self.list.last()
    }

    /// Get tag at index
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Tag> {
        self.list.get(index)
    }

    /// Iterate tags
    #[inline]
    pub fn iter(&self) -> Iter<'_, Tag> {
        self.list.iter()
    }

    /// Get first tag that match [`TagKind`].
    #[inline]
    pub fn find(&self, kind: TagKind) -> Option<&Tag> {
        self.list.iter().find(|t| t.kind() == kind)
    }

    /// Get the first tag that match [`TagKind`] and that is standardized.
    #[inline]
    pub fn find_standardized(&self, kind: TagKind) -> Option<&TagStandard> {
        self.find(kind).and_then(|t| t.as_standardized())
    }

    /// Filter tags that match [`TagKind`].
    #[inline]
    pub fn filter<'a>(&'a self, kind: TagKind<'a>) -> impl Iterator<Item = &'a Tag> {
        self.list.iter().filter(move |t| t.kind() == kind)
    }

    /// Get the first tag that match [`TagKind`] and that is standardized.
    #[inline]
    pub fn filter_standardized<'a>(
        &'a self,
        kind: TagKind<'a>,
    ) -> impl Iterator<Item = &'a TagStandard> {
        self.filter(kind).filter_map(|t| t.as_standardized())
    }

    /// Get as slice of tags
    #[inline]
    pub fn as_slice(&self) -> &[Tag] {
        &self.list
    }

    /// Convert [`Tags`] into [`Vec<Tag>`].
    #[inline]
    pub fn to_vec(self) -> Vec<Tag> {
        self.list
    }

    /// Extract identifier (`d` tag), if exists.
    #[inline]
    pub fn identifier(&self) -> Option<&str> {
        match self.find_standardized(TagKind::d())? {
            TagStandard::Identifier(identifier) => Some(identifier),
            _ => None,
        }
    }

    /// Get [`Timestamp`] expiration, if exists.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    pub fn expiration(&self) -> Option<&Timestamp> {
        match self.find_standardized(TagKind::Expiration)? {
            TagStandard::Expiration(timestamp) => Some(timestamp),
            _ => None,
        }
    }

    /// Extract NIP42 challenge, if exists.
    #[inline]
    pub fn challenge(&self) -> Option<&str> {
        match self.find_standardized(TagKind::Challenge)? {
            TagStandard::Challenge(challenge) => Some(challenge),
            _ => None,
        }
    }

    /// Extract public keys from `p` tags.
    ///
    /// This method extract only [`TagStandard::PublicKey`], [`TagStandard::PublicKeyReport`] and [`TagStandard::PublicKeyLiveEvent`] variants.
    #[inline]
    pub fn public_keys(&self) -> impl Iterator<Item = &PublicKey> {
        self.filter_standardized(TagKind::p())
            .filter_map(|t| match t {
                TagStandard::PublicKey { public_key, .. } => Some(public_key),
                TagStandard::PublicKeyReport(public_key, ..) => Some(public_key),
                TagStandard::PublicKeyLiveEvent { public_key, .. } => Some(public_key),
                _ => None,
            })
    }

    /// Extract event IDs from `e` tags.
    ///
    /// This method extract only [`TagStandard::Event`] and [`TagStandard::EventReport`] variants.
    #[inline]
    pub fn event_ids(&self) -> impl Iterator<Item = &EventId> {
        self.filter_standardized(TagKind::e())
            .filter_map(|t| match t {
                TagStandard::Event { event_id, .. } => Some(event_id),
                TagStandard::EventReport(event_id, ..) => Some(event_id),
                _ => None,
            })
    }

    /// Extract coordinates from `a` tags.
    ///
    /// This method extract only [`TagStandard::Coordinate`] variant.
    #[inline]
    pub fn coordinates(&self) -> impl Iterator<Item = &Coordinate> {
        self.filter_standardized(TagKind::a())
            .filter_map(|t| match t {
                TagStandard::Coordinate { coordinate, .. } => Some(coordinate),
                _ => None,
            })
    }

    /// Extract hashtags from `t` tags.
    ///
    /// This method extract only [`TagStandard::Hashtag`] variant.
    #[inline]
    pub fn hashtags(&self) -> impl Iterator<Item = &str> {
        self.filter_standardized(TagKind::t())
            .filter_map(|t| match t {
                TagStandard::Hashtag(hashtag) => Some(hashtag.as_ref()),
                _ => None,
            })
    }

    fn build_indexes(&self) -> TagsIndexes {
        let mut idx: TagsIndexes = TagsIndexes::new();
        for (single_letter_tag, content) in self
            .iter()
            .filter_map(|t| Some((t.single_letter_tag()?, t.content()?)))
        {
            idx.entry(single_letter_tag)
                .or_default()
                .insert(content.to_string());
        }
        idx
    }

    #[inline]
    fn erase_indexes(&mut self) {
        if self.indexes.get().is_some() {
            self.indexes = OnceCell::new();
        }
    }

    /// Get indexes
    #[inline]
    pub fn indexes(&self) -> &TagsIndexes {
        self.indexes.get_or_init(|| self.build_indexes())
    }
}

impl AsRef<[Tag]> for Tags {
    fn as_ref(&self) -> &[Tag] {
        self.as_slice()
    }
}

impl IntoIterator for Tags {
    type Item = Tag;
    type IntoIter = IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

impl FromIterator<Tag> for Tags {
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Tag>,
    {
        Self::from_list(iter.into_iter().collect())
    }
}

impl Serialize for Tags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for element in self.list.iter() {
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Tags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        type Data = Vec<Tag>;
        let tags: Vec<Tag> = Data::deserialize(deserializer)?;
        Ok(Self::from_list(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, JsonUtil, RelayUrl};

    #[test]
    fn test_collect() {
        let tags = vec![
            Tag::identifier("1"),
            Tag::identifier("2"),
            Tag::identifier("3"),
            Tag::identifier("4"),
        ];
        let tags: Tags = tags
            .into_iter()
            .filter(|t| t.content() == Some("3"))
            .collect();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags.identifier(), Some("3"));
    }

    #[test]
    fn test_extract_d_tag() {
        let json = r#"{"id":"3dfdbb371de782f51812dc4809ea1104d80e143cec1091a4be07f518ef09e3d7","pubkey":"b8aef32a5421205c1f89ad09e2d93873df68a8611b247f62af005655eadc0efb","created_at":1728728536,"kind":30000,"sig":"0395c41fd95d52b534eaa29c82cd9437130cf63e67117b1587914375fdfb878137287a1d15653161f91ea919afb06358784217409a9ff0323261f683b2936829","content":"older_param_replaceable","tags":[["d","1"]]}"#;
        let event = Event::from_json(json).unwrap();
        assert_eq!(event.tags.identifier(), Some("1"));
    }

    #[test]
    fn test_tags_dedup() {
        let pubkey1 =
            PublicKey::from_hex("b8aef32a5421205c1f89ad09e2d93873df68a8611b247f62af005655eadc0efb")
                .unwrap();
        let pubkey2 =
            PublicKey::from_hex("f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785")
                .unwrap();

        let event1 =
            EventId::from_hex("3dfdbb371de782f51812dc4809ea1104d80e143cec1091a4be07f518ef09e3d7")
                .unwrap();
        let event2 =
            EventId::from_hex("2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45")
                .unwrap();

        let long_p_tag_1 = Tag::from_standardized_without_cell(TagStandard::PublicKey {
            public_key: pubkey1,
            relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
            uppercase: false,
            alias: None,
        });

        let long_e_tag_2 = Tag::from_standardized_without_cell(TagStandard::Event {
            event_id: event2,
            relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
            marker: None,
            public_key: None,
            uppercase: false,
        });

        let empty_list: Vec<String> = Vec::new();

        let list = vec![
            Tag::protected(),
            Tag::custom(TagKind::p(), empty_list.clone()), // Non standard p tag
            Tag::public_key(pubkey1),
            Tag::public_key(pubkey2),
            Tag::event(event1),
            Tag::event(event2),
            Tag::identifier("test"),
            Tag::alt("testing deduplication"),
            Tag::alt("test"),
            long_e_tag_2.clone(),
            Tag::event(event2),
            Tag::protected(),
            long_p_tag_1.clone(),
            Tag::public_key(pubkey2),
            Tag::identifier("test"),
        ];

        let mut tags = Tags::from_list(list);
        tags.dedup();

        let expected = vec![
            Tag::protected(),
            Tag::custom(TagKind::p(), empty_list), // Non standard p tag
            long_p_tag_1,
            Tag::public_key(pubkey2),
            Tag::event(event1),
            long_e_tag_2,
            Tag::identifier("test"),
            Tag::alt("testing deduplication"),
            Tag::alt("test"),
        ];

        assert_eq!(tags.to_vec(), expected);
    }

    // Unit test for issue https://github.com/rust-nostr/nostr/issues/948
    #[test]
    fn test_hashtags_dedup() {
        let mut tags = Tags::new();

        tags.push(Tag::hashtag("a1"));
        tags.push(Tag::hashtag("a1"));
        tags.push(Tag::hashtag("a2"));
        tags.dedup();
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;
    use crate::key::Keys;
    use crate::RelayUrl;

    fn generate_tags(n: usize) -> Tags {
        let half = n / 2;

        let mut pubkeys = Vec::with_capacity(half);

        let mut tags = Vec::with_capacity(n);

        for _ in 0..half {
            let keys = Keys::generate();

            // Save pubkey
            pubkeys.push(keys.public_key());

            // Push simple p tag
            tags.push(Tag::public_key(keys.public_key()));
        }

        for pk in pubkeys.into_iter() {
            // Push long p tag
            let long_p_tag = Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: pk,
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                uppercase: false,
                alias: None,
            });
            tags.push(long_p_tag)
        }

        Tags::from_list(tags)
    }

    #[bench]
    pub fn tags_dedup_10_tags(bh: &mut Bencher) {
        let mut tags = generate_tags(10);

        assert!(tags.len() == 10);

        bh.iter(|| {
            black_box(tags.dedup());
        });

        assert!(tags.len() == 5);
    }

    #[bench]
    pub fn tags_dedup_50_tags(bh: &mut Bencher) {
        let mut tags = generate_tags(50);

        assert!(tags.len() == 50);

        bh.iter(|| {
            black_box(tags.dedup());
        });

        assert!(tags.len() == 25);
    }

    #[bench]
    pub fn tags_dedup_100_tags(bh: &mut Bencher) {
        let mut tags = generate_tags(100);

        assert!(tags.len() == 100);

        bh.iter(|| {
            black_box(tags.dedup());
        });

        assert!(tags.len() == 50);
    }

    #[bench]
    pub fn tags_dedup_500_tags(bh: &mut Bencher) {
        let mut tags = generate_tags(500);

        assert!(tags.len() == 500);

        bh.iter(|| {
            black_box(tags.dedup());
        });

        assert!(tags.len() == 250);
    }

    #[bench]
    pub fn tags_dedup_1000_tags(bh: &mut Bencher) {
        let mut tags = generate_tags(1000);

        assert!(tags.len() == 1000);

        bh.iter(|| {
            black_box(tags.dedup());
        });

        assert!(tags.len() == 500);
    }

    #[bench]
    pub fn tags_dedup_2000_tags(bh: &mut Bencher) {
        let mut tags = generate_tags(2000);

        assert!(tags.len() == 2000);

        bh.iter(|| {
            black_box(tags.dedup());
        });

        assert!(tags.len() == 1000);
    }

    #[bench]
    pub fn tags_dedup_4000_tags(bh: &mut Bencher) {
        let mut tags = generate_tags(4000);

        assert!(tags.len() == 4000);

        bh.iter(|| {
            black_box(tags.dedup());
        });

        assert!(tags.len() == 2000);
    }

    #[bench]
    pub fn tags_push(bh: &mut Bencher) {
        let mut tags = Tags::new();

        bh.iter(|| {
            black_box(tags.push(Tag::protected()));
        });
    }

    #[bench]
    pub fn vec_tag_push(bh: &mut Bencher) {
        let mut tags = Vec::new();

        bh.iter(|| {
            black_box(tags.push(Tag::protected()));
        });
    }

    #[bench]
    pub fn tags_pop(bh: &mut Bencher) {
        let mut tags = generate_tags(4000);

        bh.iter(|| {
            black_box(tags.pop());
        });
    }

    #[bench]
    pub fn vec_tag_pop(bh: &mut Bencher) {
        let tags = generate_tags(4000);
        let mut tags = tags.to_vec();

        bh.iter(|| {
            black_box(tags.pop());
        });
    }
}
