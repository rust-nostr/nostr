// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use js_sys::Error;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::nip01::JsCoordinate;
use crate::error::{into_err, Result};
use crate::protocol::event::JsEventId;
use crate::protocol::key::JsPublicKey;

/// Things the user doesn't want to see in their feeds
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Clone)]
#[wasm_bindgen(js_name = MuteList)]
pub struct JsMuteList {
    #[wasm_bindgen(getter_with_clone)]
    pub public_keys: Vec<JsPublicKey>,
    #[wasm_bindgen(getter_with_clone)]
    pub hashtags: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub event_ids: Vec<JsEventId>,
    #[wasm_bindgen(getter_with_clone)]
    pub words: Vec<String>,
}

impl From<JsMuteList> for MuteList {
    fn from(value: JsMuteList) -> Self {
        Self {
            public_keys: value.public_keys.into_iter().map(|p| p.into()).collect(),
            hashtags: value.hashtags,
            event_ids: value.event_ids.into_iter().map(|e| e.into()).collect(),
            words: value.words,
        }
    }
}

/// Uncategorized, "global" list of things a user wants to save
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Clone)]
#[wasm_bindgen(js_name = Bookmarks)]
pub struct JsBookmarks {
    #[wasm_bindgen(getter_with_clone)]
    pub event_ids: Vec<JsEventId>,
    #[wasm_bindgen(getter_with_clone)]
    pub coordinate: Vec<JsCoordinate>,
    #[wasm_bindgen(getter_with_clone)]
    pub hashtags: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub urls: Vec<String>,
}

impl TryFrom<JsBookmarks> for Bookmarks {
    type Error = Error;

    fn try_from(value: JsBookmarks) -> Result<Self, Self::Error> {
        let mut url_list: Vec<Url> = Vec::with_capacity(value.urls.len());

        for url in value.urls.into_iter() {
            url_list.push(Url::from_str(&url).map_err(into_err)?)
        }

        Ok(Self {
            event_ids: value.event_ids.into_iter().map(|e| e.into()).collect(),
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.deref().clone())
                .collect(),
            hashtags: value.hashtags,
            urls: url_list,
        })
    }
}

/// Topics a user may be interested in and pointers
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Clone)]
#[wasm_bindgen(js_name = Interests)]
pub struct JsInterests {
    #[wasm_bindgen(getter_with_clone)]
    pub hashtags: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub coordinate: Vec<JsCoordinate>,
}

impl From<JsInterests> for Interests {
    fn from(value: JsInterests) -> Self {
        Self {
            hashtags: value.hashtags,
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.deref().clone())
                .collect(),
        }
    }
}

/// Emoji
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Clone)]
#[wasm_bindgen(js_name = EmojiInfo)]
pub struct JsEmojiInfo {
    #[wasm_bindgen(getter_with_clone)]
    pub shortcode: String,
    #[wasm_bindgen(getter_with_clone)]
    pub url: String,
}

impl TryFrom<JsEmojiInfo> for (String, Url) {
    type Error = JsValue;

    fn try_from(value: JsEmojiInfo) -> Result<Self, Self::Error> {
        Ok((value.shortcode, Url::parse(&value.url).map_err(into_err)?))
    }
}

/// User preferred emojis and pointers to emoji sets
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Clone)]
#[wasm_bindgen(js_name = Emojis)]
pub struct JsEmojis {
    #[wasm_bindgen(getter_with_clone)]
    pub emojis: Vec<JsEmojiInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub coordinate: Vec<JsCoordinate>,
}

impl From<JsEmojis> for Emojis {
    fn from(value: JsEmojis) -> Self {
        Self {
            // TODO: propagate error
            emojis: value
                .emojis
                .into_iter()
                .filter_map(|e| e.try_into().ok())
                .collect(),
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.deref().clone())
                .collect(),
        }
    }
}

/// Groups of articles picked by users as interesting and/or belonging to the same category
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Clone)]
#[wasm_bindgen(js_name = ArticlesCuration)]
pub struct JsArticlesCuration {
    #[wasm_bindgen(getter_with_clone)]
    pub coordinate: Vec<JsCoordinate>,
    #[wasm_bindgen(getter_with_clone)]
    pub event_ids: Vec<JsEventId>,
}

impl From<JsArticlesCuration> for ArticlesCuration {
    fn from(value: JsArticlesCuration) -> Self {
        Self {
            coordinate: value
                .coordinate
                .into_iter()
                .map(|c| c.deref().clone())
                .collect(),
            event_ids: value.event_ids.into_iter().map(|e| e.into()).collect(),
        }
    }
}
