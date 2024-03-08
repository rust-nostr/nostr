// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip51;
use nostr::Url;
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
