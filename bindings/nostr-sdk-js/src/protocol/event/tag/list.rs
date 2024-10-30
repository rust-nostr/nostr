// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::JsTag;
use crate::protocol::event::JsEventId;
use crate::protocol::key::JsPublicKey;
use crate::protocol::nips::nip01::JsCoordinate;
use crate::protocol::types::JsTimestamp;

#[wasm_bindgen(js_name = Tags)]
pub struct JsTags {
    inner: Tags,
}

impl From<Tags> for JsTags {
    fn from(inner: Tags) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Tags)]
impl JsTags {
    #[wasm_bindgen(constructor)]
    pub fn new(list: Vec<JsTag>) -> Self {
        Self {
            inner: Tags::new(list.into_iter().map(|t| t.inner).collect()),
        }
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
