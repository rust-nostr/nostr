// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip51;
use nostr::{UncheckedUrl, Url};
use uniffi::Record;

use super::nip01::Coordinate;
use crate::error::Result;
use crate::{EventId, NostrError, PublicKey};

#[derive(Record)]
pub struct MuteList {
    pub public_keys: Vec<Arc<PublicKey>>,
    pub hashtags: Vec<String>,
    pub event_ids: Vec<Arc<EventId>>,
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

#[derive(Record)]
pub struct Bookmarks {
    pub event_ids: Vec<Arc<EventId>>,
    pub coordinate: Vec<Arc<Coordinate>>,
    pub hashtags: Vec<String>,
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
                .map(|c| c.as_ref().into())
                .collect(),
            hashtags: value.hashtags,
            urls: url_list,
        })
    }
}

#[derive(Record)]
pub struct Interests {
    pub hashtags: Vec<String>,
    pub coordinate: Vec<Arc<Coordinate>>,
}

impl From<Interests> for nip51::Interests {
    fn from(value: Interests) -> Self {
        Self {
            hashtags: value.hashtags,
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.as_ref().into())
                .collect(),
        }
    }
}

#[derive(Record)]
pub struct EmojiInfo {
    pub shortcode: String,
    pub url: String,
}

impl From<EmojiInfo> for (String, UncheckedUrl) {
    fn from(value: EmojiInfo) -> Self {
        (value.shortcode, UncheckedUrl::from(value.url))
    }
}

#[derive(Record)]
pub struct Emojis {
    /// Emojis
    pub emojis: Vec<EmojiInfo>,
    /// Coordinates
    pub coordinate: Vec<Arc<Coordinate>>,
}

impl From<Emojis> for nip51::Emojis {
    fn from(value: Emojis) -> Self {
        Self {
            emojis: value.emojis.into_iter().map(|e| e.into()).collect(),
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.as_ref().into())
                .collect(),
        }
    }
}

#[derive(Record)]
pub struct ArticlesCuration {
    /// Coordinates
    pub coordinate: Vec<Arc<Coordinate>>,
    /// Event IDs
    pub event_ids: Vec<Arc<EventId>>,
}

impl From<ArticlesCuration> for nip51::ArticlesCuration {
    fn from(value: ArticlesCuration) -> Self {
        Self {
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.as_ref().into())
                .collect(),
            event_ids: value.event_ids.into_iter().map(|e| **e).collect(),
        }
    }
}
