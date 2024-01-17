// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use js_sys::JsString;
use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::JsEventId;
use crate::key::JsPublicKey;
use crate::types::JsTimestamp;

#[wasm_bindgen(js_name = SubscriptionId)]
pub struct JsSubscriptionId {
    inner: SubscriptionId,
}

#[wasm_bindgen(js_class = SubscriptionId)]
impl JsSubscriptionId {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String) -> Self {
        Self {
            inner: SubscriptionId::new(id),
        }
    }

    /// Generate new random [`SubscriptionId`]
    #[wasm_bindgen]
    pub fn generate() -> Self {
        Self {
            inner: SubscriptionId::generate(),
        }
    }

    #[wasm_bindgen]
    pub fn get(&self) -> String {
        self.inner.to_string()
    }
}

#[wasm_bindgen(js_name = Filter)]
pub struct JsFilter {
    inner: Filter,
}

impl Deref for JsFilter {
    type Target = Filter;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<JsFilter> for Filter {
    fn from(filter: JsFilter) -> Self {
        filter.inner
    }
}

impl From<Filter> for JsFilter {
    fn from(inner: Filter) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Filter)]
impl JsFilter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Filter::new(),
        }
    }

    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: String) -> Result<JsFilter> {
        Ok(Self {
            inner: Filter::from_json(json).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }

    /// Set subscription id
    #[wasm_bindgen]
    pub fn id(self, id: &JsEventId) -> Self {
        self.inner.id(**id).into()
    }

    /// Set subscription ids
    #[wasm_bindgen]
    pub fn ids(self, ids: Vec<JsEventId>) -> Self {
        let ids = ids.into_iter().map(|id| id.inner);
        self.inner.ids(ids).into()
    }

    /// Set author
    #[wasm_bindgen]
    pub fn author(self, author: &JsPublicKey) -> Self {
        self.inner.author(author.into()).into()
    }

    /// Set authors
    #[wasm_bindgen]
    pub fn authors(self, authors: Vec<JsPublicKey>) -> Self {
        let authors = authors.into_iter().map(|p| p.inner);
        self.inner.authors(authors).into()
    }

    /// Set kind
    #[wasm_bindgen]
    pub fn kind(self, kind: f64) -> Self {
        self.inner.kind(Kind::from(kind)).into()
    }

    /// Set kinds
    #[wasm_bindgen]
    pub fn kinds(self, kinds: Vec<f64>) -> Self {
        let kinds = kinds.into_iter().map(Kind::from);
        self.inner.kinds(kinds).into()
    }

    /// Set event
    #[wasm_bindgen]
    pub fn event(self, id: &JsEventId) -> Self {
        self.inner.event(id.into()).into()
    }

    /// Set events
    #[wasm_bindgen]
    pub fn events(self, ids: Vec<JsEventId>) -> Self {
        let ids = ids.into_iter().map(|id| id.inner);
        self.inner.events(ids).into()
    }

    /// Set pubkey
    #[wasm_bindgen]
    pub fn pubkey(self, pubkey: &JsPublicKey) -> Self {
        self.inner.pubkey(pubkey.into()).into()
    }

    /// Set pubkeys
    #[wasm_bindgen]
    pub fn pubkeys(self, pubkeys: Vec<JsPublicKey>) -> Self {
        let pubkeys = pubkeys.into_iter().map(|p| p.inner);
        self.inner.pubkeys(pubkeys).into()
    }

    /// Set hashtag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen]
    pub fn hashtag(self, hashtag: String) -> Self {
        self.inner.hashtag(hashtag).into()
    }

    /// Set hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen]
    pub fn hashtags(self, hashtags: Vec<JsString>) -> Self {
        let hashtags = hashtags.into_iter().filter_map(|t| t.as_string());
        self.inner.hashtags(hashtags).into()
    }

    /// Set reference
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen]
    pub fn reference(self, v: String) -> Self {
        self.inner.reference(v).into()
    }

    /// Set references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen]
    pub fn references(self, v: Vec<JsString>) -> Self {
        let v = v.into_iter().filter_map(|v| v.as_string());
        self.inner.references(v).into()
    }

    /// Set search field
    #[wasm_bindgen]
    pub fn search(self, value: String) -> Self {
        self.inner.search(value).into()
    }

    /// Set since unix timestamp
    #[wasm_bindgen]
    pub fn since(self, since: &JsTimestamp) -> Self {
        self.inner.since(**since).into()
    }

    /// Set until unix timestamp
    #[wasm_bindgen]
    pub fn until(self, until: &JsTimestamp) -> Self {
        self.inner.until(**until).into()
    }

    /// Set limit
    #[wasm_bindgen]
    pub fn limit(self, limit: f64) -> Self {
        self.inner.limit(limit as usize).into()
    }
}
