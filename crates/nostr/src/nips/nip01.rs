// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP01: Basic protocol flow description
//!
//! <https://github.com/nostr-protocol/nips/blob/master/01.md>

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

use super::nip19::FromBech32;
use super::nip21::NostrURI;
use crate::event::id;
use crate::types::RelayUrl;
use crate::{key, Filter, Kind, PublicKey, Tag, TagStandard};

/// Raw Event error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Keys error
    Keys(key::Error),
    /// Event ID error
    EventId(id::Error),
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
            Self::Keys(e) => write!(f, "Keys: {e}"),
            Self::EventId(e) => write!(f, "Event ID: {e}"),
            Self::ParseInt(e) => write!(f, "Parse Int: {e}"),
            Self::InvalidCoordinate => write!(f, "Invalid coordinate"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<id::Error> for Error {
    fn from(e: id::Error) -> Self {
        Self::EventId(e)
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

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.kind, self.public_key, self.identifier)
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

#[cfg(test)]
mod tests {
    use super::*;

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
