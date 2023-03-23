// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Subscription filters

#![allow(missing_docs)]
use core::fmt;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::{String, ToString};
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{vec, vec::Vec};

use bitcoin_hashes::sha256::Hash as Sha256Hash;
use bitcoin_hashes::Hash;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use rand::{rngs::OsRng, RngCore};
#[cfg(feature = "std")]
use secp256k1::rand::{rngs::OsRng, RngCore};
use secp256k1::XOnlyPublicKey;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{EventId, Kind, Timestamp};

/// Subscription ID
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(String);

impl SubscriptionId {
    /// Create new [`SubscriptionId`]
    pub fn new<S>(id: S) -> Self
    where
        S: Into<String>,
    {
        Self(id.into())
    }

    /// Generate new random [`SubscriptionId`]
    pub fn generate() -> Self {
        let mut os_random = [0u8; 32];
        OsRng.fill_bytes(&mut os_random);
        let hash = Sha256Hash::hash(&os_random).to_string();
        Self::new(&hash[..32])
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}

/// Subscription filters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    /// List of event ids or prefixes
    pub ids: Option<Vec<String>>,
    /// List of pubkeys or prefixes
    pub authors: Option<Vec<String>>,
    /// List of a kind numbers
    pub kinds: Option<Vec<Kind>>,
    /// #e tag
    pub events: Option<Vec<EventId>>,
    /// #p tag
    pub pubkeys: Option<Vec<XOnlyPublicKey>>,
    /// #t tag
    pub hashtags: Option<Vec<String>>,
    /// #r tag
    pub references: Option<Vec<String>>,
    /// #d tag
    pub identifiers: Option<Vec<String>>,
    /// It's a string describing a query in a human-readable form, i.e. "best nostr apps"
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/50.md>
    pub search: Option<String>,
    /// An integer unix timestamp, events must be newer than this to pass
    pub since: Option<Timestamp>,
    /// An integer unix timestamp, events must be older than this to pass
    pub until: Option<Timestamp>,
    /// Maximum number of events to be returned in the initial query
    pub limit: Option<usize>,
    /// Custom fields
    pub custom: Map<String, Value>,
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter {
    /// Create new empty [`Filter`]
    pub fn new() -> Self {
        Self {
            ids: None,
            authors: None,
            kinds: None,
            events: None,
            pubkeys: None,
            hashtags: None,
            references: None,
            identifiers: None,
            search: None,
            since: None,
            until: None,
            limit: None,
            custom: Map::new(),
        }
    }

    /// Deserialize from `JSON` string
    pub fn from_json<S>(json: S) -> Result<Self, serde_json::Error>
    where
        S: Into<String>,
    {
        serde_json::from_str(&json.into())
    }

    /// Serialize to `JSON` string
    pub fn as_json(&self) -> String {
        json!(self).to_string()
    }

    /// Set event id or prefix
    pub fn id<S>(self, id: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            ids: Some(vec![id.into()]),
            ..self
        }
    }

    /// Set event ids or prefixes
    pub fn ids<S>(self, ids: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            ids: Some(ids.into_iter().map(|id| id.into()).collect()),
            ..self
        }
    }

    /// Set author
    pub fn author<S>(self, author: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            authors: Some(vec![author.into()]),
            ..self
        }
    }

    /// Set authors
    pub fn authors<S>(self, authors: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            authors: Some(authors.into_iter().map(|a| a.into()).collect()),
            ..self
        }
    }

    /// Set kind
    pub fn kind(self, kind: Kind) -> Self {
        Self {
            kinds: Some(vec![kind]),
            ..self
        }
    }

    /// Set kinds
    pub fn kinds(self, kinds: Vec<Kind>) -> Self {
        Self {
            kinds: Some(kinds),
            ..self
        }
    }

    /// Set event
    pub fn event(self, id: EventId) -> Self {
        Self {
            events: Some(vec![id]),
            ..self
        }
    }

    /// Set events
    pub fn events(self, ids: Vec<EventId>) -> Self {
        Self {
            events: Some(ids),
            ..self
        }
    }

    /// Set pubkey
    pub fn pubkey(self, pubkey: XOnlyPublicKey) -> Self {
        Self {
            pubkeys: Some(vec![pubkey]),
            ..self
        }
    }

    /// Set pubkeys
    pub fn pubkeys(self, pubkeys: Vec<XOnlyPublicKey>) -> Self {
        Self {
            pubkeys: Some(pubkeys),
            ..self
        }
    }

    /// Set hashtag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtag<S>(self, hashtag: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            hashtags: Some(vec![hashtag.into()]),
            ..self
        }
    }

    /// Set hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtags<S>(self, hashtags: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            hashtags: Some(hashtags.into_iter().map(|a| a.into()).collect()),
            ..self
        }
    }

    /// Set reference
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn reference<S>(self, v: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            references: Some(vec![v.into()]),
            ..self
        }
    }

    /// Set references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn references<S>(self, v: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            references: Some(v.into_iter().map(|a| a.into()).collect()),
            ..self
        }
    }

    /// Set identifier
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/33.md>
    pub fn identifier<S>(self, identifier: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            identifiers: Some(vec![identifier.into()]),
            ..self
        }
    }

    /// Set identifiers
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/33.md>
    pub fn identifiers<S>(self, identifiers: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            identifiers: Some(identifiers.into_iter().map(|a| a.into()).collect()),
            ..self
        }
    }

    /// Set search field
    pub fn search<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            search: Some(value.into()),
            ..self
        }
    }

    /// Set since unix timestamp
    pub fn since(self, since: Timestamp) -> Self {
        Self {
            since: Some(since),
            ..self
        }
    }

    /// Set until unix timestamp
    pub fn until(self, until: Timestamp) -> Self {
        Self {
            until: Some(until),
            ..self
        }
    }

    /// Set limit
    pub fn limit(self, limit: usize) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }

    /// Set custom filters
    pub fn custom(self, map: Map<String, Value>) -> Self {
        Self {
            custom: map,
            ..self
        }
    }
}

impl Serialize for Filter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len: usize = 11 + self.custom.len();
        let mut map = serializer.serialize_map(Some(len))?;
        if let Some(value) = &self.ids {
            map.serialize_entry("ids", &json!(value))?;
        }
        if let Some(value) = &self.kinds {
            map.serialize_entry("kinds", &json!(value))?;
        }
        if let Some(value) = &self.authors {
            map.serialize_entry("authors", &json!(value))?;
        }
        if let Some(value) = &self.events {
            map.serialize_entry("#e", &json!(value))?;
        }
        if let Some(value) = &self.pubkeys {
            map.serialize_entry("#p", &json!(value))?;
        }
        if let Some(value) = &self.hashtags {
            map.serialize_entry("#t", &json!(value))?;
        }
        if let Some(value) = &self.references {
            map.serialize_entry("#r", &json!(value))?;
        }
        if let Some(value) = &self.identifiers {
            map.serialize_entry("#d", &json!(value))?;
        }
        if let Some(value) = &self.search {
            map.serialize_entry("search", &json!(value))?;
        }
        if let Some(value) = &self.since {
            map.serialize_entry("since", &json!(value))?;
        }
        if let Some(value) = &self.until {
            map.serialize_entry("until", &json!(value))?;
        }
        if let Some(value) = &self.limit {
            map.serialize_entry("limit", &json!(value))?;
        }
        for (k, v) in &self.custom {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Filter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FilterVisitor)
    }
}

struct FilterVisitor;

impl<'de> Visitor<'de> for FilterVisitor {
    type Value = Filter;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A JSON object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Filter, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: Map<String, Value> = Map::new();
        while let Some((key, value)) = access.next_entry::<String, Value>()? {
            let _ = map.insert(key, value);
        }

        let mut f: Filter = Filter::new();

        if let Some(value) = map.remove("ids") {
            let ids: Vec<String> = serde_json::from_value(value).map_err(de::Error::custom)?;
            f.ids = Some(ids);
        }

        if let Some(value) = map.remove("authors") {
            let authors: Vec<String> = serde_json::from_value(value).map_err(de::Error::custom)?;
            f.authors = Some(authors);
        }

        if let Some(value) = map.remove("kinds") {
            let kinds: Vec<Kind> = serde_json::from_value(value).map_err(de::Error::custom)?;
            f.kinds = Some(kinds);
        }

        if let Some(value) = map.remove("#e") {
            let events: Vec<EventId> = serde_json::from_value(value).map_err(de::Error::custom)?;
            f.events = Some(events);
        }

        if let Some(value) = map.remove("#p") {
            let pubkeys: Vec<XOnlyPublicKey> =
                serde_json::from_value(value).map_err(de::Error::custom)?;
            f.pubkeys = Some(pubkeys);
        }

        if let Some(value) = map.remove("#t") {
            let hashtags: Vec<String> = serde_json::from_value(value).map_err(de::Error::custom)?;
            f.hashtags = Some(hashtags);
        }

        if let Some(value) = map.remove("#r") {
            let references: Vec<String> =
                serde_json::from_value(value).map_err(de::Error::custom)?;
            f.references = Some(references);
        }

        if let Some(value) = map.remove("#d") {
            let identifiers: Vec<String> =
                serde_json::from_value(value).map_err(de::Error::custom)?;
            f.identifiers = Some(identifiers);
        }

        if let Some(Value::String(search)) = map.remove("search") {
            f.search = Some(search);
        }

        if let Some(value) = map.remove("since") {
            let since: Timestamp = serde_json::from_value(value).map_err(de::Error::custom)?;
            f.since = Some(since);
        }

        if let Some(value) = map.remove("until") {
            let until: Timestamp = serde_json::from_value(value).map_err(de::Error::custom)?;
            f.until = Some(until);
        }

        if let Some(value) = map.remove("limit") {
            let limit: usize = serde_json::from_value(value).map_err(de::Error::custom)?;
            f.limit = Some(limit);
        }

        f.custom = map;

        Ok(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_filter_serialization() {
        let mut custom = Map::new();
        custom.insert(
            "#a".to_string(),
            Value::Array(vec![Value::String("...".to_string())]),
        );
        let filter = Filter::new()
            .identifier("identifier")
            .search("test")
            .custom(custom);
        let json = r##"{"#a":["..."],"#d":["identifier"],"search":"test"}"##;
        assert_eq!(filter.as_json(), json.to_string());
    }

    #[test]
    fn test_filter_deserialization() {
        let json = r##"{"#a":["..."],"search":"test","ids":["myid", "mysecondid"]}"##;
        let filter = Filter::from_json(json).unwrap();
        let mut custom = Map::new();
        custom.insert(
            "#a".to_string(),
            Value::Array(vec![Value::String("...".to_string())]),
        );
        assert_eq!(
            filter,
            Filter::new()
                .ids(vec!["myid".to_string(), "mysecondid".to_string()])
                .search("test")
                .custom(custom)
        );
    }
}
