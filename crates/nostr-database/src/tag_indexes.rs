// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag Indexes

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

use nostr::hashes::siphash24::Hash as SipHash24;
use nostr::hashes::Hash;
use nostr::{Alphabet, SingleLetterTag, Tag};

/// Tag Index Value Size
pub const TAG_INDEX_VALUE_SIZE: usize = 8;

/// Tag Indexes
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TagIndexes {
    inner: BTreeMap<SingleLetterTag, TagIndexValues>,
}

impl Deref for TagIndexes {
    type Target = BTreeMap<SingleLetterTag, TagIndexValues>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TagIndexes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl TagIndexes {
    /// Get hashed `d` tag
    #[inline]
    pub fn identifier(&self) -> Option<[u8; TAG_INDEX_VALUE_SIZE]> {
        let values = self.inner.get(&SingleLetterTag::lowercase(Alphabet::D))?;
        values.first().copied()
    }
}

impl<'a, I> From<I> for TagIndexes
where
    I: Iterator<Item = &'a Tag>,
{
    fn from(iter: I) -> Self {
        let mut tag_index: TagIndexes = TagIndexes::default();
        for (single_letter_tag, content) in
            iter.filter_map(|t| Some((t.single_letter_tag()?, t.content()?)))
        {
            let inner = hash(content);
            tag_index.entry(single_letter_tag).or_default().push(inner);
        }
        tag_index
    }
}

#[inline]
pub(crate) fn hash<S>(value: S) -> [u8; TAG_INDEX_VALUE_SIZE]
where
    S: AsRef<str>,
{
    let mut inner: [u8; TAG_INDEX_VALUE_SIZE] = [0u8; TAG_INDEX_VALUE_SIZE];
    let hash = SipHash24::hash(value.as_ref().as_bytes());
    inner.copy_from_slice(&hash[..TAG_INDEX_VALUE_SIZE]);
    inner
}

/// Tag Index Values
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TagIndexValues {
    inner: Vec<[u8; TAG_INDEX_VALUE_SIZE]>,
}

impl Deref for TagIndexValues {
    type Target = Vec<[u8; TAG_INDEX_VALUE_SIZE]>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TagIndexValues {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl TagIndexValues {
    pub fn iter<'a, I>(iter: I) -> impl Iterator<Item = [u8; TAG_INDEX_VALUE_SIZE]> + 'a
    where
        I: Iterator<Item = &'a String> + 'a,
    {
        iter.map(hash)
    }
}
