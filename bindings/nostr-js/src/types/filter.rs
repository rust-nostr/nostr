// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::{JsEvent, JsEventId};
use crate::key::JsPublicKey;
use crate::types::JsTimestamp;

#[wasm_bindgen(js_name = Alphabet)]
pub enum JsAlphabet {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl From<JsAlphabet> for Alphabet {
    fn from(value: JsAlphabet) -> Self {
        match value {
            JsAlphabet::A => Self::A,
            JsAlphabet::B => Self::B,
            JsAlphabet::C => Self::C,
            JsAlphabet::D => Self::D,
            JsAlphabet::E => Self::E,
            JsAlphabet::F => Self::F,
            JsAlphabet::G => Self::G,
            JsAlphabet::H => Self::H,
            JsAlphabet::I => Self::I,
            JsAlphabet::J => Self::J,
            JsAlphabet::K => Self::K,
            JsAlphabet::L => Self::L,
            JsAlphabet::M => Self::M,
            JsAlphabet::N => Self::N,
            JsAlphabet::O => Self::O,
            JsAlphabet::P => Self::P,
            JsAlphabet::Q => Self::Q,
            JsAlphabet::R => Self::R,
            JsAlphabet::S => Self::S,
            JsAlphabet::T => Self::T,
            JsAlphabet::U => Self::U,
            JsAlphabet::V => Self::V,
            JsAlphabet::W => Self::W,
            JsAlphabet::X => Self::X,
            JsAlphabet::Y => Self::Y,
            JsAlphabet::Z => Self::Z,
        }
    }
}

#[wasm_bindgen(js_name = SingleLetterTag)]
pub struct JsSingleLetterTag {
    inner: SingleLetterTag,
}

impl Deref for JsSingleLetterTag {
    type Target = SingleLetterTag;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<SingleLetterTag> for JsSingleLetterTag {
    fn from(inner: SingleLetterTag) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = SingleLetterTag)]
impl JsSingleLetterTag {
    pub fn lowercase(character: JsAlphabet) -> Self {
        Self {
            inner: SingleLetterTag::lowercase(character.into()),
        }
    }

    pub fn uppercase(character: JsAlphabet) -> Self {
        Self {
            inner: SingleLetterTag::uppercase(character.into()),
        }
    }

    #[wasm_bindgen(js_name = isLowercase)]
    pub fn is_lowercase(&self) -> bool {
        self.inner.is_lowercase()
    }

    #[wasm_bindgen(js_name = isUppercase)]
    pub fn is_uppercase(&self) -> bool {
        self.inner.is_uppercase()
    }
}

#[wasm_bindgen(js_name = SubscriptionId)]
pub struct JsSubscriptionId {
    inner: SubscriptionId,
}

#[wasm_bindgen(js_class = SubscriptionId)]
impl JsSubscriptionId {
    #[wasm_bindgen(constructor)]
    pub fn new(id: &str) -> Self {
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
    pub fn from_json(json: &str) -> Result<JsFilter> {
        Ok(Self {
            inner: Filter::from_json(json).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> Result<String> {
        self.inner.try_as_json().map_err(into_err)
    }

    /// Set subscription id
    pub fn id(self, id: &JsEventId) -> Self {
        self.inner.id(**id).into()
    }

    /// Set subscription ids
    pub fn ids(self, ids: Vec<JsEventId>) -> Self {
        let ids = ids.into_iter().map(|id| id.into());
        self.inner.ids(ids).into()
    }

    /// Set author
    pub fn author(self, author: &JsPublicKey) -> Self {
        self.inner.author(**author).into()
    }

    /// Set authors
    pub fn authors(self, authors: Vec<JsPublicKey>) -> Self {
        let authors = authors.into_iter().map(|p| p.into());
        self.inner.authors(authors).into()
    }

    /// Set kind
    pub fn kind(self, kind: u16) -> Self {
        self.inner.kind(Kind::from(kind)).into()
    }

    /// Set kinds
    pub fn kinds(self, kinds: Vec<u16>) -> Self {
        let kinds = kinds.into_iter().map(Kind::from);
        self.inner.kinds(kinds).into()
    }

    /// Set event
    pub fn event(self, id: &JsEventId) -> Self {
        self.inner.event(**id).into()
    }

    /// Set events
    pub fn events(self, ids: Vec<JsEventId>) -> Self {
        let ids = ids.into_iter().map(|id| id.into());
        self.inner.events(ids).into()
    }

    /// Set pubkey
    pub fn pubkey(self, pubkey: &JsPublicKey) -> Self {
        self.inner.pubkey(**pubkey).into()
    }

    /// Set pubkeys
    pub fn pubkeys(self, pubkeys: Vec<JsPublicKey>) -> Self {
        let pubkeys = pubkeys.into_iter().map(|p| p.into());
        self.inner.pubkeys(pubkeys).into()
    }

    /// Set hashtag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtag(self, hashtag: &str) -> Self {
        self.inner.hashtag(hashtag).into()
    }

    /// Set hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtags(self, hashtags: Vec<String>) -> Self {
        self.inner.hashtags(hashtags).into()
    }

    /// Set reference
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn reference(self, v: &str) -> Self {
        self.inner.reference(v).into()
    }

    /// Set references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn references(self, v: Vec<String>) -> Self {
        self.inner.references(v).into()
    }

    /// Set search field
    pub fn search(self, value: &str) -> Self {
        self.inner.search(value).into()
    }

    /// Set since unix timestamp
    pub fn since(self, since: &JsTimestamp) -> Self {
        self.inner.since(**since).into()
    }

    /// Set until unix timestamp
    pub fn until(self, until: &JsTimestamp) -> Self {
        self.inner.until(**until).into()
    }

    /// Set limit
    pub fn limit(self, limit: f64) -> Self {
        self.inner.limit(limit as usize).into()
    }

    #[wasm_bindgen(js_name = customTag)]
    pub fn custom_tag(self, tag: &JsSingleLetterTag, values: Vec<String>) -> Self {
        self.inner.custom_tag(**tag, values).into()
    }

    #[wasm_bindgen(js_name = removeCustomTag)]
    pub fn remove_custom_tag(self, tag: &JsSingleLetterTag, values: Vec<String>) -> Self {
        self.inner.remove_custom_tag(**tag, values).into()
    }

    /// Determine if `Filter` match given `Event`.
    ///
    /// The `search` filed is not supported yet!
    #[wasm_bindgen(js_name = matchEvent)]
    pub fn match_event(&self, event: &JsEvent) -> bool {
        self.inner.match_event(event.deref())
    }
}
