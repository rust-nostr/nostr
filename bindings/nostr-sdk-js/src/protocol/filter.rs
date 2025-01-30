// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::event::{JsEvent, JsEventId, JsKind};
use crate::protocol::key::JsPublicKey;
use crate::protocol::nips::nip01::JsCoordinate;
use crate::protocol::types::JsTimestamp;

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

    #[wasm_bindgen(js_name = asPrettyJson)]
    pub fn as_pretty_json(&self) -> Result<String> {
        self.inner.try_as_pretty_json().map_err(into_err)
    }

    /// Set event ID
    pub fn id(self, id: &JsEventId) -> Self {
        self.inner.id(**id).into()
    }

    /// Set event IDs
    pub fn ids(self, ids: Vec<JsEventId>) -> Self {
        self.inner.ids(ids.into_iter().map(|id| id.into())).into()
    }

    /// Remove event IDs
    #[wasm_bindgen(js_name = removeIds)]
    pub fn remove_ids(self, ids: Vec<JsEventId>) -> Self {
        self.inner
            .remove_ids(ids.into_iter().map(|id| id.into()))
            .into()
    }

    /// Set author
    pub fn author(self, author: &JsPublicKey) -> Self {
        self.inner.author(**author).into()
    }

    /// Set authors
    pub fn authors(self, authors: Vec<JsPublicKey>) -> Self {
        self.inner
            .authors(authors.into_iter().map(|p| p.into()))
            .into()
    }

    /// Remove authors
    #[wasm_bindgen(js_name = removeAuthors)]
    pub fn remove_authors(self, authors: Vec<JsPublicKey>) -> Self {
        self.inner
            .remove_authors(authors.into_iter().map(|p| p.into()))
            .into()
    }

    /// Set kind
    pub fn kind(self, kind: &JsKind) -> Self {
        self.inner.kind(**kind).into()
    }

    /// Set kinds
    pub fn kinds(self, kinds: Vec<JsKind>) -> Self {
        self.inner.kinds(kinds.into_iter().map(|k| *k)).into()
    }

    /// Remove kinds
    #[wasm_bindgen(js_name = removeKinds)]
    pub fn remove_kinds(self, kinds: Vec<JsKind>) -> Self {
        self.inner
            .remove_kinds(kinds.into_iter().map(|k| *k))
            .into()
    }

    /// Set event
    pub fn event(self, id: &JsEventId) -> Self {
        self.inner.event(**id).into()
    }

    /// Set events
    pub fn events(self, ids: Vec<JsEventId>) -> Self {
        self.inner
            .events(ids.into_iter().map(|id| id.into()))
            .into()
    }

    /// Remove events
    #[wasm_bindgen(js_name = removeEvents)]
    pub fn remove_events(self, ids: Vec<JsEventId>) -> Self {
        self.inner
            .remove_events(ids.into_iter().map(|id| id.into()))
            .into()
    }

    /// Set pubkey
    pub fn pubkey(self, pubkey: &JsPublicKey) -> Self {
        self.inner.pubkey(**pubkey).into()
    }

    /// Set pubkeys
    pub fn pubkeys(self, pubkeys: Vec<JsPublicKey>) -> Self {
        self.inner
            .pubkeys(pubkeys.into_iter().map(|p| p.into()))
            .into()
    }

    /// Remove pubkeys
    #[wasm_bindgen(js_name = removePubkeys)]
    pub fn remove_pubkeys(self, pubkeys: Vec<JsPublicKey>) -> Self {
        self.inner
            .remove_pubkeys(pubkeys.into_iter().map(|p| p.into()))
            .into()
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

    /// Remove hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen(js_name = removeHashtags)]
    pub fn remove_hashtags(self, hashtags: Vec<String>) -> Self {
        self.inner.remove_hashtags(hashtags).into()
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

    /// Remove references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[wasm_bindgen(js_name = removeReferences)]
    pub fn remove_references(self, v: Vec<String>) -> Self {
        self.inner.remove_references(v).into()
    }

    /// Add identifier
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn identifier(self, identifier: &str) -> Self {
        self.inner.identifier(identifier).into()
    }

    /// Set identifiers
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn identifiers(self, identifiers: Vec<String>) -> Self {
        self.inner.identifiers(identifiers).into()
    }

    /// Remove identifiers
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = removeIdentifiers)]
    pub fn remove_identifiers(self, identifiers: Vec<String>) -> Self {
        self.inner.remove_identifiers(identifiers).into()
    }

    /// Add coordinate
    ///
    /// Query for `a` tag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn coordinate(self, coordinate: &JsCoordinate) -> Self {
        self.inner.coordinate(coordinate.deref()).into()
    }

    /// Set coordinates
    ///
    /// Query for `a` tags.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn coordinates(self, coordinates: Vec<JsCoordinate>) -> Self {
        self.inner
            .coordinates(coordinates.iter().map(|c| c.deref()))
            .into()
    }

    /// Remove coordinates
    ///
    /// Remove `a` tags.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = removeCoordinates)]
    pub fn remove_coordinates(self, coordinates: Vec<JsCoordinate>) -> Self {
        self.inner
            .remove_coordinates(coordinates.iter().map(|c| c.deref()))
            .into()
    }

    /// Set search field
    pub fn search(self, value: &str) -> Self {
        self.inner.search(value).into()
    }

    /// Remove search
    #[wasm_bindgen(js_name = removeSearch)]
    pub fn remove_search(self) -> Self {
        self.inner.remove_search().into()
    }

    /// Set since unix timestamp
    pub fn since(self, since: &JsTimestamp) -> Self {
        self.inner.since(**since).into()
    }

    /// Remove since
    #[wasm_bindgen(js_name = removeSince)]
    pub fn remove_since(self) -> Self {
        self.inner.remove_since().into()
    }

    /// Set until unix timestamp
    pub fn until(self, until: &JsTimestamp) -> Self {
        self.inner.until(**until).into()
    }

    /// Remove until
    #[wasm_bindgen(js_name = removeUntil)]
    pub fn remove_until(self) -> Self {
        self.inner.remove_until().into()
    }

    /// Set limit
    pub fn limit(self, limit: f64) -> Self {
        self.inner.limit(limit as usize).into()
    }

    /// Remove limit
    #[wasm_bindgen(js_name = removeLimit)]
    pub fn remove_limit(self) -> Self {
        self.inner.remove_limit().into()
    }

    #[wasm_bindgen(js_name = customTag)]
    pub fn custom_tag(self, tag: &JsSingleLetterTag, value: String) -> Self {
        self.inner.custom_tag(**tag, value).into()
    }

    #[wasm_bindgen(js_name = customTags)]
    pub fn custom_tags(self, tag: &JsSingleLetterTag, values: Vec<String>) -> Self {
        self.inner.custom_tags(**tag, values).into()
    }

    #[wasm_bindgen(js_name = removeCustomTags)]
    pub fn remove_custom_tags(self, tag: &JsSingleLetterTag, values: Vec<String>) -> Self {
        self.inner.remove_custom_tags(**tag, values).into()
    }

    /// Check if `Filter` is empty
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Determine if `Filter` match given `Event`.
    #[inline]
    #[wasm_bindgen(js_name = matchEvent)]
    pub fn match_event(&self, event: &JsEvent) -> bool {
        self.inner.match_event(event.deref())
    }
}
