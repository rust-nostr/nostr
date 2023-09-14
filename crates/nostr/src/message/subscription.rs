// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Subscription filters

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::RngCore;
use bitcoin::secp256k1::XOnlyPublicKey;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{EventId, Kind, Timestamp};

/// Alphabet Error
#[derive(Debug)]
pub enum AlphabetError {
    /// Invalid char
    InvalidChar,
}

#[cfg(feature = "std")]
impl std::error::Error for AlphabetError {}

impl fmt::Display for AlphabetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChar => write!(f, "invalid alphabet char"),
        }
    }
}

/// Alphabet
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Alphabet {
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

impl fmt::Display for Alphabet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A => write!(f, "a"),
            Self::B => write!(f, "b"),
            Self::C => write!(f, "c"),
            Self::D => write!(f, "d"),
            Self::E => write!(f, "e"),
            Self::F => write!(f, "f"),
            Self::G => write!(f, "g"),
            Self::H => write!(f, "h"),
            Self::I => write!(f, "i"),
            Self::J => write!(f, "j"),
            Self::K => write!(f, "k"),
            Self::L => write!(f, "l"),
            Self::M => write!(f, "m"),
            Self::N => write!(f, "n"),
            Self::O => write!(f, "o"),
            Self::P => write!(f, "p"),
            Self::Q => write!(f, "q"),
            Self::R => write!(f, "r"),
            Self::S => write!(f, "s"),
            Self::T => write!(f, "t"),
            Self::U => write!(f, "u"),
            Self::V => write!(f, "v"),
            Self::W => write!(f, "w"),
            Self::X => write!(f, "x"),
            Self::Y => write!(f, "y"),
            Self::Z => write!(f, "z"),
        }
    }
}

impl FromStr for Alphabet {
    type Err = AlphabetError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            "c" => Ok(Self::C),
            "d" => Ok(Self::D),
            "e" => Ok(Self::E),
            "f" => Ok(Self::F),
            "g" => Ok(Self::G),
            "h" => Ok(Self::H),
            "i" => Ok(Self::I),
            "j" => Ok(Self::J),
            "k" => Ok(Self::K),
            "l" => Ok(Self::L),
            "m" => Ok(Self::M),
            "n" => Ok(Self::N),
            "o" => Ok(Self::O),
            "p" => Ok(Self::P),
            "q" => Ok(Self::Q),
            "r" => Ok(Self::R),
            "s" => Ok(Self::S),
            "t" => Ok(Self::T),
            "u" => Ok(Self::U),
            "v" => Ok(Self::V),
            "w" => Ok(Self::W),
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            "z" => Ok(Self::Z),
            _ => Err(AlphabetError::InvalidChar),
        }
    }
}

impl Serialize for Alphabet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Alphabet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let alphaber: String = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Self::from_str(&alphaber).map_err(serde::de::Error::custom)
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
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        let mut rng = OsRng;
        Self::generate_with_rng(&mut rng)
    }

    /// Generate new random [`SubscriptionId`]
    pub fn generate_with_rng<R>(rng: &mut R) -> Self
    where
        R: RngCore,
    {
        let mut os_random = [0u8; 32];
        rng.fill_bytes(&mut os_random);
        let hash = Sha256Hash::hash(&os_random).to_string();
        Self::new(&hash[..32])
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
    pub generic_tags: BTreeMap<Alphabet, Vec<String>>,
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

    /// Remove event ids or prefixes
    pub fn remove_ids<S>(self, ids: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let ids: BTreeSet<String> = ids.into_iter().map(|id| id.into()).collect();
        Self {
            ids: self
                .ids
                .into_iter()
                .filter(|id| !ids.contains(id))
                .collect(),
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

    /// Remove authors
    pub fn remove_authors<S>(self, authors: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let authors: BTreeSet<String> = authors.into_iter().map(|id| id.into()).collect();
        Self {
            authors: self
                .authors
                .into_iter()
                .filter(|value| !authors.contains(value))
                .collect(),
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

    /// Remove kinds
    pub fn remove_kinds(self, kinds: Vec<Kind>) -> Self {
        let kinds: BTreeSet<Kind> = kinds.into_iter().collect();
        Self {
            kinds: self
                .kinds
                .into_iter()
                .filter(|value| !kinds.contains(value))
                .collect(),
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

    /// Remove events
    pub fn remove_events<S>(self, events: Vec<EventId>) -> Self {
        let events: BTreeSet<EventId> = events.into_iter().collect();
        Self {
            events: self
                .events
                .into_iter()
                .filter(|value| !events.contains(value))
                .collect(),
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

    /// Remove pubkeys
    pub fn remove_pubkeys<S>(self, pubkeys: Vec<XOnlyPublicKey>) -> Self {
        let pubkeys: BTreeSet<XOnlyPublicKey> = pubkeys.into_iter().collect();
        Self {
            pubkeys: self
                .pubkeys
                .into_iter()
                .filter(|value| !pubkeys.contains(value))
                .collect(),
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

    /// Remove hashtags
    pub fn remove_hashtags<S>(self, hashtags: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let hashtags: BTreeSet<String> = hashtags.into_iter().map(|id| id.into()).collect();
        Self {
            hashtags: self
                .hashtags
                .into_iter()
                .filter(|value| !hashtags.contains(value))
                .collect(),
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

    /// Remove references
    pub fn remove_references<S>(self, references: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let references: BTreeSet<String> = references.into_iter().map(|id| id.into()).collect();
        Self {
            references: self
                .references
                .into_iter()
                .filter(|value| !references.contains(value))
                .collect(),
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

    /// Remove identifiers
    pub fn remove_identifiers<S>(self, identifiers: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let identifiers: BTreeSet<String> = identifiers.into_iter().map(|id| id.into()).collect();
        Self {
            identifiers: self
                .identifiers
                .into_iter()
                .filter(|value| !identifiers.contains(value))
                .collect(),
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

    /// Remove search
    pub fn remove_search(self) -> Self {
        Self {
            search: None,
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

    /// Remove since
    pub fn remove_since(self) -> Self {
        Self {
            since: None,
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

    /// Remove until
    pub fn remove_until(self) -> Self {
        Self {
            until: None,
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

    /// Remove limit
    pub fn remove_limit(self) -> Self {
        Self {
            limit: None,
            ..self
        }
    }

    /// Add custom tag
    pub fn custom_tag<S>(self, tag: Alphabet, values: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let values: Vec<String> = values.into_iter().map(|value| value.into()).collect();
        let mut generic_tags: BTreeMap<Alphabet, Vec<String>> = self.generic_tags;
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
        Self {
            generic_tags,
            ..self
        }
    }

    /// Remove identifiers
    pub fn remove_custom_tag<S>(self, tag: Alphabet, values: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        let values: BTreeSet<String> = values.into_iter().map(|id| id.into()).collect();
        let mut generic_tags: BTreeMap<Alphabet, Vec<String>> = self.generic_tags;
        generic_tags.entry(tag).and_modify(|list| {
            list.retain(|value| !values.contains(value));
        });
        Self {
            generic_tags,
            ..self
        }
    }
}

fn serialize_generic_tags<S>(
    generic_tags: &BTreeMap<Alphabet, Vec<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(generic_tags.len()))?;
    for (tag, values) in generic_tags {
        map.serialize_entry(&format!("#{tag}"), values)?;
    }
    map.end()
}

fn deserialize_generic_tags<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<Alphabet, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct GenericTagsVisitor;

    impl<'de> Visitor<'de> for GenericTagsVisitor {
        type Value = BTreeMap<Alphabet, Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map in which the keys are \"#X\" for some character X")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut generic_tags = BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                let mut chars = key.chars();
                if let (Some('#'), Some(ch), None) = (chars.next(), chars.next(), chars.next()) {
                    let tag: Alphabet = Alphabet::from_str(ch.to_string().as_str())
                        .map_err(serde::de::Error::custom)?;
                    let values = map.next_value()?;
                    generic_tags.insert(tag, values);
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
    fn test_remove_ids() {
        let filter = Filter::new().id("abcdefg").id("12345678").id("xyz");
        let filter = filter.remove_ids(vec!["12345678", "xyz"]);
        assert_eq!(filter, Filter::new().id("abcdefg"));
    }

    #[test]
    fn test_remove_custom_tag() {
        let filter = Filter::new().custom_tag(Alphabet::C, vec!["test", "test2"]);
        let filter = filter.remove_custom_tag(Alphabet::C, vec!["test2"]);
        assert_eq!(filter, Filter::new().custom_tag(Alphabet::C, vec!["test"]));
    }

    #[test]
    fn test_filter_serialization() {
        let filter = Filter::new()
            .identifier("identifier")
            .search("test")
            .custom_tag(Alphabet::J, vec!["test", "test1"]);
        let json = r##"{"#d":["identifier"],"#j":["test","test1"],"search":"test"}"##;
        assert_eq!(filter.as_json(), json.to_string());
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
                .custom_tag(Alphabet::A, vec!["...".to_string(), "test".to_string()])
        );

        let json = r##"{"#":["..."],"search":"test"}"##;
        let filter = Filter::from_json(json).unwrap();
        assert_eq!(filter, Filter::new().search("test"));

        let json = r##"{"aa":["..."],"search":"test"}"##;
        let filter = Filter::from_json(json).unwrap();
        assert_eq!(filter, Filter::new().search("test"));
    }
}
