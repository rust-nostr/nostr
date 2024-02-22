// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Subscription filters

#[cfg(not(feature = "std"))]
use alloc::collections::{BTreeMap as AllocMap, BTreeSet as AllocSet};
use alloc::string::{String, ToString};
use core::fmt;
#[cfg(feature = "std")]
use std::collections::{HashMap as AllocMap, HashSet as AllocSet};

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::RngCore;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use crate::{EventId, JsonUtil, Kind, PublicKey, Timestamp};

type GenericTags = AllocMap<SingleLetterTag, AllocSet<GenericTagValue>>;

/// Alphabet Error
#[derive(Debug)]
pub enum SingleLetterTagError {
    /// Invalid char
    InvalidChar,
}

#[cfg(feature = "std")]
impl std::error::Error for SingleLetterTagError {}

impl fmt::Display for SingleLetterTagError {
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

/// Single-Letter Tag (a-zA-Z)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SingleLetterTag {
    /// Single-letter char
    pub character: Alphabet,
    /// Is the `character` uppercase?
    pub uppercase: bool,
}

impl SingleLetterTag {
    /// Compose new `lowercase` single-letter tag
    pub fn lowercase(character: Alphabet) -> Self {
        Self {
            character,
            uppercase: false,
        }
    }

    /// Compose new `uppercase` single-letter tag
    pub fn uppercase(character: Alphabet) -> Self {
        Self {
            character,
            uppercase: true,
        }
    }

    /// Parse single-letter tag from [char]
    pub fn from_char(c: char) -> Result<Self, SingleLetterTagError> {
        let character = match c {
            'a' | 'A' => Alphabet::A,
            'b' | 'B' => Alphabet::B,
            'c' | 'C' => Alphabet::C,
            'd' | 'D' => Alphabet::D,
            'e' | 'E' => Alphabet::E,
            'f' | 'F' => Alphabet::F,
            'g' | 'G' => Alphabet::G,
            'h' | 'H' => Alphabet::H,
            'i' | 'I' => Alphabet::I,
            'j' | 'J' => Alphabet::J,
            'k' | 'K' => Alphabet::K,
            'l' | 'L' => Alphabet::L,
            'm' | 'M' => Alphabet::M,
            'n' | 'N' => Alphabet::N,
            'o' | 'O' => Alphabet::O,
            'p' | 'P' => Alphabet::P,
            'q' | 'Q' => Alphabet::Q,
            'r' | 'R' => Alphabet::R,
            's' | 'S' => Alphabet::S,
            't' | 'T' => Alphabet::T,
            'u' | 'U' => Alphabet::U,
            'v' | 'V' => Alphabet::V,
            'w' | 'W' => Alphabet::W,
            'x' | 'X' => Alphabet::X,
            'y' | 'Y' => Alphabet::Y,
            'z' | 'Z' => Alphabet::Z,
            _ => return Err(SingleLetterTagError::InvalidChar),
        };

        Ok(Self {
            character,
            uppercase: c.is_uppercase(),
        })
    }

    /// Get as char
    pub fn as_char(&self) -> char {
        if self.uppercase {
            match self.character {
                Alphabet::A => 'A',
                Alphabet::B => 'B',
                Alphabet::C => 'C',
                Alphabet::D => 'D',
                Alphabet::E => 'E',
                Alphabet::F => 'F',
                Alphabet::G => 'G',
                Alphabet::H => 'H',
                Alphabet::I => 'I',
                Alphabet::J => 'J',
                Alphabet::K => 'K',
                Alphabet::L => 'L',
                Alphabet::M => 'M',
                Alphabet::N => 'N',
                Alphabet::O => 'O',
                Alphabet::P => 'P',
                Alphabet::Q => 'Q',
                Alphabet::R => 'R',
                Alphabet::S => 'S',
                Alphabet::T => 'T',
                Alphabet::U => 'U',
                Alphabet::V => 'V',
                Alphabet::W => 'W',
                Alphabet::X => 'X',
                Alphabet::Y => 'Y',
                Alphabet::Z => 'Z',
            }
        } else {
            match self.character {
                Alphabet::A => 'a',
                Alphabet::B => 'b',
                Alphabet::C => 'c',
                Alphabet::D => 'd',
                Alphabet::E => 'e',
                Alphabet::F => 'f',
                Alphabet::G => 'g',
                Alphabet::H => 'h',
                Alphabet::I => 'i',
                Alphabet::J => 'j',
                Alphabet::K => 'k',
                Alphabet::L => 'l',
                Alphabet::M => 'm',
                Alphabet::N => 'n',
                Alphabet::O => 'o',
                Alphabet::P => 'p',
                Alphabet::Q => 'q',
                Alphabet::R => 'r',
                Alphabet::S => 's',
                Alphabet::T => 't',
                Alphabet::U => 'u',
                Alphabet::V => 'v',
                Alphabet::W => 'w',
                Alphabet::X => 'x',
                Alphabet::Y => 'y',
                Alphabet::Z => 'z',
            }
        }
    }

    /// Check if single-letter tag is `lowercase`
    pub fn is_lowercase(&self) -> bool {
        !self.uppercase
    }

    /// Check if single-letter tag is `uppercase`
    pub fn is_uppercase(&self) -> bool {
        self.uppercase
    }
}

impl fmt::Display for SingleLetterTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.as_char())
    }
}

impl Serialize for SingleLetterTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_char(self.as_char())
    }
}

impl<'de> Deserialize<'de> for SingleLetterTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let character: char = char::deserialize(deserializer)?;
        Self::from_char(character).map_err(serde::de::Error::custom)
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
        let id: String = String::deserialize(deserializer)?;
        Ok(Self::new(id))
    }
}

/// Generic Tag Value
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GenericTagValue {
    /// Public Key
    Pubkey(PublicKey),
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

impl IntoGenericTagValue for PublicKey {
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
///
/// <https://github.com/nostr-protocol/nips/blob/master/01.md>
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    /// List of [`EventId`]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ids: Option<AllocSet<EventId>>,
    /// List of [`PublicKey`]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub authors: Option<AllocSet<PublicKey>>,
    /// List of a kind numbers
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub kinds: Option<AllocSet<Kind>>,
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
    pub generic_tags: GenericTags,
}

impl Filter {
    /// Create new empty [`Filter`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add [`EventId`]
    #[inline]
    pub fn id(self, id: EventId) -> Self {
        self.ids([id])
    }

    /// Add event ids or prefixes
    #[inline]
    pub fn ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        self.ids = extend_or_collect(self.ids, ids);
        self
    }

    /// Remove event ids
    #[inline]
    pub fn remove_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        self.ids = remove_or_none(self.ids, ids);
        self
    }

    /// Add author
    #[inline]
    pub fn author(self, author: PublicKey) -> Self {
        self.authors([author])
    }

    /// Add authors
    #[inline]
    pub fn authors<I>(mut self, authors: I) -> Self
    where
        I: IntoIterator<Item = PublicKey>,
    {
        self.authors = extend_or_collect(self.authors, authors);
        self
    }

    /// Remove authors
    #[inline]
    pub fn remove_authors<I>(mut self, authors: I) -> Self
    where
        I: IntoIterator<Item = PublicKey>,
    {
        self.authors = remove_or_none(self.authors, authors);
        self
    }

    /// Add kind
    #[inline]
    pub fn kind(self, kind: Kind) -> Self {
        self.kinds([kind])
    }

    /// Add kinds
    #[inline]
    pub fn kinds<I>(mut self, kinds: I) -> Self
    where
        I: IntoIterator<Item = Kind>,
    {
        self.kinds = extend_or_collect(self.kinds, kinds);
        self
    }

    /// Remove kinds
    #[inline]
    pub fn remove_kinds<I>(mut self, kinds: I) -> Self
    where
        I: IntoIterator<Item = Kind>,
    {
        self.kinds = remove_or_none(self.kinds, kinds);
        self
    }

    /// Add event
    #[inline]
    pub fn event(self, id: EventId) -> Self {
        self.custom_tag(SingleLetterTag::lowercase(Alphabet::E), [id])
    }

    /// Add events
    #[inline]
    pub fn events<I>(self, events: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        self.custom_tag(SingleLetterTag::lowercase(Alphabet::E), events)
    }

    /// Remove events
    #[inline]
    pub fn remove_events<I>(self, events: I) -> Self
    where
        I: IntoIterator<Item = EventId>,
    {
        self.remove_custom_tag(SingleLetterTag::lowercase(Alphabet::E), events)
    }

    /// Add pubkey
    #[inline]
    pub fn pubkey(self, pubkey: PublicKey) -> Self {
        self.custom_tag(SingleLetterTag::lowercase(Alphabet::P), [pubkey])
    }

    /// Add pubkeys
    #[inline]
    pub fn pubkeys<I>(self, pubkeys: I) -> Self
    where
        I: IntoIterator<Item = PublicKey>,
    {
        self.custom_tag(SingleLetterTag::lowercase(Alphabet::P), pubkeys)
    }

    /// Remove pubkeys
    #[inline]
    pub fn remove_pubkeys<I>(self, pubkeys: I) -> Self
    where
        I: IntoIterator<Item = PublicKey>,
    {
        self.remove_custom_tag(SingleLetterTag::lowercase(Alphabet::P), pubkeys)
    }

    /// Add hashtag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[inline]
    pub fn hashtag<S>(self, hashtag: S) -> Self
    where
        S: Into<String>,
    {
        self.custom_tag(SingleLetterTag::lowercase(Alphabet::T), [hashtag.into()])
    }

    /// Add hashtags
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[inline]
    pub fn hashtags<I, S>(self, hashtags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.custom_tag(
            SingleLetterTag::lowercase(Alphabet::T),
            hashtags.into_iter().map(|s| s.into()),
        )
    }

    /// Remove hashtags
    #[inline]
    pub fn remove_hashtags<I, S>(self, hashtags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.remove_custom_tag(
            SingleLetterTag::lowercase(Alphabet::T),
            hashtags.into_iter().map(|s| s.into()),
        )
    }

    /// Add reference
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[inline]
    pub fn reference<S>(self, reference: S) -> Self
    where
        S: Into<String>,
    {
        self.custom_tag(SingleLetterTag::lowercase(Alphabet::R), [reference.into()])
    }

    /// Add references
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/12.md>
    #[inline]
    pub fn references<I, S>(self, references: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.custom_tag(
            SingleLetterTag::lowercase(Alphabet::R),
            references.into_iter().map(|s| s.into()),
        )
    }

    /// Remove references
    #[inline]
    pub fn remove_references<I, S>(self, references: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.remove_custom_tag(
            SingleLetterTag::lowercase(Alphabet::R),
            references.into_iter().map(|s| s.into()),
        )
    }

    /// Add identifier
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/33.md>
    #[inline]
    pub fn identifier<S>(self, identifier: S) -> Self
    where
        S: Into<String>,
    {
        self.custom_tag(SingleLetterTag::lowercase(Alphabet::D), [identifier.into()])
    }

    /// Add identifiers
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/33.md>
    #[inline]
    pub fn identifiers<I, S>(self, identifiers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.custom_tag(
            SingleLetterTag::lowercase(Alphabet::D),
            identifiers.into_iter().map(|s| s.into()),
        )
    }

    /// Remove identifiers
    #[inline]
    pub fn remove_identifiers<I, S>(self, identifiers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.remove_custom_tag(
            SingleLetterTag::lowercase(Alphabet::D),
            identifiers.into_iter().map(|s| s.into()),
        )
    }

    /// Add search field
    #[inline]
    pub fn search<S>(mut self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.search = Some(value.into());
        self
    }

    /// Remove search
    #[inline]
    pub fn remove_search(mut self) -> Self {
        self.search = None;
        self
    }

    /// Add since unix timestamp
    #[inline]
    pub fn since(mut self, since: Timestamp) -> Self {
        self.since = Some(since);
        self
    }

    /// Remove since
    #[inline]
    pub fn remove_since(mut self) -> Self {
        self.since = None;
        self
    }

    /// Add until unix timestamp
    #[inline]
    pub fn until(mut self, until: Timestamp) -> Self {
        self.until = Some(until);
        self
    }

    /// Remove until
    #[inline]
    pub fn remove_until(mut self) -> Self {
        self.until = None;
        self
    }

    /// Add limit
    ///
    /// Maximum number of events to be returned in the initial query
    #[inline]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Remove limit
    #[inline]
    pub fn remove_limit(mut self) -> Self {
        self.limit = None;
        self
    }

    /// Add custom tag
    pub fn custom_tag<I, T>(mut self, tag: SingleLetterTag, values: I) -> Self
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

    /// Remove custom tag
    pub fn remove_custom_tag<I, T>(mut self, tag: SingleLetterTag, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoGenericTagValue,
    {
        let values = values.into_iter().map(|v| v.into_generic_tag_value());
        self.generic_tags.entry(tag).and_modify(|set| {
            for item in values {
                set.remove(&item);
            }
        });

        // Remove tag if empty
        if let Some(set) = self.generic_tags.get(&tag) {
            if set.is_empty() {
                self.generic_tags.remove(&tag);
            }
        }

        self
    }

    /// Check if [`Filter`] is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self == &Filter::default()
    }
}

impl JsonUtil for Filter {
    type Err = serde_json::Error;
}

fn serialize_generic_tags<S>(generic_tags: &GenericTags, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(generic_tags.len()))?;
    for (tag, values) in generic_tags.iter() {
        map.serialize_entry(&tag.to_string(), values)?;
    }
    map.end()
}

fn deserialize_generic_tags<'de, D>(deserializer: D) -> Result<GenericTags, D::Error>
where
    D: Deserializer<'de>,
{
    struct GenericTagsVisitor;

    impl<'de> Visitor<'de> for GenericTagsVisitor {
        type Value = GenericTags;

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
                    let tag: SingleLetterTag =
                        SingleLetterTag::from_char(ch).map_err(serde::de::Error::custom)?;
                    let mut values: AllocSet<GenericTagValue> = map.next_value()?;

                    // Check if char is lowercase
                    if tag.is_lowercase() {
                        match tag.character {
                            Alphabet::P => {
                                values.retain(|v| matches!(v, GenericTagValue::Pubkey(_)))
                            }
                            Alphabet::E => {
                                values.retain(|v| matches!(v, GenericTagValue::EventId(_)))
                            }
                            _ => {}
                        }
                    } else if tag.character == Alphabet::P {
                        values.retain(|v| matches!(v, GenericTagValue::Pubkey(_)))
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

fn extend_or_collect<T, I>(mut set: Option<AllocSet<T>>, iter: I) -> Option<AllocSet<T>>
where
    I: IntoIterator<Item = T>,
    T: Eq + Ord + core::hash::Hash,
{
    match set.as_mut() {
        Some(s) => {
            s.extend(iter);
        }
        None => set = Some(iter.into_iter().collect()),
    };
    set
}

/// Remove values from set
/// If after remove the set is empty, will be returned `None`
fn remove_or_none<T, I>(mut set: Option<AllocSet<T>>, iter: I) -> Option<AllocSet<T>>
where
    I: IntoIterator<Item = T>,
    T: Eq + Ord + core::hash::Hash,
{
    if let Some(s) = set.as_mut() {
        for item in iter.into_iter() {
            s.remove(&item);
        }

        if s.is_empty() {
            set = None;
        }
    }
    set
}

#[cfg(test)]
mod test {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_kind_concatenation() {
        let filter = Filter::new()
            .kind(Kind::Metadata)
            .kind(Kind::TextNote)
            .kind(Kind::ContactList)
            .kinds([
                Kind::EncryptedDirectMessage,
                Kind::Metadata,
                Kind::LongFormTextNote,
            ]);
        assert_eq!(
            filter,
            Filter::new().kinds([
                Kind::Metadata,
                Kind::TextNote,
                Kind::ContactList,
                Kind::EncryptedDirectMessage,
                Kind::LongFormTextNote
            ])
        );
    }

    #[test]
    fn test_empty_filter_serialization() {
        let filter = Filter::new().authors([]);
        assert_eq!(filter.as_json(), r#"{"authors":[]}"#);

        let filter = Filter::new().pubkeys([]);
        assert_eq!(filter.as_json(), r##"{"#p":[]}"##);
    }

    #[test]
    fn test_remove() {
        let event_id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();

        // Test remove ids
        let filter = Filter::new().id(EventId::all_zeros()).id(event_id);
        let filter = filter.remove_ids([EventId::all_zeros()]);
        assert_eq!(filter, Filter::new().id(event_id));

        // Test remove #e tag
        let filter = Filter::new().events([EventId::all_zeros(), event_id]);
        let filter = filter.remove_events([EventId::all_zeros()]);
        assert_eq!(filter, Filter::new().event(event_id));
        let filter = filter.remove_events([event_id]);
        assert!(filter.is_empty());

        // Test remove #d tag
        let mut filter = Filter::new().identifier("myidentifier");
        filter = filter.custom_tag(SingleLetterTag::lowercase(Alphabet::D), ["mysecondid"]);
        filter = filter.identifiers(["test", "test2"]);
        filter = filter.remove_custom_tag(SingleLetterTag::lowercase(Alphabet::D), ["test2"]);
        filter = filter.remove_identifiers(["mysecondid"]);
        assert_eq!(filter, Filter::new().identifiers(["myidentifier", "test"]));

        // Test remove custom tag
        let filter =
            Filter::new().custom_tag(SingleLetterTag::lowercase(Alphabet::C), ["test", "test2"]);
        let filter = filter.remove_custom_tag(SingleLetterTag::lowercase(Alphabet::C), ["test2"]);
        assert_eq!(
            filter,
            Filter::new().custom_tag(SingleLetterTag::lowercase(Alphabet::C), ["test"])
        );
    }

    #[test]
    #[cfg(not(feature = "std"))]
    fn test_filter_serialization() {
        let filter = Filter::new()
            .identifier("identifier")
            .search("test")
            .custom_tag(SingleLetterTag::lowercase(Alphabet::J), ["test1"])
            .custom_tag(
                SingleLetterTag::lowercase(Alphabet::P),
                ["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"],
            );
        let json = r##"{"search":"test","#d":["identifier"],"#j":["test1"],"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}"##;
        assert_eq!(filter.as_json(), json.to_string());
    }

    #[test]
    fn test_filter_serialization_with_uppercase_tag() {
        let filter = Filter::new().custom_tag(
            SingleLetterTag::uppercase(Alphabet::P),
            ["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"],
        );
        let json =
            r##"{"#P":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}"##;
        assert_eq!(filter.as_json(), json);
    }

    #[test]
    fn test_filter_deserialization() {
        let json = r##"{"#a":["...", "test"],"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"],"search":"test","ids":["70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5"]}"##;
        let filter = Filter::from_json(json).unwrap();
        let event_id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let pubkey =
            PublicKey::from_str("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();
        assert_eq!(
            filter,
            Filter::new()
                .ids([event_id])
                .search("test")
                .custom_tag(
                    SingleLetterTag::lowercase(Alphabet::A),
                    ["...".to_string(), "test".to_string()]
                )
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
    fn test_filter_is_empty() {
        let filter = Filter::new().identifier("test");
        assert!(!filter.is_empty());

        let filter = Filter::new();
        assert!(filter.is_empty());
    }
}
