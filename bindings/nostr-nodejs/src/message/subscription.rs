// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use napi::bindgen_prelude::BigInt;
use nostr::prelude::*;

use crate::{event::id::JsEventId, key::JsPublicKey};

#[napi(js_name = "SubscriptionId")]
pub struct JsSubscriptionId {
    inner: SubscriptionId,
}

#[napi]
impl JsSubscriptionId {
    #[napi(constructor)]
    pub fn new(id: String) -> Self {
        Self {
            inner: SubscriptionId::new(id),
        }
    }

    /// Generate new random [`SubscriptionId`]
    #[napi(factory)]
    pub fn generate() -> Self {
        Self {
            inner: SubscriptionId::generate(),
        }
    }

    #[napi(getter)]
    pub fn get(&self) -> String {
        self.inner.to_string()
    }
}

#[napi(js_name = "SubscriptionFilter")]
pub struct JsSubscriptionFilter {
    inner: SubscriptionFilter,
}

#[napi]
impl JsSubscriptionFilter {
    #[allow(clippy::new_without_default)]
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SubscriptionFilter::new(),
        }
    }

    /// Set subscription id
    #[napi]
    pub fn id(&self, id: String) -> Self {
        Self {
            inner: self.inner.to_owned().id(id),
        }
    }

    /// Set subscription ids
    #[napi]
    pub fn ids(&self, ids: Vec<String>) -> Self {
        Self {
            inner: self.inner.to_owned().ids(ids),
        }
    }

    /// Set author
    #[napi]
    pub fn author(&self, author: &JsPublicKey) -> Self {
        Self {
            inner: self.inner.to_owned().author(author.into()),
        }
    }

    /// Set authors
    #[napi]
    pub fn authors(&self, authors: Vec<&JsPublicKey>) -> Self {
        let authors = authors.into_iter().map(|a| a.into()).collect();
        Self {
            inner: self.inner.to_owned().authors(authors),
        }
    }

    /// Set kind
    #[napi]
    pub fn kind(&self, kind: BigInt) -> Self {
        let kind: u64 = kind.get_u64().1;
        Self {
            inner: self.inner.to_owned().kind(Kind::from(kind)),
        }
    }

    /// Set kinds
    #[napi]
    pub fn kinds(&self, kinds: Vec<BigInt>) -> Self {
        let kinds: Vec<Kind> = kinds
            .into_iter()
            .map(|k| Kind::from(k.get_u64().1))
            .collect();
        Self {
            inner: self.inner.to_owned().kinds(kinds),
        }
    }

    /// Set event
    #[napi]
    pub fn event(&self, id: &JsEventId) -> Self {
        Self {
            inner: self.inner.to_owned().event(id.into()),
        }
    }

    /// Set events
    #[napi]
    pub fn events(&self, ids: Vec<&JsEventId>) -> Self {
        let events = ids.into_iter().map(|a| a.into()).collect();
        Self {
            inner: self.inner.to_owned().events(events),
        }
    }

    /// Set pubkey
    #[napi]
    pub fn pubkey(&self, pubkey: &JsPublicKey) -> Self {
        Self {
            inner: self.inner.to_owned().pubkey(pubkey.into()),
        }
    }

    /// Set pubkeys
    #[napi]
    pub fn pubkeys(&self, pubkeys: Vec<&JsPublicKey>) -> Self {
        let pubkeys = pubkeys.into_iter().map(|a| a.into()).collect();
        Self {
            inner: self.inner.to_owned().pubkeys(pubkeys),
        }
    }

    /// Set hashtag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[napi]
    pub fn hashtag(&self, hashtag: String) -> Self {
        Self {
            inner: self.inner.to_owned().hashtag(hashtag),
        }
    }

    /// Set hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[napi]
    pub fn hashtags(&self, hashtags: Vec<String>) -> Self {
        Self {
            inner: self.inner.to_owned().hashtags(hashtags),
        }
    }

    /// Set reference
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[napi]
    pub fn reference(&self, v: String) -> Self {
        Self {
            inner: self.inner.to_owned().reference(v),
        }
    }

    /// Set references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[napi]
    pub fn references(&self, v: Vec<String>) -> Self {
        Self {
            inner: self.inner.to_owned().references(v),
        }
    }

    /// Set search field
    #[napi]
    pub fn search(&self, value: String) -> Self {
        Self {
            inner: self.inner.to_owned().search(value),
        }
    }

    /// Set since unix timestamp
    #[napi]
    pub fn since(&self, since: BigInt) -> Self {
        let since = Timestamp::from(since.get_u64().1);
        Self {
            inner: self.inner.to_owned().since(since),
        }
    }

    /// Set until unix timestamp
    #[napi]
    pub fn until(&self, until: BigInt) -> Self {
        let until = Timestamp::from(until.get_u64().1);
        Self {
            inner: self.inner.to_owned().until(until),
        }
    }

    /// Set limit
    #[napi]
    pub fn limit(&self, limit: BigInt) -> Self {
        let limit: u64 = limit.get_u64().1;
        Self {
            inner: self.inner.to_owned().limit(limit as usize),
        }
    }
}
