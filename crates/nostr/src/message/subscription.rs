// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Subscription filters

use core::fmt;

use bitcoin_hashes::sha256::Hash as Sha256Hash;
use bitcoin_hashes::Hash;
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::RngCore;
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

    /// Add event id or prefix
    pub fn id<S>(self, id: S) -> Self
    where
        S: Into<String>,
    {
        let id: String = id.into();
        Self {
            ids: Some(self.ids.map_or(vec![id.clone()], |mut ids| {
                if !ids.contains(&id) {
                    ids.push(id);
                }
                ids
            })),
            ..self
        }
    }

    /// Add event ids or prefixes
    pub fn ids<S>(self, ids: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_ids: Vec<String> = self.ids.unwrap_or_default();
        for value in ids.into_iter().map(|value| value.into()) {
            if !current_ids.contains(&value) {
                current_ids.push(value);
            }
        }
        Self {
            ids: Some(current_ids),
            ..self
        }
    }

    /// Add author
    pub fn author<S>(self, author: S) -> Self
    where
        S: Into<String>,
    {
        let author: String = author.into();
        Self {
            authors: Some(self.authors.map_or(vec![author.clone()], |mut authors| {
                if !authors.contains(&author) {
                    authors.push(author);
                }
                authors
            })),
            ..self
        }
    }

    /// Add authors
    pub fn authors<S>(self, authors: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_authors: Vec<String> = self.authors.unwrap_or_default();
        for value in authors.into_iter().map(|value| value.into()) {
            if !current_authors.contains(&value) {
                current_authors.push(value);
            }
        }
        Self {
            authors: Some(current_authors),
            ..self
        }
    }

    /// Add kind
    pub fn kind(self, kind: Kind) -> Self {
        Self {
            kinds: Some(self.kinds.map_or(vec![kind], |mut kinds| {
                if !kinds.contains(&kind) {
                    kinds.push(kind);
                }
                kinds
            })),
            ..self
        }
    }

    /// Add kinds
    pub fn kinds(self, kinds: Vec<Kind>) -> Self {
        let mut current_kinds: Vec<Kind> = self.kinds.unwrap_or_default();
        for value in kinds.into_iter() {
            if !current_kinds.contains(&value) {
                current_kinds.push(value);
            }
        }
        Self {
            kinds: Some(current_kinds),
            ..self
        }
    }

    /// Add event
    pub fn event(self, id: EventId) -> Self {
        Self {
            events: Some(self.events.map_or(vec![id], |mut events| {
                if !events.contains(&id) {
                    events.push(id);
                }
                events
            })),
            ..self
        }
    }

    /// Add events
    pub fn events(self, events: Vec<EventId>) -> Self {
        let mut current_events: Vec<EventId> = self.events.unwrap_or_default();
        for value in events.into_iter() {
            if !current_events.contains(&value) {
                current_events.push(value);
            }
        }
        Self {
            events: Some(current_events),
            ..self
        }
    }

    /// Add pubkey
    pub fn pubkey(self, pubkey: XOnlyPublicKey) -> Self {
        Self {
            pubkeys: Some(self.pubkeys.map_or(vec![pubkey], |mut pubkeys| {
                if !pubkeys.contains(&pubkey) {
                    pubkeys.push(pubkey);
                }
                pubkeys
            })),
            ..self
        }
    }

    /// Add pubkeys
    pub fn pubkeys(self, pubkeys: Vec<XOnlyPublicKey>) -> Self {
        let mut current_pubkeys: Vec<XOnlyPublicKey> = self.pubkeys.unwrap_or_default();
        for value in pubkeys.into_iter() {
            if !current_pubkeys.contains(&value) {
                current_pubkeys.push(value);
            }
        }
        Self {
            pubkeys: Some(current_pubkeys),
            ..self
        }
    }

    /// Add hashtag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtag<S>(self, hashtag: S) -> Self
    where
        S: Into<String>,
    {
        let hashtag: String = hashtag.into();
        Self {
            hashtags: Some(self.hashtags.map_or(vec![hashtag.clone()], |mut hashtags| {
                if !hashtags.contains(&hashtag) {
                    hashtags.push(hashtag);
                }
                hashtags
            })),
            ..self
        }
    }

    /// Add hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtags<S>(self, hashtags: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_hashtags: Vec<String> = self.hashtags.unwrap_or_default();
        for value in hashtags.into_iter().map(|value| value.into()) {
            if !current_hashtags.contains(&value) {
                current_hashtags.push(value);
            }
        }
        Self {
            hashtags: Some(current_hashtags),
            ..self
        }
    }

    /// Add reference
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn reference<S>(self, reference: S) -> Self
    where
        S: Into<String>,
    {
        let reference: String = reference.into();
        Self {
            references: Some(
                self.references
                    .map_or(vec![reference.clone()], |mut references| {
                        if !references.contains(&reference) {
                            references.push(reference);
                        }
                        references
                    }),
            ),
            ..self
        }
    }

    /// Add references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn references<S>(self, references: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_references: Vec<String> = self.references.unwrap_or_default();
        for value in references.into_iter().map(|value| value.into()) {
            if !current_references.contains(&value) {
                current_references.push(value);
            }
        }
        Self {
            references: Some(current_references),
            ..self
        }
    }

    /// Add identifier
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/33.md>
    pub fn identifier<S>(self, identifier: S) -> Self
    where
        S: Into<String>,
    {
        let identifier: String = identifier.into();
        Self {
            identifiers: Some(self.identifiers.map_or(
                vec![identifier.clone()],
                |mut identifiers| {
                    if !identifiers.contains(&identifier) {
                        identifiers.push(identifier);
                    }
                    identifiers
                },
            )),
            ..self
        }
    }

    /// Add identifiers
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/33.md>
    pub fn identifiers<S>(self, identifiers: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_identifiers: Vec<String> = self.identifiers.unwrap_or_default();
        for value in identifiers.into_iter().map(|value| value.into()) {
            if !current_identifiers.contains(&value) {
                current_identifiers.push(value);
            }
        }
        Self {
            identifiers: Some(current_identifiers),
            ..self
        }
    }

    /// Add search field
    pub fn search<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            search: Some(value.into()),
            ..self
        }
    }

    /// Add since unix timestamp
    pub fn since(self, since: Timestamp) -> Self {
        Self {
            since: Some(since),
            ..self
        }
    }

    /// Add until unix timestamp
    pub fn until(self, until: Timestamp) -> Self {
        Self {
            until: Some(until),
            ..self
        }
    }

    /// Add limit
    pub fn limit(self, limit: usize) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }

    /// Add custom filters
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
    fn test_kind_concatenation() {
        let filter = Filter::new()
            .kind(Kind::Metadata)
            .kind(Kind::TextNote)
            .kind(Kind::ContactList)
            .kinds(vec![
                Kind::EncryptedDirectMessage,
                Kind::Metadata,
                Kind::LongFormTextNote,
            ]);
        assert_eq!(
            filter,
            Filter::new().kinds(vec![
                Kind::Metadata,
                Kind::TextNote,
                Kind::ContactList,
                Kind::EncryptedDirectMessage,
                Kind::LongFormTextNote
            ])
        );
    }

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
