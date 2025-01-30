// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP51: Lists
//!
//! <https://github.com/nostr-protocol/nips/blob/master/51.md>

use alloc::string::String;
use alloc::vec::Vec;

use super::nip01::Coordinate;
use crate::{EventId, PublicKey, Tag, TagStandard, Url};

/// Things the user doesn't want to see in their feeds
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MuteList {
    /// Public Keys
    pub public_keys: Vec<PublicKey>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// Event IDs
    pub event_ids: Vec<EventId>,
    /// Words
    pub words: Vec<String>,
}

impl From<MuteList> for Vec<Tag> {
    fn from(
        MuteList {
            public_keys,
            hashtags,
            event_ids,
            words,
        }: MuteList,
    ) -> Self {
        let mut tags =
            Vec::with_capacity(public_keys.len() + hashtags.len() + event_ids.len() + words.len());

        tags.extend(public_keys.into_iter().map(Tag::public_key));
        tags.extend(hashtags.into_iter().map(Tag::hashtag));
        tags.extend(event_ids.into_iter().map(Tag::event));
        tags.extend(
            words
                .into_iter()
                .map(TagStandard::Word)
                .map(Tag::from_standardized),
        );

        tags
    }
}

/// Uncategorized, "global" list of things a user wants to save
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bookmarks {
    /// Event IDs
    pub event_ids: Vec<EventId>,
    /// Coordinates
    pub coordinate: Vec<Coordinate>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// Urls
    pub urls: Vec<Url>,
}

impl From<Bookmarks> for Vec<Tag> {
    fn from(
        Bookmarks {
            event_ids,
            coordinate,
            hashtags,
            urls,
        }: Bookmarks,
    ) -> Self {
        let mut tags =
            Vec::with_capacity(event_ids.len() + coordinate.len() + hashtags.len() + urls.len());

        tags.extend(event_ids.into_iter().map(Tag::event));
        tags.extend(coordinate.into_iter().map(Tag::from));
        tags.extend(hashtags.into_iter().map(Tag::hashtag));
        tags.extend(
            urls.into_iter()
                .map(TagStandard::Url)
                .map(Tag::from_standardized),
        );

        tags
    }
}

/// Topics a user may be interested in and pointers
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interests {
    /// Hashtags
    pub hashtags: Vec<String>,
    /// Coordinates
    pub coordinate: Vec<Coordinate>,
}

impl From<Interests> for Vec<Tag> {
    fn from(
        Interests {
            hashtags,
            coordinate,
        }: Interests,
    ) -> Self {
        let mut tags = Vec::with_capacity(hashtags.len() + coordinate.len());

        tags.extend(hashtags.into_iter().map(Tag::hashtag));
        tags.extend(coordinate.into_iter().map(Tag::from));

        tags
    }
}

/// User preferred emojis and pointers to emoji sets
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Emojis {
    /// Emojis
    pub emojis: Vec<(String, Url)>,
    /// Coordinates
    pub coordinate: Vec<Coordinate>,
}

impl From<Emojis> for Vec<Tag> {
    fn from(Emojis { emojis, coordinate }: Emojis) -> Self {
        let mut tags = Vec::with_capacity(emojis.len() + coordinate.len());

        tags.extend(emojis.into_iter().map(|(s, url)| {
            Tag::from_standardized_without_cell(TagStandard::Emoji { shortcode: s, url })
        }));
        tags.extend(coordinate.into_iter().map(Tag::from));

        tags
    }
}

/// Groups of articles picked by users as interesting and/or belonging to the same category
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArticlesCuration {
    /// Coordinates
    pub coordinate: Vec<Coordinate>,
    /// Event IDs
    pub event_ids: Vec<EventId>,
}

impl From<ArticlesCuration> for Vec<Tag> {
    fn from(
        ArticlesCuration {
            coordinate,
            event_ids,
        }: ArticlesCuration,
    ) -> Self {
        let mut tags = Vec::with_capacity(coordinate.len() + event_ids.len());

        tags.extend(coordinate.into_iter().map(Tag::from));
        tags.extend(event_ids.into_iter().map(Tag::event));

        tags
    }
}
