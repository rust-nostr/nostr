// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Subscription filters

use core::fmt;
use std::collections::HashMap;

use bitcoin_hashes::sha256::Hash as Sha256Hash;
use bitcoin_hashes::Hash;
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::RngCore;
use secp256k1::XOnlyPublicKey;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use crate::{EventId, Kind, Timestamp};

/// Filter error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid Tag
    InvalidTag(char),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTag(c) => write!(f, "Invalid tag: {c}"),
        }
    }
}

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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    /// List of event ids or prefixes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub ids: Vec<String>,
    /// List of pubkeys or prefixes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub authors: Vec<String>,
    /// List of a kind numbers
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub kinds: Vec<Kind>,
    /// #e tag
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "#e")]
    #[serde(default)]
    pub events: Vec<EventId>,
    /// #p tag
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "#p")]
    #[serde(default)]
    pub pubkeys: Vec<XOnlyPublicKey>,
    /// #t tag
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "#t")]
    #[serde(default)]
    pub hashtags: Vec<String>,
    /// #r tag
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "#r")]
    #[serde(default)]
    pub references: Vec<String>,
    /// #d tag
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "#d")]
    #[serde(default)]
    pub identifiers: Vec<String>,
    /// It's a string describing a query in a human-readable form, i.e. "best nostr apps"
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/50.md>
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub search: Option<String>,
    /// An integer unix timestamp, events must be newer than this to pass
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub since: Option<Timestamp>,
    /// An integer unix timestamp, events must be older than this to pass
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub until: Option<Timestamp>,
    /// Maximum number of events to be returned in the initial query
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub limit: Option<usize>,
    /// Generic tag queries (NIP12)
    #[serde(
        flatten,
        serialize_with = "serialize_generic_tags",
        deserialize_with = "deserialize_generic_tags"
    )]
    #[serde(default)]
    pub generic_tags: HashMap<char, Vec<String>>,
}

impl Filter {
    /// Create new empty [`Filter`]
    pub fn new() -> Self {
        Self::default()
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
        serde_json::json!(self).to_string()
    }

    /// Add event id or prefix
    pub fn id<S>(self, id: S) -> Self
    where
        S: Into<String>,
    {
        let id: String = id.into();
        let mut ids: Vec<String> = self.ids;
        if !ids.contains(&id) {
            ids.push(id);
        }
        Self { ids, ..self }
    }

    /// Add event ids or prefixes
    pub fn ids<S>(self, ids: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_ids: Vec<String> = self.ids;
        for value in ids.into_iter().map(|value| value.into()) {
            if !current_ids.contains(&value) {
                current_ids.push(value);
            }
        }
        Self {
            ids: current_ids,
            ..self
        }
    }

    /// Add author
    pub fn author<S>(self, author: S) -> Self
    where
        S: Into<String>,
    {
        let author: String = author.into();
        let mut authors: Vec<String> = self.authors;
        if !authors.contains(&author) {
            authors.push(author);
        }
        Self { authors, ..self }
    }

    /// Add authors
    pub fn authors<S>(self, authors: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_authors: Vec<String> = self.authors;
        for value in authors.into_iter().map(|value| value.into()) {
            if !current_authors.contains(&value) {
                current_authors.push(value);
            }
        }
        Self {
            authors: current_authors,
            ..self
        }
    }

    /// Add kind
    pub fn kind(self, kind: Kind) -> Self {
        let mut kinds: Vec<Kind> = self.kinds;
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
        Self { kinds, ..self }
    }

    /// Add kinds
    pub fn kinds(self, kinds: Vec<Kind>) -> Self {
        let mut current_kinds: Vec<Kind> = self.kinds;
        for value in kinds.into_iter() {
            if !current_kinds.contains(&value) {
                current_kinds.push(value);
            }
        }
        Self {
            kinds: current_kinds,
            ..self
        }
    }

    /// Add event
    pub fn event(self, id: EventId) -> Self {
        let mut events: Vec<EventId> = self.events;
        if !events.contains(&id) {
            events.push(id);
        }
        Self { events, ..self }
    }

    /// Add events
    pub fn events(self, events: Vec<EventId>) -> Self {
        let mut current_events: Vec<EventId> = self.events;
        for value in events.into_iter() {
            if !current_events.contains(&value) {
                current_events.push(value);
            }
        }
        Self {
            events: current_events,
            ..self
        }
    }

    /// Add pubkey
    pub fn pubkey(self, pubkey: XOnlyPublicKey) -> Self {
        let mut pubkeys: Vec<XOnlyPublicKey> = self.pubkeys;
        if !pubkeys.contains(&pubkey) {
            pubkeys.push(pubkey);
        }
        Self { pubkeys, ..self }
    }

    /// Add pubkeys
    pub fn pubkeys(self, pubkeys: Vec<XOnlyPublicKey>) -> Self {
        let mut current_pubkeys: Vec<XOnlyPublicKey> = self.pubkeys;
        for value in pubkeys.into_iter() {
            if !current_pubkeys.contains(&value) {
                current_pubkeys.push(value);
            }
        }
        Self {
            pubkeys: current_pubkeys,
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
        let mut hashtags: Vec<String> = self.hashtags;
        if !hashtags.contains(&hashtag) {
            hashtags.push(hashtag);
        }
        Self { hashtags, ..self }
    }

    /// Add hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtags<S>(self, hashtags: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_hashtags: Vec<String> = self.hashtags;
        for value in hashtags.into_iter().map(|value| value.into()) {
            if !current_hashtags.contains(&value) {
                current_hashtags.push(value);
            }
        }
        Self {
            hashtags: current_hashtags,
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
        let mut references: Vec<String> = self.references;
        if !references.contains(&reference) {
            references.push(reference);
        }
        Self { references, ..self }
    }

    /// Add references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn references<S>(self, references: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let mut current_references: Vec<String> = self.references;
        for value in references.into_iter().map(|value| value.into()) {
            if !current_references.contains(&value) {
                current_references.push(value);
            }
        }
        Self {
            references: current_references,
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
        let mut identifiers: Vec<String> = self.identifiers;
        if !identifiers.contains(&identifier) {
            identifiers.push(identifier);
        }
        Self {
            identifiers,
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
        let mut current_identifiers: Vec<String> = self.identifiers;
        for value in identifiers.into_iter().map(|value| value.into()) {
            if !current_identifiers.contains(&value) {
                current_identifiers.push(value);
            }
        }
        Self {
            identifiers: current_identifiers,
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

    /// Add custom tag
    pub fn custom_tag<S>(self, tag: char, values: Vec<S>) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        if tag.is_alphabetic() {
            let values: Vec<String> = values.into_iter().map(|value| value.into()).collect();
            let mut generic_tags: HashMap<char, Vec<String>> = self.generic_tags;
            generic_tags
                .entry(tag)
                .and_modify(|list| {
                    for value in values.clone().into_iter() {
                        if !list.contains(&value) {
                            list.push(value);
                        }
                    }
                })
                .or_insert(values);
            Ok(Self {
                generic_tags,
                ..self
            })
        } else {
            Err(Error::InvalidTag(tag))
        }
    }
}

fn serialize_generic_tags<S>(
    generic_tags: &HashMap<char, Vec<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(generic_tags.len()))?;
    for (ch, values) in generic_tags {
        map.serialize_entry(&format!("#{}", ch), values)?;
    }
    map.end()
}

fn deserialize_generic_tags<'de, D>(deserializer: D) -> Result<HashMap<char, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct GenericTagsVisitor;

    impl<'de> Visitor<'de> for GenericTagsVisitor {
        type Value = HashMap<char, Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map in which the keys are \"#X\" for some character X")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut generic_tags = HashMap::new();
            while let Some(key) = map.next_key::<String>()? {
                let mut chars = key.chars();
                if let (Some('#'), Some(ch), None) = (chars.next(), chars.next(), chars.next()) {
                    let values = map.next_value()?;
                    generic_tags.insert(ch, values);
                } else {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
            Ok(generic_tags)
        }
    }

    deserializer.deserialize_map(GenericTagsVisitor)
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
        let filter = Filter::new()
            .identifier("identifier")
            .search("test")
            .custom_tag('j', vec!["test", "test1"])
            .unwrap();
        let json = r##"{"#d":["identifier"],"#j":["test","test1"],"search":"test"}"##;
        assert_eq!(filter.as_json(), json.to_string());

        assert_eq!(
            Filter::new().custom_tag('\n', vec!["test"]).unwrap_err(),
            Error::InvalidTag('\n')
        );
    }

    #[test]
    fn test_filter_deserialization() {
        let json = r##"{"#a":["...", "test"],"search":"test","ids":["myid", "mysecondid"]}"##;
        let filter = Filter::from_json(json).unwrap();
        assert_eq!(
            filter,
            Filter::new()
                .ids(vec!["myid".to_string(), "mysecondid".to_string()])
                .search("test")
                .custom_tag('a', vec!["...".to_string(), "test".to_string()])
                .unwrap()
        );

        let json = r##"{"#":["..."],"search":"test"}"##;
        let filter = Filter::from_json(json).unwrap();
        assert_eq!(filter, Filter::new().search("test"));

        let json = r##"{"aa":["..."],"search":"test"}"##;
        let filter = Filter::from_json(json).unwrap();
        assert_eq!(filter, Filter::new().search("test"));
    }
}
