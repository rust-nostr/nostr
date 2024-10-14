// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP21: `nostr:` URI scheme
//!
//! <https://github.com/nostr-protocol/nips/blob/master/21.md>

use alloc::string::String;
use core::fmt;

use super::nip01::Coordinate;
use super::nip19::{self, FromBech32, Nip19, Nip19Event, Nip19Profile, ToBech32};
use crate::{EventId, PublicKey};

/// URI scheme
pub const SCHEME: &str = "nostr";

/// Unsupported Bech32 Type
#[derive(Debug, PartialEq, Eq)]
pub enum UnsupportedBech32Type {
    /// Secret Key
    SecretKey,
}

impl fmt::Display for UnsupportedBech32Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SecretKey => write!(f, "secret key"),
        }
    }
}

/// NIP21 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// NIP19 error
    NIP19(nip19::Error),
    /// Invalid nostr URI
    InvalidURI,
    /// Unsupported bech32 type
    UnsupportedBech32Type(UnsupportedBech32Type),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP19(e) => write!(f, "NIP19: {e}"),
            Self::InvalidURI => write!(f, "Invalid nostr URI"),
            Self::UnsupportedBech32Type(t) => write!(f, "Unsupported bech32 type: {t}"),
        }
    }
}

impl From<nip19::Error> for Error {
    fn from(e: nip19::Error) -> Self {
        Self::NIP19(e)
    }
}

fn split_uri(uri: &str) -> Result<&str, Error> {
    let mut splitted = uri.split(':');
    let prefix: &str = splitted.next().ok_or(Error::InvalidURI)?;

    if prefix != SCHEME {
        return Err(Error::InvalidURI);
    }

    splitted.next().ok_or(Error::InvalidURI)
}

/// Nostr URI trait
pub trait NostrURI: Sized + ToBech32 + FromBech32
where
    Error: From<<Self as ToBech32>::Err>,
    Error: From<<Self as FromBech32>::Err>,
{
    /// Get nostr URI
    #[inline]
    fn to_nostr_uri(&self) -> Result<String, Error> {
        Ok(format!("{SCHEME}:{}", self.to_bech32()?))
    }

    /// From `nostr` URI
    #[inline]
    fn from_nostr_uri<S>(uri: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let data: &str = split_uri(uri.as_ref())?;
        Ok(Self::from_bech32(data)?)
    }
}

impl NostrURI for PublicKey {}
impl NostrURI for EventId {}
impl NostrURI for Nip19Profile {}
impl NostrURI for Nip19Event {}
impl NostrURI for Coordinate {}

/// A representation any `NIP21` object. Useful for decoding
/// `NIP21` strings without necessarily knowing what you're decoding
/// ahead of time.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Nip21 {
    /// nostr::npub
    Pubkey(PublicKey),
    /// nostr::nprofile
    Profile(Nip19Profile),
    /// nostr::note
    EventId(EventId),
    /// nostr::nevent
    Event(Nip19Event),
    /// nostr::naddr
    Coordinate(Coordinate),
}

impl From<Nip21> for Nip19 {
    fn from(value: Nip21) -> Self {
        match value {
            Nip21::Pubkey(val) => Self::Pubkey(val),
            Nip21::Profile(val) => Self::Profile(val),
            Nip21::EventId(val) => Self::EventId(val),
            Nip21::Event(val) => Self::Event(val),
            Nip21::Coordinate(val) => Self::Coordinate(val),
        }
    }
}

impl TryFrom<Nip19> for Nip21 {
    type Error = Error;

    fn try_from(value: Nip19) -> Result<Self, Self::Error> {
        match value {
            Nip19::Secret(..) => Err(Error::UnsupportedBech32Type(
                UnsupportedBech32Type::SecretKey,
            )),
            #[cfg(feature = "nip49")]
            Nip19::EncryptedSecret(..) => Err(Error::UnsupportedBech32Type(
                UnsupportedBech32Type::SecretKey,
            )),
            Nip19::Pubkey(val) => Ok(Self::Pubkey(val)),
            Nip19::Profile(val) => Ok(Self::Profile(val)),
            Nip19::EventId(val) => Ok(Self::EventId(val)),
            Nip19::Event(val) => Ok(Self::Event(val)),
            Nip19::Coordinate(val) => Ok(Self::Coordinate(val)),
        }
    }
}

impl Nip21 {
    /// Parse NIP21 string
    #[inline]
    pub fn parse<S>(uri: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let data: &str = split_uri(uri.as_ref())?;
        let nip19: Nip19 = Nip19::from_bech32(data)?;
        Self::try_from(nip19)
    }

    /// Serialize to NIP21 nostr URI
    pub fn to_nostr_uri(&self) -> Result<String, Error> {
        match self {
            Self::Pubkey(val) => Ok(val.to_nostr_uri()?),
            Self::Profile(val) => Ok(val.to_nostr_uri()?),
            Self::EventId(val) => Ok(val.to_nostr_uri()?),
            Self::Event(val) => Ok(val.to_nostr_uri()?),
            Self::Coordinate(val) => Ok(val.to_nostr_uri()?),
        }
    }

    /// Get [EventId] if exists
    pub fn event_id(&self) -> Option<EventId> {
        match self {
            Self::EventId(id) => Some(*id),
            Self::Event(e) => Some(e.event_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_to_nostr_uri() {
        let pubkey =
            PublicKey::from_str("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        assert_eq!(
            pubkey.to_nostr_uri().unwrap(),
            String::from("nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
        );

        let generic = Nip21::Pubkey(pubkey);
        assert_eq!(
            generic.to_nostr_uri().unwrap(),
            String::from("nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
        );
    }

    #[test]
    fn test_from_nostr_uri() {
        let pubkey =
            PublicKey::from_str("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        assert_eq!(
            PublicKey::from_nostr_uri(
                "nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy"
            )
            .unwrap(),
            pubkey
        );

        assert_eq!(
            Nip21::parse("nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
                .unwrap(),
            Nip21::Pubkey(pubkey),
        );

        assert_eq!(
            Nip21::parse("nostr:nprofile1qqsr9cvzwc652r4m83d86ykplrnm9dg5gwdvzzn8ameanlvut35wy3gpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsz4nxck").unwrap(),
            Nip21::Profile(Nip19Profile::new(
                PublicKey::from_str(
                    "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
                )
                .unwrap(),
                ["wss://relay.damus.io/"]
            ).unwrap()),
        );
    }

    #[test]
    fn test_unsupported_from_nostr_uri() {
        assert_eq!(
            Nip21::parse("nostr:nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
                .unwrap_err(),
            Error::UnsupportedBech32Type(UnsupportedBech32Type::SecretKey)
        );
    }
}
