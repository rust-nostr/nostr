// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Weak tag

use alloc::string::String;
use core::cmp::Ordering;

use super::Tag;

/// [`WeakTag`] wraps [`Tag`] to provide a comparison based on only the first two values of `buf`.
pub struct WeakTag(Tag);

impl WeakTag {
    #[inline]
    pub fn new(tag: Tag) -> Self {
        Self(tag)
    }

    #[inline]
    fn first_two(&self) -> &[String] {
        &self.0.buf[..self.0.buf.len().min(2)]
    }

    #[inline]
    pub fn into_inner(self) -> Tag {
        self.0
    }
}

// Implement ordering and equality based on the first two elements of `buf`
impl PartialEq for WeakTag {
    fn eq(&self, other: &Self) -> bool {
        self.first_two() == other.first_two()
    }
}

impl Eq for WeakTag {}

impl PartialOrd for WeakTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeakTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.first_two().cmp(other.first_two())
    }
}
