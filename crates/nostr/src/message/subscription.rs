// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Subscription filters

#[cfg(not(feature = "std"))]
use alloc::collections::{BTreeMap as AllocMap, BTreeSet as AllocSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::collections::{HashMap as AllocMap, HashSet as AllocSet};

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

use crate::event::{TagIndexValues, TagIndexes};
use crate::{Event, EventId, JsonUtil, Kind, Timestamp};

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

impl Alphabet {
    /// Get as char
    pub fn as_char(&self) -> char {
        match self {
            Self::A => 'a',
            Self::B => 'b',
            Self::C => 'c',
            Self::D => 'd',
            Self::E => 'e',
            Self::F => 'f',
            Self::G => 'g',
            Self::H => 'h',
            Self::I => 'i',
            Self::J => 'j',
            Self::K => 'k',
            Self::L => 'l',
            Self::M => 'm',
            Self::N => 'n',
            Self::O => 'o',
            Self::P => 'p',
            Self::Q => 'q',
            Self::R => 'r',
            Self::S => 's',
            Self::T => 't',
            Self::U => 'u',
            Self::V => 'v',
            Self::W => 'w',
            Self::X => 'x',
            Self::Y => 'y',
            Self::Z => 'z',
        }
    }
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

impl TryFrom<char> for Alphabet {
    type Error = AlphabetError;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'a' => Ok(Self::A),
            'b' => Ok(Self::B),
            'c' => Ok(Self::C),
            'd' => Ok(Self::D),
            'e' => Ok(Self::E),
            'f' => Ok(Self::F),
            'g' => Ok(Self::G),
            'h' => Ok(Self::H),
            'i' => Ok(Self::I),
            'j' => Ok(Self::J),
            'k' => Ok(Self::K),
            'l' => Ok(Self::L),
            'm' => Ok(Self::M),
            'n' => Ok(Self::N),
            'o' => Ok(Self::O),
            'p' => Ok(Self::P),
            'q' => Ok(Self::Q),
            'r' => Ok(Self::R),
            's' => Ok(Self::S),
            't' => Ok(Self::T),
            'u' => Ok(Self::U),
            'v' => Ok(Self::V),
            'w' => Ok(Self::W),
            'x' => Ok(Self::X),
            'y' => Ok(Self::Y),
            'z' => Ok(Self::Z),
            _ => Err(AlphabetError::InvalidChar),
        }
    }
}

impl FromStr for Alphabet {
    type Err = AlphabetError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c: char = s.chars().next().ok_or(AlphabetError::InvalidChar)?;
        Self::try_from(c)
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl Serialize for SubscriptionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SubscriptionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let id: String = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self::new(id))
    }
}

/// Generic Tag Value
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GenericTagValue {
    /// Public Key
    Pubkey(XOnlyPublicKey),
    /// Event Id
    EventId(EventId),
    /// Other (string)
    String(String),
}

impl fmt::Display for GenericTagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pubkey(inner) => write!(f, "{inner}"),
            Self::EventId(inner) => write!(f, "{inner}"),
            Self::String(inner) => write!(f, "{inner}"),
        }
    }
}

#[allow(missing_docs)]
pub trait IntoGenericTagValue {
    fn into_generic_tag_value(self) -> GenericTagValue;
}

impl IntoGenericTagValue for XOnlyPublicKey {
    fn into_generic_tag_value(self) -> GenericTagValue {
        GenericTagValue::Pubkey(self)
    }
}

impl IntoGenericTagValue for EventId {
    fn into_generic_tag_value(self) -> GenericTagValue {
        GenericTagValue::EventId(self)
    }
}

impl IntoGenericTagValue for String {
    fn into_generic_tag_value(self) -> GenericTagValue {
        GenericTagValue::String(self)
    }
}

impl IntoGenericTagValue for &str {
    fn into_generic_tag_value(self) -> GenericTagValue {
        GenericTagValue::String(self.to_string())
    }
}

/// Subscription filters
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    /// List of [`EventId`]
    #[serde(skip_serializing_if = "AllocSet::is_empty")]
    #[serde(default)]
    pub ids: AllocSet<EventId>,
    /// List of [`XOnlyPublicKey`]
    #[serde(skip_serializing_if = "AllocSet::is_empty")]
    #[serde(default)]
    pub authors: AllocSet<XOnlyPublicKey>,
    /// List of a kind numbers
    #[serde(skip_serializing_if = "AllocSet::is_empty")]
    #[serde(default)]
    pub kinds: AllocSet<Kind>,
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
    pub generic_tags: AllocMap<Alphabet, AllocSet<GenericTagValue>>,
}

impl Filter {
    /// Create new empty [`Filter`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add [`EventId`]
    pub fn id(mut self, id: EventId) -> Self {
        self.ids.insert(id);
        self
    }

    /// Add event ids or prefixes
    pub fn ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        self.ids.extend(ids);
        self
    }

    /// Remove event ids
    pub fn remove_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        for id in ids {
            self.ids.remove(&id);
        }
        self
    }

    /// Add author
    pub fn author(mut self, author: XOnlyPublicKey) -> Self {
        self.authors.insert(author);
        self
    }

    /// Add authors
    pub fn authors<I>(mut self, authors: I) -> Self
    where
        I: IntoIterator<Item = XOnlyPublicKey>,
    {
        self.authors.extend(authors);
        self
    }

    /// Remove authors
    pub fn remove_authors<I>(mut self, authors: I) -> Self
    where
        I: IntoIterator<Item = XOnlyPublicKey>,
    {
        for author in authors {
            self.authors.remove(&author);
        }
        self
    }

    /// Add kind
    pub fn kind(mut self, kind: Kind) -> Self {
        self.kinds.insert(kind);
        self
    }

    /// Add kinds
    pub fn kinds<I>(mut self, kinds: I) -> Self
    where
        I: IntoIterator<Item = Kind>,
    {
        self.kinds.extend(kinds);
        self
    }

    /// Remove kinds
    pub fn remove_kinds<I>(mut self, kinds: I) -> Self
    where
        I: IntoIterator<Item = Kind>,
    {
        for kind in kinds {
            self.kinds.remove(&kind);
        }
        self
    }

    /// Add event
    pub fn event(self, id: EventId) -> Self {
        self.custom_tag(Alphabet::E, vec![id])
    }

    /// Add events
    pub fn events<I>(self, events: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        self.custom_tag(Alphabet::E, events)
    }

    /// Remove events
    pub fn remove_events<I>(self, events: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        self.remove_custom_tag(Alphabet::E, events)
    }

    /// Add pubkey
    pub fn pubkey(self, pubkey: XOnlyPublicKey) -> Self {
        self.custom_tag(Alphabet::P, vec![pubkey])
    }

    /// Add pubkeys
    pub fn pubkeys<I>(self, pubkeys: I) -> Self
    where
        I: IntoIterator<Item = XOnlyPublicKey>,
    {
        self.custom_tag(Alphabet::P, pubkeys)
    }

    /// Remove pubkeys
    pub fn remove_pubkeys<I>(self, pubkeys: I) -> Self
    where
        I: IntoIterator<Item = XOnlyPublicKey>,
    {
        self.remove_custom_tag(Alphabet::P, pubkeys)
    }

    /// Add hashtag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtag<S>(self, hashtag: S) -> Self
    where
        S: Into<String>,
    {
        self.custom_tag(Alphabet::T, vec![hashtag.into()])
    }

    /// Add hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn hashtags<I, S>(self, hashtags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.custom_tag(Alphabet::T, hashtags.into_iter().map(|s| s.into()))
    }

    /// Remove hashtags
    pub fn remove_hashtags<I, S>(self, hashtags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.remove_custom_tag(Alphabet::T, hashtags.into_iter().map(|s| s.into()))
    }

    /// Add reference
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn reference<S>(self, reference: S) -> Self
    where
        S: Into<String>,
    {
        self.custom_tag(Alphabet::R, vec![reference.into()])
    }

    /// Add references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    pub fn references<I, S>(self, references: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.custom_tag(Alphabet::R, references.into_iter().map(|s| s.into()))
    }

    /// Remove references
    pub fn remove_references<I, S>(self, references: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.remove_custom_tag(Alphabet::R, references.into_iter().map(|s| s.into()))
    }

    /// Add identifier
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/33.md>
    pub fn identifier<S>(self, identifier: S) -> Self
    where
        S: Into<String>,
    {
        self.custom_tag(Alphabet::D, vec![identifier.into()])
    }

    /// Add identifiers
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/33.md>
    pub fn identifiers<I, S>(self, identifiers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.custom_tag(Alphabet::D, identifiers.into_iter().map(|s| s.into()))
    }

    /// Remove identifiers
    pub fn remove_identifiers<I, S>(self, identifiers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.remove_custom_tag(Alphabet::D, identifiers.into_iter().map(|s| s.into()))
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
    pub fn custom_tag<I, T>(mut self, tag: Alphabet, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoGenericTagValue,
    {
        let values: AllocSet<GenericTagValue> = values
            .into_iter()
            .map(|v| v.into_generic_tag_value())
            .collect();
        self.generic_tags
            .entry(tag)
            .and_modify(|list| {
                list.extend(values.clone());
            })
            .or_insert(values);
        self
    }

    /// Remove identifiers
    pub fn remove_custom_tag<I, T>(mut self, tag: Alphabet, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoGenericTagValue,
    {
        let values: AllocSet<GenericTagValue> = values
            .into_iter()
            .map(|v| v.into_generic_tag_value())
            .collect();
        self.generic_tags.entry(tag).and_modify(|list| {
            list.retain(|value| !values.contains(value));
        });
        self
    }
}

impl Filter {
    fn ids_match(&self, event: &Event) -> bool {
        self.ids.is_empty() || self.ids.contains(&event.id)
    }

    fn authors_match(&self, event: &Event) -> bool {
        self.authors.is_empty() || self.authors.contains(&event.pubkey)
    }

    fn tag_match(&self, event: &Event) -> bool {
        if self.generic_tags.is_empty() {
            return true;
        }
        if event.tags.is_empty() {
            return false;
        }

        let idx: TagIndexes = event.build_tags_index();
        self.generic_tags.iter().all(|(tagname, set)| {
            idx.get(tagname).map_or(false, |valset| {
                TagIndexValues::iter(set)
                    .filter(|t| valset.contains(t))
                    .count()
                    > 0
            })
        })
    }

    fn kind_match(&self, kind: &Kind) -> bool {
        self.kinds.is_empty() || self.kinds.contains(kind)
    }

    /// Determine if [`Filter`] match the provided [`Event`].
    pub fn match_event(&self, event: &Event) -> bool {
        self.ids_match(event)
            && self.since.map_or(true, |t| event.created_at >= t)
            && self.until.map_or(true, |t| event.created_at <= t)
            && self.kind_match(&event.kind)
            && self.authors_match(event)
            && self.tag_match(event)
    }

    /// Check if [`Filter`] is empty
    pub fn is_empty(&self) -> bool {
        self == &Filter::default()
    }
}

/// Filters match event trait
pub trait FiltersMatchEvent {
    /// Determine if [`Filter`] match the provided [`Event`].
    fn match_event(&self, event: &Event) -> bool;
}

impl FiltersMatchEvent for Vec<Filter> {
    fn match_event(&self, event: &Event) -> bool {
        self.iter().any(|f| f.match_event(event))
    }
}

impl JsonUtil for Filter {
    type Err = serde_json::Error;
}

fn serialize_generic_tags<S>(
    generic_tags: &AllocMap<Alphabet, AllocSet<GenericTagValue>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(generic_tags.len()))?;
    for (tag, values) in generic_tags.iter() {
        map.serialize_entry(&format!("#{tag}"), values)?;
    }
    map.end()
}

fn deserialize_generic_tags<'de, D>(
    deserializer: D,
) -> Result<AllocMap<Alphabet, AllocSet<GenericTagValue>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct GenericTagsVisitor;

    impl<'de> Visitor<'de> for GenericTagsVisitor {
        type Value = AllocMap<Alphabet, AllocSet<GenericTagValue>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map in which the keys are \"#X\" for some character X")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut generic_tags = AllocMap::new();
            while let Some(key) = map.next_key::<String>()? {
                let mut chars = key.chars();
                if let (Some('#'), Some(ch), None) = (chars.next(), chars.next(), chars.next()) {
                    let tag: Alphabet = Alphabet::from_str(ch.to_string().as_str())
                        .map_err(serde::de::Error::custom)?;
                    let mut values: AllocSet<GenericTagValue> = map.next_value()?;

                    match tag {
                        Alphabet::P => values.retain(|v| matches!(v, GenericTagValue::Pubkey(_))),
                        Alphabet::E => values.retain(|v| matches!(v, GenericTagValue::EventId(_))),
                        _ => {}
                    }

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
    use crate::Tag;

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
        let event_id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let filter = Filter::new().id(EventId::all_zeros()).id(event_id);
        let filter = filter.remove_ids(vec![EventId::all_zeros()]);
        assert_eq!(filter, Filter::new().id(event_id));
    }

    #[test]
    fn test_remove_custom_tag() {
        let filter = Filter::new().custom_tag(Alphabet::C, vec!["test", "test2"]);
        let filter = filter.remove_custom_tag(Alphabet::C, vec!["test2"]);
        assert_eq!(filter, Filter::new().custom_tag(Alphabet::C, vec!["test"]));
    }

    #[test]
    fn test_add_remove_event_tag() {
        let mut filter = Filter::new().identifier("myidentifier");
        filter = filter.custom_tag(Alphabet::D, vec!["mysecondid"]);
        filter = filter.identifiers(vec!["test", "test2"]);
        filter = filter.remove_custom_tag(Alphabet::D, vec!["test2"]);
        filter = filter.remove_identifiers(vec!["mysecondid"]);
        assert_eq!(
            filter,
            Filter::new().identifiers(vec!["myidentifier", "test"])
        );
    }

    #[test]
    fn test_filter_serialization() {
        let filter = Filter::new()
            .identifier("identifier")
            .search("test")
            .custom_tag(Alphabet::J, vec!["test1"])
            .custom_tag(
                Alphabet::P,
                vec!["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"],
            );
        let json = r##"{"#d":["identifier"],"#j":["test1"],"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"],"search":"test"}"##;
        assert_eq!(filter.as_json(), json.to_string());
    }

    #[test]
    fn test_filter_deserialization() {
        let json = r##"{"#a":["...", "test"],"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"],"search":"test","ids":["70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5"]}"##;
        let filter = Filter::from_json(json).unwrap();
        let event_id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let pubkey = XOnlyPublicKey::from_str(
            "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
        )
        .unwrap();
        assert_eq!(
            filter,
            Filter::new()
                .ids(vec![event_id])
                .search("test")
                .custom_tag(Alphabet::A, vec!["...".to_string(), "test".to_string()])
                .pubkey(pubkey)
        );

        let json = r##"{"#":["..."],"search":"test"}"##;
        let filter = Filter::from_json(json).unwrap();
        assert_eq!(filter, Filter::new().search("test"));

        let json = r##"{"aa":["..."],"search":"test"}"##;
        let filter = Filter::from_json(json).unwrap();
        assert_eq!(filter, Filter::new().search("test"));
    }

    #[test]
    fn test_match_event() {
        let event_id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let pubkey = XOnlyPublicKey::from_str(
            "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
        )
        .unwrap();
        let event =
            Event::new_dummy(
                "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5",
                "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
                Timestamp::from(1612809991),
                1,
                vec![
                    Tag::PubKey(XOnlyPublicKey::from_str("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a").unwrap(), None),
                    Tag::Event(EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96").unwrap(), None, None),
                ],
                "test",
                "273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"
            );

        let event_with_empty_tags =
            Event::new_dummy(
                "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5",
                "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
                Timestamp::from(1612809991),
                1,
                vec![],
                "test",
                "273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"
            );

        // ID match
        let filter = Filter::new().id(event_id);
        assert!(filter.match_event(&event));

        // Not match (kind)
        let filter = Filter::new().id(event_id).kind(Kind::Metadata);
        assert!(!filter.match_event(&event));

        // Match (author, kind and since)
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::TextNote)
            .since(Timestamp::from(1612808000));
        assert!(filter.match_event(&event));

        // Not match (since)
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::TextNote)
            .since(Timestamp::from(1700000000));
        assert!(!filter.match_event(&event));

        // Match (#p tag and kind)
        let filter = Filter::new()
            .pubkey(
                XOnlyPublicKey::from_str(
                    "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
                )
                .unwrap(),
            )
            .kind(Kind::TextNote);
        assert!(filter.match_event(&event));

        // Match (tags)
        let filter = Filter::new()
            .pubkey(
                XOnlyPublicKey::from_str(
                    "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
                )
                .unwrap(),
            )
            .event(
                EventId::from_hex(
                    "7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96",
                )
                .unwrap(),
            );
        assert!(filter.match_event(&event));

        // Match (tags)
        let filter = Filter::new().events(vec![
            EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96")
                .unwrap(),
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap(),
        ]);
        assert!(filter.match_event(&event));

        // Not match (tags)
        let filter = Filter::new().events(vec![EventId::from_hex(
            "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5",
        )
        .unwrap()]);
        assert!(!filter.match_event(&event));

        let filters: Vec<Filter> = vec![
            // Filter that match
            Filter::new()
                .author(pubkey)
                .kind(Kind::TextNote)
                .since(Timestamp::from(1612808000)),
            // Filter that not match
            Filter::new().events(vec![EventId::from_hex(
                "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5",
            )
            .unwrap()]),
        ];
        assert!(filters.match_event(&event));

        // Not match (tags filter for events with empty tags)
        let filter = Filter::new().hashtag("this-should-not-match");
        assert!(!filter.match_event(&event));
        assert!(!filter.match_event(&event_with_empty_tags));
    }

    #[test]
    fn test_filter_is_empty() {
        let filter = Filter::new().identifier("test");
        assert!(!filter.is_empty());

        let filter = Filter::new();
        assert!(filter.is_empty());
    }
}
