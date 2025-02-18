// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::event::tag::list;
use uniffi::Object;

use super::{Tag, TagKind, TagStandard};
use crate::error::Result;
use crate::protocol::event::{EventId, PublicKey, Timestamp};
use crate::protocol::nips::nip01::Coordinate;

#[derive(Object)]
pub struct Tags {
    inner: list::Tags,
}

impl Deref for Tags {
    type Target = list::Tags;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<list::Tags> for Tags {
    fn from(inner: list::Tags) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Tags {
    #[uniffi::constructor]
    pub fn from_list(list: Vec<Arc<Tag>>) -> Self {
        Self {
            inner: list::Tags::from_list(
                list.into_iter()
                    .map(|t| t.as_ref().deref().clone())
                    .collect(),
            ),
        }
    }

    /// Extract `nostr:` URIs from a text and construct tags.
    ///
    /// This method deduplicates the tags.
    #[uniffi::constructor]
    pub fn from_text(text: &str) -> Self {
        Self {
            inner: list::Tags::from_text(text),
        }
    }

    #[uniffi::constructor]
    pub fn parse(tags: Vec<Vec<String>>) -> Result<Self> {
        Ok(Self {
            inner: list::Tags::parse(tags)?,
        })
    }

    /// Get number of tags
    pub fn len(&self) -> u64 {
        self.inner.len() as u64
    }

    /// Check if contains no tags.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get first tag
    pub fn first(&self) -> Option<Arc<Tag>> {
        self.inner.first().cloned().map(|t| Arc::new(t.into()))
    }

    /// Get last tag
    pub fn last(&self) -> Option<Arc<Tag>> {
        self.inner.last().cloned().map(|t| Arc::new(t.into()))
    }

    /// Get tag at index
    pub fn get(&self, index: u64) -> Option<Arc<Tag>> {
        self.inner
            .get(index as usize)
            .cloned()
            .map(|t| Arc::new(t.into()))
    }

    /// /// Get first tag that match `TagKind`.
    pub fn find(&self, kind: TagKind) -> Option<Arc<Tag>> {
        self.inner
            .find(kind.into())
            .cloned()
            .map(|t| Arc::new(t.into()))
    }

    /// Get first tag that match `TagKind` and that is standardized.
    pub fn find_standardized(&self, kind: TagKind) -> Option<TagStandard> {
        self.inner
            .find_standardized(kind.into())
            .cloned()
            .map(|t| t.into())
    }

    /// Get first tag that match `TagKind`.
    pub fn filter(&self, kind: TagKind) -> Vec<Arc<Tag>> {
        self.inner
            .filter(kind.into())
            .cloned()
            .map(|t| Arc::new(t.into()))
            .collect()
    }

    /// Get first tag that match `TagKind` and that is standardized.
    pub fn filter_standardized(&self, kind: TagKind) -> Vec<TagStandard> {
        self.inner
            .filter_standardized(kind.into())
            .cloned()
            .map(|t| t.into())
            .collect()
    }

    pub fn to_vec(&self) -> Vec<Arc<Tag>> {
        self.inner
            .iter()
            .map(|t| Arc::new(t.clone().into()))
            .collect()
    }

    /// Extract identifier (`d` tag), if exists.
    pub fn identifier(&self) -> Option<String> {
        self.inner.identifier().map(|i| i.to_string())
    }

    /// Get timestamp expiration, if set
    pub fn expiration(&self) -> Option<Arc<Timestamp>> {
        self.inner.expiration().map(|t| Arc::new((*t).into()))
    }

    /// Extract public keys from `p` tags.
    ///
    /// This method extract ONLY supported standard variants
    pub fn public_keys(&self) -> Vec<Arc<PublicKey>> {
        self.inner
            .public_keys()
            .copied()
            .map(|p| Arc::new(p.into()))
            .collect()
    }

    /// Extract event IDs from `e` tags.
    ///
    /// This method extract ONLY supported standard variants
    pub fn event_ids(&self) -> Vec<Arc<EventId>> {
        self.inner
            .event_ids()
            .copied()
            .map(|p| Arc::new(p.into()))
            .collect()
    }

    /// Extract coordinates from `a` tags.
    ///
    /// This method extract ONLY supported standard variants
    pub fn coordinates(&self) -> Vec<Arc<Coordinate>> {
        self.inner
            .coordinates()
            .cloned()
            .map(|p| Arc::new(p.into()))
            .collect()
    }

    /// Extract hashtags from `t` tags.
    ///
    /// This method extract ONLY supported standard variants
    pub fn hashtags(&self) -> Vec<String> {
        self.inner.hashtags().map(|t| t.to_owned()).collect()
    }
}
