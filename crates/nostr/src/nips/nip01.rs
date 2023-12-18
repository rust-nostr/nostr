// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP01
//!
//! <https://github.com/nostr-protocol/nips/blob/master/01.md>

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::secp256k1::{self, XOnlyPublicKey};

use crate::event::id;
use crate::nips::nip19::{
    Error as Bech32Error, FromBech32, ToBech32, AUTHOR, KIND,
    PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT, RELAY, SPECIAL,
};
use crate::{Filter, Kind, Tag, UncheckedUrl};

/// [`RawEvent`] error
#[derive(Debug)]
pub enum Error {
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
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
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::EventId(e) => write!(f, "Event ID: {e}"),
            Self::ParseInt(e) => write!(f, "Parse Int: {e}"),
            Self::InvalidCoordinate => write!(f, "Invalid coordinate"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
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
    pub pubkey: XOnlyPublicKey,
    /// `d` tag identifier
    ///
    /// Needed for a parametrized replaceable event.
    /// Leave empty for a replaceable event.
    pub identifier: String,
    /// Relays
    pub relays: Vec<String>,
}

impl Coordinate {
    /// Create new event coordinate
    pub fn new(kind: Kind, pubkey: XOnlyPublicKey) -> Self {
        Self {
            kind,
            pubkey,
            identifier: String::new(),
            relays: Vec::new(),
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
}

impl From<Coordinate> for Tag {
    fn from(value: Coordinate) -> Self {
        Self::A {
            kind: value.kind,
            public_key: value.pubkey,
            identifier: value.identifier,
            relay_url: value.relays.first().map(UncheckedUrl::from),
        }
    }
}

impl From<Coordinate> for Filter {
    fn from(value: Coordinate) -> Self {
        if value.identifier.is_empty() {
            Filter::new().kind(value.kind).author(value.pubkey)
        } else {
            Filter::new()
                .kind(value.kind)
                .author(value.pubkey)
                .identifier(value.identifier)
        }
    }
}

impl FromStr for Coordinate {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut kpi = s.split(':');
        if let (Some(kind_str), Some(pubkey_str), Some(identifier)) =
            (kpi.next(), kpi.next(), kpi.next())
        {
            Ok(Self {
                kind: Kind::from_str(kind_str)?,
                pubkey: XOnlyPublicKey::from_str(pubkey_str)?,
                identifier: identifier.to_owned(),
                relays: Vec::new(),
            })
        } else {
            Err(Error::InvalidCoordinate)
        }
    }
}

impl FromBech32 for Coordinate {
    type Err = Bech32Error;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: AsRef<str>,
    {
        let (hrp, data, checksum) = bech32::decode(s.as_ref())?;

        if hrp != PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT || checksum != Variant::Bech32 {
            return Err(Bech32Error::WrongPrefixOrVariant);
        }

        let mut data: Vec<u8> = Vec::from_base32(&data)?;

        let mut identifier: Option<String> = None;
        let mut pubkey: Option<XOnlyPublicKey> = None;
        let mut kind: Option<Kind> = None;
        let mut relays: Vec<String> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Bech32Error::TLV)?;
            let l = data.get(1).ok_or(Bech32Error::TLV)?;
            let l = *l as usize;

            let bytes = data.get(2..l + 2).ok_or(Bech32Error::TLV)?;

            match *t {
                SPECIAL => {
                    if identifier.is_none() {
                        identifier = Some(String::from_utf8(bytes.to_vec())?);
                    }
                }
                RELAY => {
                    relays.push(String::from_utf8(bytes.to_vec())?);
                }
                AUTHOR => {
                    if pubkey.is_none() {
                        pubkey = Some(XOnlyPublicKey::from_slice(bytes)?);
                    }
                }
                KIND => {
                    if kind.is_none() {
                        let k: u64 = u32::from_be_bytes(
                            bytes.try_into().map_err(|_| Bech32Error::TryFromSlice)?,
                        ) as u64;
                        kind = Some(Kind::from(k));
                    }
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            kind: kind.ok_or_else(|| Bech32Error::FieldMissing("kind".to_string()))?,
            pubkey: pubkey.ok_or_else(|| Bech32Error::FieldMissing("pubkey".to_string()))?,
            identifier: identifier
                .ok_or_else(|| Bech32Error::FieldMissing("identifier".to_string()))?,
            relays,
        })
    }
}

impl ToBech32 for Coordinate {
    type Err = Bech32Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = Vec::new();

        // Identifier
        bytes.extend([SPECIAL, self.identifier.len() as u8]);
        bytes.extend(self.identifier.as_bytes());

        for relay in self.relays.iter() {
            bytes.extend([RELAY, relay.len() as u8]);
            bytes.extend(relay.as_bytes());
        }

        // Author
        bytes.extend([AUTHOR, 32]);
        bytes.extend(self.pubkey.serialize());

        // Kind
        bytes.extend([KIND, 4]);
        bytes.extend(self.kind.as_u32().to_be_bytes());

        let data = bytes.to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT,
            data,
            Variant::Bech32,
        )?)
    }
}
