// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use js_sys::Array;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::JsTag;
use crate::error::{into_err, Result};
use crate::protocol::event::JsEventId;
use crate::protocol::key::JsPublicKey;
use crate::protocol::nips::nip01::JsCoordinate;
use crate::protocol::types::JsTimestamp;
use crate::JsStringArray;

#[wasm_bindgen(js_name = Tags)]
pub struct JsTags {
    inner: Tags,
}

impl Deref for JsTags {
    type Target = Tags;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Tags> for JsTags {
    fn from(inner: Tags) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Tags)]
impl JsTags {
    #[wasm_bindgen(js_name = fromList)]
    pub fn from_list(list: Vec<JsTag>) -> Self {
        Self {
            inner: Tags::from_list(list.into_iter().map(|t| t.inner).collect()),
        }
    }

    /// Extract `nostr:` URIs from a text and construct tags.
    ///
    /// This method deduplicates the tags.
    #[wasm_bindgen(js_name = fromText)]
    pub fn from_text(text: &str) -> Self {
        Self {
            inner: list::Tags::from_text(text),
        }
    }

    #[wasm_bindgen]
    pub fn parse(tags: Vec<JsStringArray>) -> Result<JsTags> {
        let mut new_tags: Vec<Vec<String>> = Vec::with_capacity(tags.len());

        for tag in tags.into_iter() {
            let array: Array = tag.dyn_into()?;
            let mut tag: Vec<String> = Vec::with_capacity(array.length() as usize);

            for val in array.into_iter() {
                let val: String = val
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("tag values must be strings"))?;
                tag.push(val);
            }

            new_tags.push(tag);
        }

        Ok(Self {
            inner: Tags::parse(new_tags).map_err(into_err)?,
        })
    }

    /// Get number of tags
    pub fn len(&self) -> u64 {
        self.inner.len() as u64
    }

    /// Check if contains no tags.
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get first tag
    pub fn first(&self) -> Option<JsTag> {
        self.inner.first().cloned().map(|t| t.into())
    }

    /// Get last tag
    pub fn last(&self) -> Option<JsTag> {
        self.inner.last().cloned().map(|t| t.into())
    }

    /// Get tag at index
    pub fn get(&self, index: u64) -> Option<JsTag> {
        self.inner.get(index as usize).cloned().map(|t| t.into())
    }

    /// /// Get first tag that match tag kind
    pub fn find(&self, kind: &str) -> Option<JsTag> {
        self.inner
            .find(TagKind::from(kind))
            .cloned()
            .map(|t| t.into())
    }

    /// Get first tag that match tag kind.
    pub fn filter(&self, kind: &str) -> Vec<JsTag> {
        self.inner
            .filter(TagKind::from(kind))
            .cloned()
            .map(|t| t.into())
            .collect()
    }

    /// Clone the object and return list of tags
    #[wasm_bindgen(js_name = asVec)]
    pub fn as_vec(&self) -> Vec<JsTag> {
        self.inner.iter().cloned().map(|t| t.into()).collect()
    }

    /// This method consume the object and return a list of tags
    #[wasm_bindgen(js_name = toVec)]
    pub fn to_vec(self) -> Vec<JsTag> {
        self.inner.into_iter().map(|t| t.into()).collect()
    }

    /// Extract identifier (`d` tag), if exists.
    pub fn identifier(&self) -> Option<String> {
        self.inner.identifier().map(|i| i.to_string())
    }

    /// Get timestamp expiration, if set
    pub fn expiration(&self) -> Option<JsTimestamp> {
        self.inner.expiration().copied().map(|t| t.into())
    }

    /// Extract public keys from `p` tags.
    ///
    /// This method extract ONLY supported standard variants
    #[wasm_bindgen(js_name = publicKeys)]
    pub fn public_keys(&self) -> Vec<JsPublicKey> {
        self.inner
            .public_keys()
            .copied()
            .map(|p| p.into())
            .collect()
    }

    /// Extract event IDs from `e` tags.
    ///
    /// This method extract ONLY supported standard variants
    #[wasm_bindgen(js_name = eventIds)]
    pub fn event_ids(&self) -> Vec<JsEventId> {
        self.inner.event_ids().copied().map(|p| p.into()).collect()
    }

    /// Extract coordinates from `a` tags.
    ///
    /// This method extract ONLY supported standard variants
    pub fn coordinates(&self) -> Vec<JsCoordinate> {
        self.inner
            .coordinates()
            .cloned()
            .map(|p| p.into())
            .collect()
    }

    /// Extract hashtags from `t` tags.
    ///
    /// This method extract ONLY supported standard variants
    pub fn hashtags(&self) -> Vec<String> {
        self.inner.hashtags().map(|t| t.to_owned()).collect()
    }
}
