// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use js_sys::{Array, JsString};
use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::JsEventId;
use crate::key::JsPublicKey;
use crate::util;

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

impl From<&JsFilter> for Filter {
    fn from(filter: &JsFilter) -> Self {
        filter.inner.clone()
    }
}

#[wasm_bindgen(js_class = Filter)]
impl JsFilter {
    #[allow(clippy::new_without_default)]
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
    pub fn id(&self, id: String) -> Self {
        Self {
            inner: self.inner.to_owned().id(id),
        }
    }

    /// Set subscription ids
    #[wasm_bindgen]
    pub fn ids(&self, ids: Vec<JsString>) -> Result<JsFilter> {
        let ids: Vec<String> = ids.iter().filter_map(|id| id.as_string()).collect();
        Ok(Self {
            inner: self.inner.to_owned().ids(ids),
        })
    }

    /// Set author
    #[wasm_bindgen]
    pub fn author(&self, author: String) -> Self {
        Self {
            inner: self.inner.to_owned().author(author),
        }
    }

    /// Set authors
    #[wasm_bindgen]
    pub fn authors(&self, authors: Vec<JsString>) -> Result<JsFilter> {
        let authors: Vec<String> = authors.iter().filter_map(|a| a.as_string()).collect();
        Ok(Self {
            inner: self.inner.to_owned().authors(authors),
        })
    }

    /// Set kind
    #[wasm_bindgen]
    pub fn kind(&self, kind: u64) -> Self {
        Self {
            inner: self.inner.to_owned().kind(Kind::from(kind)),
        }
    }

    /// Set kinds
    #[wasm_bindgen]
    pub fn kinds(&self, kinds: Vec<u64>) -> Self {
        let kinds: Vec<Kind> = kinds.into_iter().map(Kind::from).collect();
        Self {
            inner: self.inner.to_owned().kinds(kinds),
        }
    }

    /// Set event
    #[wasm_bindgen]
    pub fn event(&self, id: &JsEventId) -> Self {
        Self {
            inner: self.inner.to_owned().event(id.into()),
        }
    }

    /// Set events
    #[wasm_bindgen]
    pub fn events(&self, ids: Array) -> Result<JsFilter> {
        let events = ids
            .iter()
            .map(|v| Ok(util::downcast::<JsEventId>(&v, "EventId")?.inner))
            .collect::<Result<Vec<EventId>, JsError>>()?;
        Ok(Self {
            inner: self.inner.to_owned().events(events),
        })
    }

    /// Set pubkey
    #[wasm_bindgen]
    pub fn pubkey(&self, pubkey: &JsPublicKey) -> Self {
        Self {
            inner: self.inner.to_owned().pubkey(pubkey.into()),
        }
    }

    /// Set pubkeys
    #[wasm_bindgen]
    pub fn pubkeys(&self, pubkeys: Array) -> Result<JsFilter> {
        let pubkeys = pubkeys
            .iter()
            .map(|v| Ok(util::downcast::<JsPublicKey>(&v, "PublicKey")?.inner))
            .collect::<Result<Vec<XOnlyPublicKey>, JsError>>()?;
        Ok(Self {
            inner: self.inner.to_owned().pubkeys(pubkeys),
        })
    }

    /// Set hashtag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen]
    pub fn hashtag(&self, hashtag: String) -> Self {
        Self {
            inner: self.inner.to_owned().hashtag(hashtag),
        }
    }

    /// Set hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen]
    pub fn hashtags(&self, hashtags: Vec<JsString>) -> Self {
        let hashtags: Vec<String> = hashtags.iter().filter_map(|t| t.as_string()).collect();
        Self {
            inner: self.inner.to_owned().hashtags(hashtags),
        }
    }

    /// Set reference
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen]
    pub fn reference(&self, v: String) -> Self {
        Self {
            inner: self.inner.to_owned().reference(v),
        }
    }

    /// Set references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen]
    pub fn references(&self, v: Vec<JsString>) -> Result<JsFilter> {
        let v: Vec<String> = v.iter().filter_map(|v| v.as_string()).collect();
        Ok(Self {
            inner: self.inner.to_owned().references(v),
        })
    }

    /// Set search field
    #[wasm_bindgen]
    pub fn search(&self, value: String) -> Self {
        Self {
            inner: self.inner.to_owned().search(value),
        }
    }

    /// Set since unix timestamp
    #[wasm_bindgen]
    pub fn since(&self, since: u64) -> Self {
        let since = Timestamp::from(since);
        Self {
            inner: self.inner.to_owned().since(since),
        }
    }

    /// Set until unix timestamp
    #[wasm_bindgen]
    pub fn until(&self, until: u64) -> Self {
        let until = Timestamp::from(until);
        Self {
            inner: self.inner.to_owned().until(until),
        }
    }

    /// Set limit
    #[wasm_bindgen]
    pub fn limit(&self, limit: u64) -> Self {
        Self {
            inner: self.inner.to_owned().limit(limit as usize),
        }
    }
}

impl JsFilter {
    pub fn inner(&self) -> Filter {
        self.inner.clone()
    }
}
