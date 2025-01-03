// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Cow Tag

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;

use super::error::Error;
use super::Tag;

/// Cow Tag
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CowTag<'a> {
    buf: Vec<Cow<'a, str>>,
}

impl<'a> CowTag<'a> {
    /// Parse tag
    ///
    /// Return error if the tag is empty!
    pub fn parse(tag: Vec<Cow<'a, str>>) -> Result<Self, Error> {
        // Check if it's empty
        if tag.is_empty() {
            return Err(Error::EmptyTag);
        }

        Ok(Self { buf: tag })
    }

    /// Into owned tag
    pub fn into_owned(self) -> Tag {
        let buf: Vec<String> = self.buf.into_iter().map(|t| t.into_owned()).collect();
        Tag::new_with_empty_cell(buf)
    }

    /// Get inner value
    #[inline]
    pub fn into_inner(self) -> Vec<Cow<'a, str>> {
        self.buf
    }
}
