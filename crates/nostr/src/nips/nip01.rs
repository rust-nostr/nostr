// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP01: Basic protocol flow description
//!
//! <https://github.com/nostr-protocol/nips/blob/master/01.md>

use alloc::borrow::ToOwned;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as AllocMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::collections::HashMap as AllocMap;

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::nip19::FromBech32;
use super::nip21::NostrURI;
use crate::types::{RelayUrl, Url};
use crate::{event, key, Filter, JsonUtil, Kind, PublicKey, Tag, TagStandard};

/// Raw Event error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Keys error
    Keys(key::Error),
    /// Event ID error
    Event(event::Error),
    /// Parse Int error
    ParseInt(ParseIntError),
    /// Invalid coordinate
    InvalidCoordinate,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keys(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::ParseInt(e) => write!(f, "{e}"),
            Self::InvalidCoordinate => write!(f, "Invalid coordinate"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

/// Coordinate for event (`a` tag)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Coordinate {
    /// Kind
    pub kind: Kind,
    /// Public Key
    pub public_key: PublicKey,
    /// `d` tag identifier
    ///
    /// Needed for a parametrized replaceable event.
    /// Leave empty for a replaceable event.
    pub identifier: String,
    /// Relays
    pub relays: Vec<RelayUrl>,
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.kind, self.public_key, self.identifier)
    }
}

impl Coordinate {
    /// Create new event coordinate
    #[inline]
    pub fn new(kind: Kind, public_key: PublicKey) -> Self {
        Self {
            kind,
            public_key,
            identifier: String::new(),
            relays: Vec::new(),
        }
    }

    /// Parse coordinate from `<kind>:<pubkey>:[<d-tag>]` format, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    pub fn parse<S>(coordinate: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let coordinate: &str = coordinate.as_ref();

        // Try from hex
        if let Ok(coordinate) = Self::from_kpi_format(coordinate) {
            return Ok(coordinate);
        }

        // Try from bech32
        if let Ok(coordinate) = Self::from_bech32(coordinate) {
            return Ok(coordinate);
        }

        // Try from NIP21 URI
        if let Ok(coordinate) = Self::from_nostr_uri(coordinate) {
            return Ok(coordinate);
        }

        Err(Error::InvalidCoordinate)
    }

    /// Try to parse from `<kind>:<pubkey>:[<d-tag>]` format
    pub fn from_kpi_format<S>(coordinate: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let coordinate: &str = coordinate.as_ref();
        let mut kpi = coordinate.split(':');
        if let (Some(kind_str), Some(public_key_str), Some(identifier)) =
            (kpi.next(), kpi.next(), kpi.next())
        {
            Ok(Self {
                kind: Kind::from_str(kind_str)?,
                public_key: PublicKey::from_hex(public_key_str)?,
                identifier: identifier.to_owned(),
                relays: Vec::new(),
            })
        } else {
            Err(Error::InvalidCoordinate)
        }
    }

    /// Set a `d` tag identifier
    ///
    /// Needed for a parametrized replaceable event.
    pub fn identifier<S>(mut self, identifier: S) -> Self
    where
        S: Into<String>,
    {
        self.identifier = identifier.into();
        self
    }

    /// Check if coordinate has identifier
    #[inline]
    pub fn has_identifier(&self) -> bool {
        !self.identifier.is_empty()
    }

    /// Borrow coordinate
    pub fn borrow(&self) -> CoordinateBorrow<'_> {
        CoordinateBorrow {
            kind: &self.kind,
            public_key: &self.public_key,
            identifier: Some(&self.identifier),
        }
    }
}

impl From<Coordinate> for Tag {
    fn from(coordinate: Coordinate) -> Self {
        Self::from_standardized(TagStandard::Coordinate {
            relay_url: coordinate.relays.first().cloned(),
            coordinate,
            uppercase: false,
        })
    }
}

impl From<Coordinate> for Filter {
    fn from(value: Coordinate) -> Self {
        if value.identifier.is_empty() {
            Filter::new().kind(value.kind).author(value.public_key)
        } else {
            Filter::new()
                .kind(value.kind)
                .author(value.public_key)
                .identifier(value.identifier)
        }
    }
}

impl From<&Coordinate> for Filter {
    fn from(value: &Coordinate) -> Self {
        if value.identifier.is_empty() {
            Filter::new().kind(value.kind).author(value.public_key)
        } else {
            Filter::new()
                .kind(value.kind)
                .author(value.public_key)
                .identifier(value.identifier.clone())
        }
    }
}

impl FromStr for Coordinate {
    type Err = Error;

    /// Try to parse [Coordinate] from `<kind>:<pubkey>:[<d-tag>]` format, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    #[inline]
    fn from_str(coordinate: &str) -> Result<Self, Self::Err> {
        Self::parse(coordinate)
    }
}

/// Borrowed coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordinateBorrow<'a> {
    /// Kind
    pub kind: &'a Kind,
    /// Public key
    pub public_key: &'a PublicKey,
    /// `d` tag identifier
    ///
    /// Needed for a parametrized replaceable event.
    pub identifier: Option<&'a str>,
}

impl CoordinateBorrow<'_> {
    /// Into owned coordinate
    pub fn into_owned(self) -> Coordinate {
        Coordinate {
            kind: *self.kind,
            public_key: *self.public_key,
            identifier: self.identifier.map(|s| s.to_string()).unwrap_or_default(),
            relays: Vec::new(),
        }
    }
}

/// Metadata
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    /// Name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub name: Option<String>,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub display_name: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub about: Option<String>,
    /// Website url
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub website: Option<String>,
    /// Picture url
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub picture: Option<String>,
    /// Banner url
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub banner: Option<String>,
    /// NIP05 (ex. name@example.com)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub nip05: Option<String>,
    /// LNURL
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub lud06: Option<String>,
    /// Lightning Address
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub lud16: Option<String>,
    /// Custom fields
    #[serde(
        flatten,
        serialize_with = "serialize_custom_fields",
        deserialize_with = "deserialize_custom_fields"
    )]
    #[serde(default)]
    pub custom: AllocMap<String, Value>,
}

impl Metadata {
    /// New empty [`Metadata`]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set name
    pub fn name<S>(self, name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    /// Set display name
    pub fn display_name<S>(self, display_name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            display_name: Some(display_name.into()),
            ..self
        }
    }

    /// Set about
    pub fn about<S>(self, about: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            about: Some(about.into()),
            ..self
        }
    }

    /// Set website
    pub fn website(self, url: Url) -> Self {
        Self {
            website: Some(url.into()),
            ..self
        }
    }

    /// Set picture
    pub fn picture(self, url: Url) -> Self {
        Self {
            picture: Some(url.into()),
            ..self
        }
    }

    /// Set banner
    pub fn banner(self, url: Url) -> Self {
        Self {
            banner: Some(url.into()),
            ..self
        }
    }

    /// Set nip05
    pub fn nip05<S>(self, nip05: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            nip05: Some(nip05.into()),
            ..self
        }
    }

    /// Set lud06 (LNURL)
    pub fn lud06<S>(self, lud06: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            lud06: Some(lud06.into()),
            ..self
        }
    }

    /// Set lud16 (Lightning Address)
    pub fn lud16<S>(self, lud16: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            lud16: Some(lud16.into()),
            ..self
        }
    }

    /// Set custom metadata field
    pub fn custom_field<K, S>(mut self, field_name: K, value: S) -> Self
    where
        K: Into<String>,
        S: Into<Value>,
    {
        self.custom.insert(field_name.into(), value.into());
        self
    }
}

impl JsonUtil for Metadata {
    type Err = serde_json::Error;
}

fn serialize_custom_fields<S>(
    custom_fields: &AllocMap<String, Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(custom_fields.len()))?;
    for (field_name, value) in custom_fields {
        map.serialize_entry(field_name, value)?;
    }
    map.end()
}

fn deserialize_custom_fields<'de, D>(deserializer: D) -> Result<AllocMap<String, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    struct GenericTagsVisitor;

    impl<'de> Visitor<'de> for GenericTagsVisitor {
        type Value = AllocMap<String, Value>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map where keys are strings and values are valid json")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            #[cfg(not(feature = "std"))]
            let mut custom_fields: AllocMap<String, Value> = AllocMap::new();
            #[cfg(feature = "std")]
            let mut custom_fields: AllocMap<String, Value> =
                AllocMap::with_capacity(map.size_hint().unwrap_or_default());
            while let Some(field_name) = map.next_key::<String>()? {
                if let Ok(value) = map.next_value::<Value>() {
                    custom_fields.insert(field_name, value);
                }
            }
            Ok(custom_fields)
        }
    }

    deserializer.deserialize_map(GenericTagsVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_metadata() {
        let content = r#"{"name":"myname","about":"Description","display_name":""}"#;
        let metadata = Metadata::from_json(content).unwrap();
        assert_eq!(
            metadata,
            Metadata::new()
                .name("myname")
                .about("Description")
                .display_name("")
        );

        let content = r#"{"name":"myname","about":"Description","displayName":"Jack"}"#;
        let metadata = Metadata::from_json(content).unwrap();
        assert_eq!(
            metadata,
            Metadata::new()
                .name("myname")
                .about("Description")
                .custom_field("displayName", "Jack")
        );

        let content = r#"{"lud16":"thesimplekid@cln.thesimplekid.com","nip05":"_@thesimplekid.com","display_name":"thesimplekid","about":"Wannabe open source dev","name":"thesimplekid","username":"thesimplekid","displayName":"thesimplekid","lud06":"","reactions":false,"damus_donation_v2":0}"#;
        let metadata = Metadata::from_json(content).unwrap();
        assert_eq!(
            metadata,
            Metadata::new()
                .name("thesimplekid")
                .display_name("thesimplekid")
                .about("Wannabe open source dev")
                .nip05("_@thesimplekid.com")
                .lud06("")
                .lud16("thesimplekid@cln.thesimplekid.com")
                .custom_field("username", "thesimplekid")
                .custom_field("displayName", "thesimplekid")
                .custom_field("reactions", false)
                .custom_field("damus_donation_v2", 0)
        );
        assert_eq!(metadata, Metadata::from_json(metadata.as_json()).unwrap());
    }

    #[test]
    fn parse_valid_coordinate() {
        let coordinate: &str =
            "30023:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:ipsum";
        let coordinate: Coordinate = Coordinate::parse(coordinate).unwrap();

        let expected_public_key: PublicKey =
            PublicKey::from_hex("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();

        assert_eq!(coordinate.kind.as_u16(), 30023);
        assert_eq!(coordinate.public_key, expected_public_key);
        assert_eq!(coordinate.identifier, "ipsum");

        let coordinate: &str =
            "20500:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:";
        let coordinate: Coordinate = Coordinate::parse(coordinate).unwrap();

        assert_eq!(coordinate.kind.as_u16(), 20500);
        assert_eq!(coordinate.public_key, expected_public_key);
        assert_eq!(coordinate.identifier, "");
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn parse_coordinate(bh: &mut Bencher) {
        let coordinate: &str =
            "30023:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:ipsum";
        bh.iter(|| {
            black_box(Coordinate::parse(coordinate)).unwrap();
        });
    }
}
