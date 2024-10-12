// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tags (tag list)

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::{IntoIter, Vec};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::slice::Iter;
#[cfg(feature = "std")]
use std::sync::OnceLock;

use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Tag;
use crate::nips::nip01::Coordinate;
use crate::{EventId, PublicKey, SingleLetterTag, TagKind, TagStandard, Timestamp};

/// Tags Indexes
pub type TagsIndexes = BTreeMap<SingleLetterTag, BTreeSet<String>>;

/// Tag list
#[derive(Clone)]
pub struct Tags {
    list: Vec<Tag>,
    #[cfg(feature = "std")]
    indexes: OnceLock<TagsIndexes>,
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
    /// Construct new tag list.
    #[inline]
    pub fn new(list: Vec<Tag>) -> Self {
        Self {
            list,
            #[cfg(feature = "std")]
            indexes: OnceLock::new(),
        }
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

    /// Get first tag that match [`TagKind`] and that is standardized.
    #[inline]
    pub fn find_standardized(&self, kind: TagKind) -> Option<&TagStandard> {
        self.find(kind).and_then(|t| t.as_standardized())
    }

    /// Get first tag that match [`TagKind`].
    #[inline]
    pub fn filter<'a>(&'a self, kind: TagKind<'a>) -> impl Iterator<Item = &'a Tag> {
        self.list.iter().filter(move |t| t.kind() == kind)
    }

    /// Get first tag that match [`TagKind`] and that is standardized.
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
        let standardized: &TagStandard = self.find_standardized(TagKind::d())?;
        match standardized {
            TagStandard::Identifier(identifier) => Some(identifier),
            _ => None,
        }
    }

    /// Get [`Timestamp`] expiration, if set.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    pub fn expiration(&self) -> Option<&Timestamp> {
        let standardized: &TagStandard = self.find_standardized(TagKind::Expiration)?;
        match standardized {
            TagStandard::Expiration(timestamp) => Some(timestamp),
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

    pub(crate) fn build_indexes(&self) -> TagsIndexes {
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

    /// Get indexes
    #[inline]
    #[cfg(feature = "std")]
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
        Ok(Self::new(tags))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Event, JsonUtil};

    #[test]
    fn test_extract_d_tag() {
        let json = r#"{"id":"3dfdbb371de782f51812dc4809ea1104d80e143cec1091a4be07f518ef09e3d7","pubkey":"b8aef32a5421205c1f89ad09e2d93873df68a8611b247f62af005655eadc0efb","created_at":1728728536,"kind":30000,"sig":"0395c41fd95d52b534eaa29c82cd9437130cf63e67117b1587914375fdfb878137287a1d15653161f91ea919afb06358784217409a9ff0323261f683b2936829","content":"older_param_replaceable","tags":[["d","1"]]}"#;
        let event = Event::from_json(json).unwrap();
        assert_eq!(event.tags.identifier(), Some("1"));
    }
}
