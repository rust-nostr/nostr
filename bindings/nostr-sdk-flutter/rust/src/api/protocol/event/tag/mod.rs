// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use anyhow::Result;
use flutter_rust_bridge::frb;
use nostr_sdk::prelude::*;

/// Tag
#[frb(name = "Tag")]
pub struct _Tag {
    inner: Tag,
}

impl Deref for _Tag {
    type Target = Tag;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Tag> for _Tag {
    fn from(inner: Tag) -> Self {
        Self { inner }
    }
}

#[frb(sync)]
impl _Tag {
    /// Parse tag
    ///
    /// Return error if the tag is empty!
    pub fn parse(tag: Vec<String>) -> Result<Self> {
        Ok(Self {
            inner: Tag::parse(tag)?,
        })
    }

    // TODO: add from_standardized

    /// Get tag kind
    // TODO: return TagKind
    pub fn kind(&self) -> String {
        self.inner.kind().to_string()
    }

    /// Return the **first** tag value (index `1`), if exists.
    pub fn content(&self) -> Option<String> {
        self.inner.content().map(|c| c.to_string())
    }

    // TODO: add single_letter_tag

    // TODO: add as_standardized

    // TODO: add to_standardized

    /// Get array of strings
    pub fn as_vec(&self) -> Vec<String> {
        self.inner.as_slice().to_vec()
    }

    /// Get array of strings
    pub fn to_vec(self) -> Vec<String> {
        self.inner.to_vec()
    }

    // TODO: add custom
    // /// Compose custom tag
    // ///
    // /// JSON: `["<kind>", "<value-1>", "<value-2>", ...]`
    //
    // pub fn custom(kind: TagKind, values: &[String]) -> Self {
    //     Self {
    //         inner: Tag::custom(kind.into(), values),
    //     }
    // }

    /// Check if is a standard event tag with `root` marker
    pub fn is_root(&self) -> bool {
        self.inner.is_root()
    }

    /// Check if is a standard event tag with `reply` marker
    pub fn is_reply(&self) -> bool {
        self.inner.is_reply()
    }

    /// Check if it's a protected event tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    pub fn is_protected(&self) -> bool {
        self.inner.is_protected()
    }
}
