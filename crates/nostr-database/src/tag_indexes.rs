// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag Indexes

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use nostr::hashes::siphash24::Hash as SipHash24;
use nostr::hashes::Hash;
use nostr::{Alphabet, GenericTagValue};

/// Tag Index Value Size
pub const TAG_INDEX_VALUE_SIZE: usize = 8;

/// Tag Indexes
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagIndexes {
    inner: HashMap<Alphabet, TagIndexValues>,
}

impl Deref for TagIndexes {
    type Target = HashMap<Alphabet, TagIndexValues>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TagIndexes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<I, S> From<I> for TagIndexes
where
    I: Iterator<Item = Vec<S>>,
    S: AsRef<str>,
{
    fn from(iter: I) -> Self {
        let mut tag_index: TagIndexes = TagIndexes::default();
        for t in iter.filter(|t| t.len() > 1) {
            if let Some(tagnamechar) = single_char_tagname(t[0].as_ref()) {
                let inner = hash(t[1].as_ref());
                tag_index.entry(tagnamechar).or_default().insert(inner);
            }
        }
        tag_index
    }
}

#[inline]
fn single_char_tagname(tagname: &str) -> Option<Alphabet> {
    tagname
        .chars()
        .next()
        .and_then(|first| Alphabet::try_from(first).ok())
}

#[inline]
fn hash<S>(value: S) -> [u8; TAG_INDEX_VALUE_SIZE]
where
    S: AsRef<str>,
{
    let mut inner: [u8; TAG_INDEX_VALUE_SIZE] = [0u8; TAG_INDEX_VALUE_SIZE];
    let hash = SipHash24::hash(value.as_ref().as_bytes());
    inner.copy_from_slice(&hash[..TAG_INDEX_VALUE_SIZE]);
    inner
}

/// Tag Index Values
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagIndexValues {
    inner: HashSet<[u8; TAG_INDEX_VALUE_SIZE]>,
}

impl Deref for TagIndexValues {
    type Target = HashSet<[u8; TAG_INDEX_VALUE_SIZE]>;
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
    #[allow(missing_docs)]
    pub fn iter<'a, I>(iter: I) -> impl Iterator<Item = [u8; TAG_INDEX_VALUE_SIZE]> + 'a
    where
        I: Iterator<Item = &'a GenericTagValue> + 'a,
    {
        iter.map(|value| {
            let s: String = value.to_string();
            hash(s)
        })
    }
}
