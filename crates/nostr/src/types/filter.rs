// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Filters

#[cfg(not(feature = "std"))]
use alloc::collections::{BTreeMap as AllocMap, BTreeSet as AllocSet};
use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::collections::{HashMap as AllocMap, HashSet as AllocSet};

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use crate::{Event, EventId, JsonUtil, Kind, PublicKey, Timestamp};

type GenericTags = AllocMap<SingleLetterTag, AllocSet<GenericTagValue>>;

/// Alphabet Error
#[derive(Debug)]
pub enum SingleLetterTagError {
    /// Invalid char
    InvalidChar,
    /// Expected char
    ExpectedChar,
}

#[cfg(feature = "std")]
impl std::error::Error for SingleLetterTagError {}

impl fmt::Display for SingleLetterTagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChar => write!(f, "invalid alphabet char"),
            Self::ExpectedChar => write!(f, "Expected char "),
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
    #[inline]
    pub fn lowercase(character: Alphabet) -> Self {
        Self {
            character,
            uppercase: false,
        }
    }

    /// Compose new `uppercase` single-letter tag
    #[inline]
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
    #[inline]
    pub fn is_lowercase(&self) -> bool {
        !self.uppercase
    }

    /// Check if single-letter tag is `uppercase`
    #[inline]
    pub fn is_uppercase(&self) -> bool {
        self.uppercase
    }
}

impl fmt::Display for SingleLetterTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl FromStr for SingleLetterTag {
    type Err = SingleLetterTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1 {
            let c: char = s.chars().next().ok_or(SingleLetterTagError::ExpectedChar)?;
            Self::from_char(c)
        } else {
            Err(SingleLetterTagError::ExpectedChar)
        }
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

/// Generic Tag Value
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GenericTagValue {
    /// Public key
    PublicKey(PublicKey),
    /// Event ID
    EventId(EventId),
    /// Other (string)
    String(String),
}

impl fmt::Display for GenericTagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PublicKey(inner) => write!(f, "{inner}"),
            Self::EventId(inner) => write!(f, "{inner}"),
            Self::String(inner) => write!(f, "{inner}"),
        }
    }
}

impl Serialize for GenericTagValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[allow(missing_docs)]
pub trait IntoGenericTagValue {
    fn into_generic_tag_value(self) -> GenericTagValue;
}

impl IntoGenericTagValue for PublicKey {
    fn into_generic_tag_value(self) -> GenericTagValue {
        GenericTagValue::PublicKey(self)
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

impl IntoGenericTagValue for &String {
    fn into_generic_tag_value(self) -> GenericTagValue {
        GenericTagValue::String(self.clone())
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
    /// Generic tag queries
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
    #[inline]
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
        self.generic_tags.entry(tag).or_default().extend(values);
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

    #[inline]
    fn ids_match(&self, event: &Event) -> bool {
        self.ids
            .as_ref()
            .map_or(true, |ids| ids.is_empty() || ids.contains(&event.id))
    }

    #[inline]
    fn authors_match(&self, event: &Event) -> bool {
        self.authors.as_ref().map_or(true, |authors| {
            authors.is_empty() || authors.contains(&event.pubkey)
        })
    }

    fn tag_match(&self, event: &Event) -> bool {
        if self.generic_tags.is_empty() {
            return true;
        }

        if event.tags.is_empty() {
            return false;
        }

        // Build tags indexes
        let mut idx: AllocMap<SingleLetterTag, AllocSet<GenericTagValue>> = AllocMap::new();
        for (single_letter_tag, content) in event
            .iter_tags()
            .filter_map(|t| Some((t.single_letter_tag()?, t.content()?)))
        {
            idx.entry(single_letter_tag).or_default().insert(content);
        }

        // Match
        self.generic_tags.iter().all(|(tag_name, set)| {
            if let Some(val_set) = idx.get(tag_name) {
                set.iter().any(|t| val_set.contains(t))
            } else {
                false
            }
        })
    }

    #[inline]
    fn kind_match(&self, event: &Event) -> bool {
        self.kinds.as_ref().map_or(true, |kinds| {
            kinds.is_empty() || kinds.contains(&event.kind)
        })
    }

    /// Determine if [Filter] match given [Event].
    ///
    /// The `search` filed is not supported yet!
    #[inline]
    pub fn match_event(&self, event: &Event) -> bool {
        self.ids_match(event)
            && self.authors_match(event)
            && self.kind_match(event)
            && self.since.map_or(true, |t| event.created_at >= t)
            && self.until.map_or(true, |t| event.created_at <= t)
            && self.tag_match(event)
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
        map.serialize_entry(&format!("#{tag}"), values)?;
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
                    let temp_values: AllocSet<String> = map.next_value()?;

                    #[cfg(feature = "std")]
                    let mut values: AllocSet<GenericTagValue> =
                        AllocSet::with_capacity(temp_values.len());
                    #[cfg(not(feature = "std"))]
                    let mut values: AllocSet<GenericTagValue> = AllocSet::new();

                    for v in temp_values.into_iter() {
                        match (tag.character, tag.uppercase) {
                            (Alphabet::E, false) => {
                                let id: EventId =
                                    EventId::from_hex(v).map_err(serde::de::Error::custom)?;
                                values.insert(GenericTagValue::EventId(id));
                            }
                            (Alphabet::P, ..) => {
                                let pk: PublicKey =
                                    PublicKey::from_hex(v).map_err(serde::de::Error::custom)?;
                                values.insert(GenericTagValue::PublicKey(pk));
                            }
                            _ => {
                                values.insert(GenericTagValue::String(v));
                            }
                        }
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
mod tests {
    use bitcoin::secp256k1::schnorr::Signature;

    use super::*;
    use crate::Tag;

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
            )
            .custom_tag(SingleLetterTag::lowercase(Alphabet::Z), ["rating"]);
        let json = r##"{"search":"test","#d":["identifier"],"#j":["test1"],"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"],"#z":["rating"]}"##;
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
        let json = r##"{"#a":["...", "test"],"#e":["70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5"],"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"],"search":"test","ids":["70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5"]}"##;
        let filter = Filter::from_json(json).unwrap();
        let event_id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let pubkey =
            PublicKey::from_hex("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();

        // Check IDs
        assert!(filter.ids.unwrap().contains(&event_id));
        assert_eq!(filter.search, Some(String::from("test")));

        // Check #e tag
        let set = filter
            .generic_tags
            .get(&SingleLetterTag {
                character: Alphabet::E,
                uppercase: false,
            })
            .unwrap();
        assert!(set.contains(&GenericTagValue::EventId(event_id)));

        // Check #p tag
        let set = filter
            .generic_tags
            .get(&SingleLetterTag {
                character: Alphabet::P,
                uppercase: false,
            })
            .unwrap();
        assert!(set.contains(&GenericTagValue::PublicKey(pubkey)));

        // Check #a tag
        let set = filter
            .generic_tags
            .get(&SingleLetterTag {
                character: Alphabet::A,
                uppercase: false,
            })
            .unwrap();
        assert!(set.contains(&GenericTagValue::String(String::from("..."))));
        assert!(set.contains(&GenericTagValue::String(String::from("test"))));

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

    #[test]
    fn test_match_event() {
        let event_id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let pubkey =
            PublicKey::from_str("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();
        let event =
            Event::new(
                event_id,
                pubkey,
                Timestamp::from(1612809991),
                Kind::TextNote,
                [
                    Tag::public_key(PublicKey::from_str("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a").unwrap()),
                    Tag::event(EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96").unwrap()),
                ],
                "test",
                Signature::from_str("273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502").unwrap(),
            );
        let event_with_empty_tags: Event = Event::new(
            event_id,
            pubkey,
            Timestamp::from(1612809992),
            Kind::TextNote,
            [],
            "test",
            Signature::from_str("273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502").unwrap(),
          );

        // ID match
        let filter: Filter = Filter::new().id(event_id);
        assert!(filter.match_event(&event));

        // Not match (kind)
        let filter: Filter = Filter::new().id(event_id).kind(Kind::Metadata);
        assert!(!filter.match_event(&event));

        // Match (author, kind and since)
        let filter: Filter = Filter::new()
            .author(pubkey)
            .kind(Kind::TextNote)
            .since(Timestamp::from(1612808000));
        assert!(filter.match_event(&event));

        // Not match (since)
        let filter: Filter = Filter::new()
            .author(pubkey)
            .kind(Kind::TextNote)
            .since(Timestamp::from(1700000000));
        assert!(!filter.match_event(&event));

        // Match (#p tag and kind)
        let filter: Filter = Filter::new()
            .pubkey(
                PublicKey::from_str(
                    "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
                )
                .unwrap(),
            )
            .kind(Kind::TextNote);
        assert!(filter.match_event(&event));

        // Match (tags)
        let filter: Filter = Filter::new()
            .pubkey(
                PublicKey::from_str(
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
        let filter: Filter = Filter::new().events(vec![
            EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96")
                .unwrap(),
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap(),
        ]);
        assert!(filter.match_event(&event));

        // Not match (tags)
        let filter: Filter = Filter::new().events(vec![EventId::from_hex(
            "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5",
        )
        .unwrap()]);
        assert!(!filter.match_event(&event));

        // Not match (tags filter for events with empty tags)
        let filter: Filter = Filter::new().hashtag("this-should-not-match");
        assert!(!filter.match_event(&event));
        assert!(!filter.match_event(&event_with_empty_tags));
    }
}

#[cfg(bench)]
mod benches {
    use core::str::FromStr;

    use bitcoin::secp256k1::schnorr::Signature;
    use test::{black_box, Bencher};

    use super::*;
    use crate::Tag;

    #[bench]
    pub fn filter_match_event(bh: &mut Bencher) {
        // Event
        let event =
            Event::new(
                EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap(),
                PublicKey::from_hex("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap(),
                Timestamp::from(1612809991),
                Kind::TextNote,
                [
                    Tag::public_key(PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a").unwrap()),
                    Tag::event(EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96").unwrap()),
                ],
                "test",
                Signature::from_str("273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502").unwrap(),
            );

        // Filter
        let pk =
            PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
                .unwrap();
        let filter = Filter::new().pubkey(pk).kind(Kind::TextNote);

        bh.iter(|| {
            black_box(filter.match_event(&event));
        });
    }
}
