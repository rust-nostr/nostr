// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tags (tag list)

use alloc::vec::{IntoIter, Vec};
use core::fmt;
use core::slice::Iter;

use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Tag;
use crate::nips::nip01::Coordinate;
use crate::{EventId, PublicKey, TagKind, TagStandard, Timestamp};

/// Tag list
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tags {
    list: Vec<Tag>,
}

impl fmt::Debug for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.list)
    }
}

impl Tags {
    /// Construct new tag list.
    #[inline]
    pub fn new(list: Vec<Tag>) -> Self {
        Self { list }
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
