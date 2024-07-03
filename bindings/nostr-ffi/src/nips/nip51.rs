// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip51;
use nostr::{UncheckedUrl, Url};
use uniffi::Record;

use super::nip01::Coordinate;
use crate::error::Result;
use crate::{EventId, NostrError, PublicKey};

/// Things the user doesn't want to see in their feeds
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Record)]
pub struct MuteList {
    #[uniffi(default = [])]
    pub public_keys: Vec<Arc<PublicKey>>,
    #[uniffi(default = [])]
    pub hashtags: Vec<String>,
    #[uniffi(default = [])]
    pub event_ids: Vec<Arc<EventId>>,
    #[uniffi(default = [])]
    pub words: Vec<String>,
}

impl From<MuteList> for nip51::MuteList {
    fn from(value: MuteList) -> Self {
        Self {
            public_keys: value.public_keys.into_iter().map(|p| **p).collect(),
            hashtags: value.hashtags,
            event_ids: value.event_ids.into_iter().map(|e| **e).collect(),
            words: value.words,
        }
    }
}

/// Uncategorized, "global" list of things a user wants to save
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Record)]
pub struct Bookmarks {
    #[uniffi(default = [])]
    pub event_ids: Vec<Arc<EventId>>,
    #[uniffi(default = [])]
    pub coordinate: Vec<Arc<Coordinate>>,
    #[uniffi(default = [])]
    pub hashtags: Vec<String>,
    #[uniffi(default = [])]
    pub urls: Vec<String>,
}

impl TryFrom<Bookmarks> for nip51::Bookmarks {
    type Error = NostrError;

    fn try_from(value: Bookmarks) -> Result<Self, Self::Error> {
        let mut url_list: Vec<Url> = Vec::with_capacity(value.urls.len());

        for url in value.urls.into_iter() {
            url_list.push(Url::from_str(&url)?)
        }

        Ok(Self {
            event_ids: value.event_ids.into_iter().map(|e| **e).collect(),
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.as_ref().deref().clone())
                .collect(),
            hashtags: value.hashtags,
            urls: url_list,
        })
    }
}

/// Topics a user may be interested in and pointers
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Record)]
pub struct Interests {
    #[uniffi(default = [])]
    pub hashtags: Vec<String>,
    #[uniffi(default = [])]
    pub coordinate: Vec<Arc<Coordinate>>,
}

impl From<Interests> for nip51::Interests {
    fn from(value: Interests) -> Self {
        Self {
            hashtags: value.hashtags,
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.as_ref().deref().clone())
                .collect(),
        }
    }
}

/// Emoji
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Record, o2o::o2o)]
#[owned_into((String, UncheckedUrl))]
pub struct EmojiInfo {
    pub shortcode: String,
    #[into(UncheckedUrl::from(~))]
    pub url: String,
}

/// User preferred emojis and pointers to emoji sets
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Record, o2o::o2o)]
#[owned_into(nip51::Emojis)]
pub struct Emojis {
    /// Emojis
    #[uniffi(default = [])]
    #[into(~.into_iter().map(|e| e.into()).collect())]
    pub emojis: Vec<EmojiInfo>,
    /// Coordinates
    #[uniffi(default = [])]
    #[into(~.into_iter().map(|c| c.as_ref().deref().clone()).collect())]
    pub coordinate: Vec<Arc<Coordinate>>,
}

/// Groups of articles picked by users as interesting and/or belonging to the same category
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Record)]
pub struct ArticlesCuration {
    /// Coordinates
    #[uniffi(default = [])]
    pub coordinate: Vec<Arc<Coordinate>>,
    /// Event IDs
    #[uniffi(default = [])]
    pub event_ids: Vec<Arc<EventId>>,
}

impl From<ArticlesCuration> for nip51::ArticlesCuration {
    fn from(value: ArticlesCuration) -> Self {
        Self {
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.as_ref().deref().clone())
                .collect(),
            event_ids: value.event_ids.into_iter().map(|e| **e).collect(),
        }
    }
}
