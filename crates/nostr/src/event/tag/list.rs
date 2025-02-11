// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Tags (tag list)

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
use std::sync::OnceLock as OnceCell;

use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Error, Tag};
use crate::nips::nip01::Coordinate;
use crate::{EventId, PublicKey, SingleLetterTag, TagKind, TagStandard, Timestamp};

/// Tags Indexes
pub type TagsIndexes = BTreeMap<SingleLetterTag, BTreeSet<String>>;

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

    /// Deduplicate tags
    ///
    /// # Deduplication policy
    ///
    /// - Two tags are considered duplicates if:
    ///   1) They have the same [`TagKind`]
    ///   2) They contain the same content (if applicable)
    ///
    /// - Among duplicates, the longest tag is retained; shorter ones are discarded.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use nostr::Tags;
    /// let tags = [
    ///     vec!["t", "test"], // This will be discarded since exists one with same kind + content and with longer len.
    ///     vec!["t", "test1"],
    ///     vec!["t", "test", "wss://relay.damus.io"],
    /// ];
    /// let mut tags = Tags::parse(tags).unwrap();
    ///
    /// let expected_tags = [
    ///     vec!["t", "test1"],
    ///     vec!["t", "test", "wss://relay.damus.io"],
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

        // Keep track which tag survives
        let mut keep: Vec<bool> = vec![true; self.list.len()];

        // Map from (&str, &str) → index of whichever tag is longest
        let mut map: BTreeMap<(TagKind, Option<&str>), usize> = BTreeMap::new();

        // Figure out which tags to keep
        for (idx, tag) in self.list.iter().enumerate() {
            let kind: TagKind = tag.kind();
            let content: Option<&str> = tag.content();

            let key: (TagKind, Option<&str>) = (kind, content);

            match map.get(&key) {
                // The value already exists
                Some(&old_idx) => {
                    // Compare lengths; keep whichever is longer.
                    if tag.len() > self.list[old_idx].len() {
                        // The current tag is longer -> discard the older one and update the map.
                        keep[old_idx] = false;
                        map.insert(key, idx);
                    } else {
                        // The tag in the map is longer -> discard the current one.
                        keep[idx] = false;
                    }
                }
                // Key not exists, insert.
                None => {
                    map.insert(key, idx);
                }
            }
        }

        // We never use references in the map again after the loop,
        // so any borrowed strings are no longer needed.
        drop(map);

        // Rebuild list
        let mut new_list: Vec<Tag> = Vec::with_capacity(self.list.len());
        for (idx, tag) in self.list.drain(..).enumerate() {
            if keep[idx] {
                new_list.push(tag);
            }
        }

        // Update
        self.list = new_list;
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
            Tag::public_key(pubkey2),
            Tag::event(event1),
            Tag::identifier("test"),
            Tag::alt("testing deduplication"),
            Tag::alt("test"),
            long_e_tag_2,
            long_p_tag_1,
        ];

        assert_eq!(tags.to_vec(), expected);
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;
    use crate::RelayUrl;

    #[bench]
    pub fn tags_dedup(bh: &mut Bencher) {
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

        let list = vec![
            Tag::public_key(pubkey1),
            Tag::public_key(pubkey2),
            Tag::event(event1),
            Tag::event(event2),
            Tag::identifier("test"),
            Tag::alt("testing deduplication"),
            Tag::alt("test"),
            long_e_tag_2.clone(),
            Tag::event(event2),
            long_p_tag_1.clone(),
            Tag::public_key(pubkey2),
            Tag::identifier("test"),
        ];

        let mut tags = Tags::from_list(list);

        bh.iter(|| {
            black_box(tags.dedup());
        });
    }
}
