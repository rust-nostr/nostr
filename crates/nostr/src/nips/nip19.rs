// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP19: bech32-encoded entities
//!
//! <https://github.com/nostr-protocol/nips/blob/master/19.md>

#![allow(missing_docs)]

use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bech32::{self, Bech32, Hrp};

use super::nip01::Coordinate;
#[cfg(all(feature = "std", feature = "nip05"))]
use super::nip05::Nip05Profile;
#[cfg(feature = "nip49")]
use super::nip49::{self, EncryptedSecretKey};
use crate::event::id::EventId;
use crate::types::url::{self, RelayUrl, TryIntoUrl};
use crate::{event, key, Kind, PublicKey, SecretKey};

pub const PREFIX_BECH32_SECRET_KEY: &str = "nsec";
pub const PREFIX_BECH32_SECRET_KEY_ENCRYPTED: &str = "ncryptsec";
pub const PREFIX_BECH32_PUBLIC_KEY: &str = "npub";
pub const PREFIX_BECH32_NOTE_ID: &str = "note";
pub const PREFIX_BECH32_PROFILE: &str = "nprofile";
pub const PREFIX_BECH32_EVENT: &str = "nevent";
pub const PREFIX_BECH32_COORDINATE: &str = "naddr";

const HRP_SECRET_KEY: Hrp = Hrp::parse_unchecked(PREFIX_BECH32_SECRET_KEY);
#[cfg(feature = "nip49")]
const HRP_SECRET_KEY_ENCRYPTED: Hrp = Hrp::parse_unchecked(PREFIX_BECH32_SECRET_KEY_ENCRYPTED);
const HRP_PUBLIC_KEY: Hrp = Hrp::parse_unchecked(PREFIX_BECH32_PUBLIC_KEY);
const HRP_NOTE_ID: Hrp = Hrp::parse_unchecked(PREFIX_BECH32_NOTE_ID);
const HRP_PROFILE: Hrp = Hrp::parse_unchecked(PREFIX_BECH32_PROFILE);
const HRP_EVENT: Hrp = Hrp::parse_unchecked(PREFIX_BECH32_EVENT);
const HRP_COORDINATE: Hrp = Hrp::parse_unchecked(PREFIX_BECH32_COORDINATE);

pub const SPECIAL: u8 = 0;
pub const RELAY: u8 = 1;
pub const AUTHOR: u8 = 2;
pub const KIND: u8 = 3;

/// 1 (type) + 1 (len) + 32 (value)
const FIXED_1_1_32_BYTES_TVL: usize = 1 + 1 + 32;

/// 1 (type) + 1 (len) + 4 (value - 32-bit unsigned number)
const FIXED_KIND_BYTES_TVL: usize = 1 + 1 + 4;

/// `NIP19` error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Relay Url parse error
    RelayUrl(url::Error),
    /// Bech32 decode error.
    Bech32Decode(bech32::DecodeError),
    /// Bech32 encode error
    Bech32Encode(bech32::EncodeError),
    /// Keys error
    Keys(key::Error),
    /// Event error
    Event(event::Error),
    /// NIP49 error
    #[cfg(feature = "nip49")]
    NIP49(nip49::Error),
    /// Wrong prefix or variant
    WrongPrefix,
    /// Field missing
    FieldMissing(String),
    /// TLV error
    TLV,
    /// From slice error
    TryFromSlice,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RelayUrl(e) => write!(f, "{e}"),
            Self::Bech32Decode(e) => write!(f, "{e}"),
            Self::Bech32Encode(e) => write!(f, "{e}"),
            Self::Keys(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            #[cfg(feature = "nip49")]
            Self::NIP49(e) => write!(f, "{e}"),
            Self::WrongPrefix => write!(f, "Wrong prefix"),
            Self::FieldMissing(name) => write!(f, "Field missing: {name}"),
            Self::TLV => write!(f, "TLV error"),
            Self::TryFromSlice => write!(f, "From slice error"),
        }
    }
}

impl From<url::Error> for Error {
    fn from(e: url::Error) -> Self {
        Self::RelayUrl(e)
    }
}

impl From<bech32::DecodeError> for Error {
    fn from(e: bech32::DecodeError) -> Self {
        Self::Bech32Decode(e)
    }
}

impl From<bech32::EncodeError> for Error {
    fn from(e: bech32::EncodeError) -> Self {
        Self::Bech32Encode(e)
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

#[cfg(feature = "nip49")]
impl From<nip49::Error> for Error {
    fn from(e: nip49::Error) -> Self {
        Self::NIP49(e)
    }
}

/// To ensure total matching on prefixes when decoding a [`Nip19`] object
enum Nip19Prefix {
    /// Secret Key
    NSec,
    /// Encrypted Secret Key
    #[cfg(feature = "nip49")]
    NCryptSec,
    /// Public key
    NPub,
    /// note
    Note,
    /// nprofile
    NProfile,
    /// nevent
    NEvent,
    /// naddr
    NAddr,
}

impl FromStr for Nip19Prefix {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            PREFIX_BECH32_SECRET_KEY => Ok(Nip19Prefix::NSec),
            #[cfg(feature = "nip49")]
            PREFIX_BECH32_SECRET_KEY_ENCRYPTED => Ok(Nip19Prefix::NCryptSec),
            PREFIX_BECH32_PUBLIC_KEY => Ok(Nip19Prefix::NPub),
            PREFIX_BECH32_NOTE_ID => Ok(Nip19Prefix::Note),
            PREFIX_BECH32_PROFILE => Ok(Nip19Prefix::NProfile),
            PREFIX_BECH32_EVENT => Ok(Nip19Prefix::NEvent),
            PREFIX_BECH32_COORDINATE => Ok(Nip19Prefix::NAddr),
            _ => Err(Error::WrongPrefix),
        }
    }
}

/// A representation any `NIP19` bech32 nostr object. Useful for decoding
/// `NIP19` bech32 strings without necessarily knowing what you're decoding
/// ahead of time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nip19 {
    /// nsec
    Secret(SecretKey),
    /// Encrypted Secret Key (ncryptsec)
    #[cfg(feature = "nip49")]
    EncryptedSecret(EncryptedSecretKey),
    /// npub
    Pubkey(PublicKey),
    /// nprofile
    Profile(Nip19Profile),
    /// note
    EventId(EventId),
    /// nevent
    Event(Nip19Event),
    /// naddr
    Coordinate(Coordinate),
}

pub trait FromBech32: Sized {
    type Err;
    fn from_bech32(bech32: &str) -> Result<Self, Self::Err>;
}

pub trait ToBech32 {
    type Err;
    fn to_bech32(&self) -> Result<String, Self::Err>;
}

impl FromBech32 for Nip19 {
    type Err = Error;

    fn from_bech32(hash: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(hash)?;
        let prefix: Nip19Prefix = Nip19Prefix::from_str(hrp.as_str())?;

        match prefix {
            Nip19Prefix::NSec => Ok(Self::Secret(SecretKey::from_slice(data.as_slice())?)),
            #[cfg(feature = "nip49")]
            Nip19Prefix::NCryptSec => Ok(Self::EncryptedSecret(EncryptedSecretKey::from_slice(
                data.as_slice(),
            )?)),
            Nip19Prefix::NPub => Ok(Self::Pubkey(PublicKey::from_slice(data.as_slice())?)),
            Nip19Prefix::NProfile => Ok(Self::Profile(Nip19Profile::from_bech32_data(data)?)),
            Nip19Prefix::NEvent => Ok(Self::Event(Nip19Event::from_bech32_data(data)?)),
            Nip19Prefix::Note => Ok(Self::EventId(EventId::from_slice(data.as_slice())?)),
            Nip19Prefix::NAddr => Ok(Self::Coordinate(Coordinate::from_bech32_data(data)?)),
        }
    }
}

impl ToBech32 for Nip19 {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        match self {
            Nip19::Secret(sec) => sec.to_bech32(),
            #[cfg(feature = "nip49")]
            Nip19::EncryptedSecret(cryptsec) => cryptsec.to_bech32(),
            Nip19::Pubkey(pubkey) => pubkey.to_bech32(),
            Nip19::Event(event) => event.to_bech32(),
            Nip19::Profile(profile) => profile.to_bech32(),
            Nip19::EventId(event_id) => event_id.to_bech32(),
            Nip19::Coordinate(coordinate) => coordinate.to_bech32(),
        }
    }
}

impl FromBech32 for SecretKey {
    type Err = Error;

    fn from_bech32(secret_key: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(secret_key)?;

        if hrp != HRP_SECRET_KEY {
            return Err(Error::WrongPrefix);
        }

        Ok(Self::from_slice(data.as_slice())?)
    }
}

impl ToBech32 for SecretKey {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        Ok(bech32::encode::<Bech32>(
            HRP_SECRET_KEY,
            self.as_secret_bytes(),
        )?)
    }
}

#[cfg(feature = "nip49")]
impl FromBech32 for EncryptedSecretKey {
    type Err = Error;

    fn from_bech32(secret_key: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(secret_key)?;

        if hrp != HRP_SECRET_KEY_ENCRYPTED {
            return Err(Error::WrongPrefix);
        }

        Ok(Self::from_slice(data.as_slice())?)
    }
}

#[cfg(feature = "nip49")]
impl ToBech32 for EncryptedSecretKey {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        Ok(bech32::encode::<Bech32>(
            HRP_SECRET_KEY_ENCRYPTED,
            &self.as_vec(),
        )?)
    }
}

impl FromBech32 for PublicKey {
    type Err = Error;

    fn from_bech32(public_key: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(public_key)?;

        if hrp != HRP_PUBLIC_KEY {
            return Err(Error::WrongPrefix);
        }

        Ok(Self::from_slice(data.as_slice())?)
    }
}

impl ToBech32 for PublicKey {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        Ok(bech32::encode::<Bech32>(HRP_PUBLIC_KEY, self.as_bytes())?)
    }
}

impl FromBech32 for EventId {
    type Err = Error;

    fn from_bech32(id: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(id)?;

        if hrp != HRP_NOTE_ID {
            return Err(Error::WrongPrefix);
        }

        Ok(Self::from_slice(data.as_slice())?)
    }
}

impl ToBech32 for EventId {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        Ok(bech32::encode::<Bech32>(HRP_NOTE_ID, self.as_bytes())?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nip19Event {
    pub event_id: EventId,
    pub author: Option<PublicKey>,
    pub kind: Option<Kind>,
    pub relays: Vec<String>,
}

impl Nip19Event {
    pub fn new<I, S>(event_id: EventId, relays: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            event_id,
            author: None,
            kind: None,
            relays: relays.into_iter().map(|u| u.into()).collect(),
        }
    }

    /// Add author
    #[inline]
    pub fn author(mut self, author: PublicKey) -> Self {
        self.author = Some(author);
        self
    }

    /// Add kind
    #[inline]
    pub fn kind(mut self, kind: Kind) -> Self {
        self.kind = Some(kind);
        self
    }

    fn from_bech32_data(mut data: Vec<u8>) -> Result<Self, Error> {
        let mut event_id: Option<EventId> = None;
        let mut author: Option<PublicKey> = None;
        let mut kind: Option<Kind> = None;
        let mut relays: Vec<String> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Error::TLV)?;
            let l = data.get(1).ok_or(Error::TLV)?;
            let l = *l as usize;

            let bytes = data.get(2..l + 2).ok_or(Error::TLV)?;

            match *t {
                SPECIAL => {
                    if event_id.is_none() {
                        event_id = Some(EventId::from_slice(bytes)?);
                    }
                }
                // from nip19: "for nevent, *optionally*, the 32 bytes of
                // the pubkey of the event"
                AUTHOR => {
                    if author.is_none() {
                        author = PublicKey::from_slice(bytes).ok(); // NOT propagate error if public key is invalid
                    }
                }
                RELAY => {
                    relays.push(String::from_utf8_lossy(bytes).to_string());
                }
                KIND => {
                    if kind.is_none() {
                        // The kind value must be a 32-bit unsigned number according to
                        // https://github.com/nostr-protocol/nips/blob/37f6cbb775126b386414220f783ca0f5f85e7614/19.md#shareable-identifiers-with-extra-metadata
                        let k: u16 =
                            u32::from_be_bytes(bytes.try_into().map_err(|_| Error::TryFromSlice)?)
                                as u16;
                        kind = Some(Kind::from(k));
                    }
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            event_id: event_id.ok_or_else(|| Error::FieldMissing("event id".to_string()))?,
            author,
            kind,
            relays,
        })
    }
}

impl FromBech32 for Nip19Event {
    type Err = Error;

    fn from_bech32(event: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(event)?;

        if hrp != HRP_EVENT {
            return Err(Error::WrongPrefix);
        }

        Self::from_bech32_data(data)
    }
}

impl ToBech32 for Nip19Event {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        // Allocate capacity
        let relays_len: usize = self.relays.iter().map(|u| 2 + u.len()).sum();
        let author_len: usize = if self.author.is_some() {
            FIXED_1_1_32_BYTES_TVL
        } else {
            0
        };
        let mut bytes: Vec<u8> =
            Vec::with_capacity(FIXED_1_1_32_BYTES_TVL + author_len + relays_len);

        bytes.push(SPECIAL); // Type
        bytes.push(32); // Len
        bytes.extend(self.event_id.as_bytes()); // Value

        if let Some(author) = &self.author {
            bytes.push(AUTHOR); // Type
            bytes.push(32); // Len
            bytes.extend(author.to_bytes()); // Value
        }

        if let Some(kind) = &self.kind {
            bytes.push(KIND); // Type
            bytes.push(4); // Len
            bytes.extend((kind.as_u16() as u32).to_be_bytes()); // Value
        }

        for relay in self.relays.iter() {
            bytes.push(RELAY); // Type
            bytes.push(relay.len() as u8); // Len
            bytes.extend(relay.as_bytes()); // Value
        }

        Ok(bech32::encode::<Bech32>(HRP_EVENT, &bytes)?)
    }
}

#[cfg(all(feature = "std", feature = "nip05"))]
impl ToBech32 for Nip05Profile {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        // Convert to NIP19 profile
        let profile: Nip19Profile = Nip19Profile {
            public_key: self.public_key,
            relays: self.relays.clone(),
        };
        // Encode
        profile.to_bech32()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nip19Profile {
    pub public_key: PublicKey,
    pub relays: Vec<RelayUrl>,
}

impl Nip19Profile {
    pub fn new<I, U>(public_key: PublicKey, relays: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(Self {
            public_key,
            relays: relays
                .into_iter()
                .map(|u| u.try_into_url())
                .collect::<Result<_, _>>()?,
        })
    }

    fn from_bech32_data(mut data: Vec<u8>) -> Result<Self, Error> {
        let mut public_key: Option<PublicKey> = None;
        let mut relays: Vec<RelayUrl> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Error::TLV)?;
            let l = data.get(1).ok_or(Error::TLV)?;
            let l = *l as usize;

            let bytes = data.get(2..l + 2).ok_or(Error::TLV)?;

            match *t {
                SPECIAL => {
                    if public_key.is_none() {
                        public_key = Some(PublicKey::from_slice(bytes)?);
                    }
                }
                RELAY => {
                    let url: Cow<str> = String::from_utf8_lossy(bytes);
                    if let Ok(url) = RelayUrl::parse(&url) {
                        relays.push(url);
                    }
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            public_key: public_key.ok_or_else(|| Error::FieldMissing("pubkey".to_string()))?,
            relays,
        })
    }
}

impl ToBech32 for Nip19Profile {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        // Allocate capacity
        let relays_len: usize = self.relays.iter().map(|u| 2 + u.as_str().len()).sum();
        let mut bytes: Vec<u8> = Vec::with_capacity(FIXED_1_1_32_BYTES_TVL + relays_len);

        bytes.push(SPECIAL); // Type
        bytes.push(32); // Len
        bytes.extend(self.public_key.as_bytes()); // Value

        for relay in self.relays.iter() {
            let url: &[u8] = relay.as_str().as_bytes();
            bytes.push(RELAY); // Type
            bytes.push(url.len() as u8); // Len
            bytes.extend(url); // Value
        }

        Ok(bech32::encode::<Bech32>(HRP_PROFILE, &bytes)?)
    }
}

impl FromBech32 for Nip19Profile {
    type Err = Error;

    fn from_bech32(profile: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(profile)?;

        if hrp != HRP_PROFILE {
            return Err(Error::WrongPrefix);
        }

        Self::from_bech32_data(data)
    }
}

impl Coordinate {
    fn from_bech32_data(mut data: Vec<u8>) -> Result<Self, Error> {
        let mut identifier: Option<String> = None;
        let mut pubkey: Option<PublicKey> = None;
        let mut kind: Option<Kind> = None;
        let mut relays: Vec<RelayUrl> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Error::TLV)?;
            let l = data.get(1).ok_or(Error::TLV)?;
            let l = *l as usize;

            let bytes: &[u8] = data.get(2..l + 2).ok_or(Error::TLV)?;

            match *t {
                SPECIAL => {
                    if identifier.is_none() {
                        identifier = Some(String::from_utf8_lossy(bytes).to_string());
                    }
                }
                RELAY => {
                    let url: Cow<str> = String::from_utf8_lossy(bytes);
                    if let Ok(url) = RelayUrl::parse(&url) {
                        relays.push(url);
                    }
                }
                AUTHOR => {
                    if pubkey.is_none() {
                        pubkey = Some(PublicKey::from_slice(bytes)?);
                    }
                }
                KIND => {
                    if kind.is_none() {
                        // The kind value must be a 32-bit unsigned number according to
                        // https://github.com/nostr-protocol/nips/blob/37f6cbb775126b386414220f783ca0f5f85e7614/19.md#shareable-identifiers-with-extra-metadata
                        let k: u16 =
                            u32::from_be_bytes(bytes.try_into().map_err(|_| Error::TryFromSlice)?)
                                as u16;
                        kind = Some(Kind::from(k));
                    }
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            kind: kind.ok_or_else(|| Error::FieldMissing("kind".to_string()))?,
            public_key: pubkey.ok_or_else(|| Error::FieldMissing("pubkey".to_string()))?,
            identifier: identifier.ok_or_else(|| Error::FieldMissing("identifier".to_string()))?,
            relays,
        })
    }
}

impl FromBech32 for Coordinate {
    type Err = Error;

    fn from_bech32(addr: &str) -> Result<Self, Self::Err> {
        let (hrp, data) = bech32::decode(addr)?;

        if hrp != HRP_COORDINATE {
            return Err(Error::WrongPrefix);
        }

        Self::from_bech32_data(data)
    }
}

impl ToBech32 for Coordinate {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        // Allocate capacity
        let identifier_len: usize = 2 + self.identifier.len();
        let relays_len: usize = self.relays.iter().map(|u| 2 + u.as_str().len()).sum();
        let mut bytes: Vec<u8> = Vec::with_capacity(
            identifier_len + FIXED_1_1_32_BYTES_TVL + FIXED_KIND_BYTES_TVL + relays_len,
        );

        // Identifier
        bytes.push(SPECIAL); // Type
        bytes.push(self.identifier.len() as u8); // Len
        bytes.extend(self.identifier.as_bytes()); // Value

        // Author
        bytes.push(AUTHOR); // Type
        bytes.push(32); // Len
        bytes.extend(self.public_key.as_bytes()); // Value

        // Kind
        bytes.push(KIND); // Type
        bytes.push(4); // Len
        bytes.extend((self.kind.as_u16() as u32).to_be_bytes()); // Value

        for relay in self.relays.iter() {
            bytes.push(RELAY); // Type
            bytes.push(relay.as_str().len() as u8); // Len
            bytes.extend(relay.as_str().as_bytes()); // Value
        }

        Ok(bech32::encode::<Bech32>(HRP_COORDINATE, &bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn to_bech32_public_key() {
        let public_key =
            PublicKey::from_str("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        assert_eq!(
            "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy".to_string(),
            public_key.to_bech32().unwrap()
        );
    }

    #[test]
    fn to_bech32_secret_key() {
        let secret_key =
            SecretKey::from_str("9571a568a42b9e05646a349c783159b906b498119390df9a5a02667155128028")
                .unwrap();
        assert_eq!(
            "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99".to_string(),
            secret_key.to_bech32().unwrap()
        );
    }

    #[test]
    fn to_bech32_note() {
        let event_id =
            EventId::from_hex("d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5")
                .unwrap();
        assert_eq!(
            "note1m99r7nwc0wdrkzldrqan96gklg5usqspq7z9696j6unf0ljnpxjspqfw99".to_string(),
            event_id.to_bech32().unwrap()
        );
    }

    #[test]
    fn from_bech32_nip19_event() {
        let expected_event_id =
            EventId::from_hex("d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5")
                .unwrap();

        let nip19 =
            Nip19::from_bech32("note1m99r7nwc0wdrkzldrqan96gklg5usqspq7z9696j6unf0ljnpxjspqfw99")
                .unwrap();

        assert_eq!(Nip19::EventId(expected_event_id), nip19);
    }

    #[test]
    fn from_bech32_nip19_profile() {
        let nprofile = "nprofile1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gppemhxue69uhhytnc9e3k7mf0qyt8wumn8ghj7er2vfshxtnnv9jxkc3wvdhk6tclr7lsh";
        let nip19 = Nip19::from_bech32(nprofile).unwrap();

        let expected_pubkey =
            PublicKey::from_str("3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d")
                .unwrap();

        assert_eq!(
            Nip19::Profile(
                Nip19Profile::new(
                    expected_pubkey,
                    ["wss://r.x.com/", "wss://djbas.sadkb.com/"]
                )
                .unwrap()
            ),
            nip19
        );

        assert_eq!(nip19.to_bech32().unwrap(), nprofile);
    }

    #[test]
    fn test_bech32_nevent() {
        let nevent = "nevent1qqsdhet4232flykq3048jzc9msmaa3hnxuesxy3lnc33vd0wt9xwk6szyqewrqnkx4zsaweutf739s0cu7et29zrntqs5elw70vlm8zudr3y24sqsgy";
        let nip19_event = Nip19Event::from_bech32(nevent).unwrap();

        let expected_pubkey =
            PublicKey::from_str("32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245")
                .unwrap();

        assert_eq!(nip19_event.author, Some(expected_pubkey));
        assert_eq!(nip19_event.kind, None);
        assert_eq!(nip19_event.to_bech32().unwrap(), nevent);

        // Test serialization and deserialization
        let event = Nip19Event {
            event_id: EventId::from_hex(
                "d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5",
            )
            .unwrap(),
            author: None,
            kind: Some(Kind::TextNote),
            relays: Vec::new(),
        };
        let serialized = event.to_bech32().unwrap();
        assert_eq!(event, Nip19Event::from_bech32(&serialized).unwrap());
    }

    #[test]
    fn from_bech32_naddr() {
        let coordinate: &str = "naddr1qqxnzd3exgersv33xymnsve3qgs8suecw4luyht9ekff89x4uacneapk8r5dyk0gmn6uwwurf6u9rusrqsqqqa282m3gxt";
        let coordinate: Coordinate = Coordinate::from_bech32(coordinate).unwrap();

        let expected_pubkey: PublicKey =
            PublicKey::from_hex("787338757fc25d65cd929394d5e7713cf43638e8d259e8dcf5c73b834eb851f2")
                .unwrap();
        let expected_kind: Kind = Kind::LongFormTextNote;
        let exected_identifier: &str = "1692282117831";

        assert_eq!(coordinate.public_key, expected_pubkey);
        assert_eq!(coordinate.kind, expected_kind);
        assert_eq!(coordinate.identifier, exected_identifier);
    }

    #[test]
    fn test_parse_nevent_with_malformed_public_key() {
        let event = Nip19Event::from_bech32("nevent1qqsqye53g5jg5pzw87q6a3nstkf2wu7jph87nala2nvfyw5u3ewlhfspr9mhxue69uhkymmnw3ezumr9vd682unfveujumn9wspyqve5xasnyvehxqunqvryxyukydr9xsmn2d3jxgcn2wf5v5uxyerpxucrvct9x43nwwp4v3jnqwt9x5uk2dpkxq6kvwf3vycrxe35893ska2ytu").unwrap();
        assert!(event.author.is_none());
    }
}

#[cfg(bench)]
mod benches {
    use super::*;
    use crate::test::{black_box, Bencher};

    #[bench]
    pub fn to_bech32_nevent(bh: &mut Bencher) {
        let event_id =
            EventId::from_hex("d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5")
                .unwrap();
        let public_key =
            PublicKey::from_str("32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245")
                .unwrap();
        let nip19_event = Nip19Event::new(event_id, ["wss://r.x.com", "wss://djbas.sadkb.com"])
            .author(public_key);

        bh.iter(|| {
            black_box(nip19_event.to_bech32()).unwrap();
        });
    }
}
