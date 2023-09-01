// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP21
//!
//! <https://github.com/nostr-protocol/nips/blob/master/21.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use bitcoin::secp256k1::XOnlyPublicKey;

use super::nip19::{Error as NIP19Error, FromBech32, Nip19Event, ToBech32};
use super::nip33::ParameterizedReplaceableEvent;
use crate::event::id::EventId;
use crate::types::profile::Profile;

/// URI scheme
pub const SCHEME: &str = "nostr";

/// NIP21 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// NIP19 error
    NIP19(NIP19Error),
    /// Invalid nostr URI
    InvalidURI,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP19(e) => write!(f, "NIP19: {e}"),
            Self::InvalidURI => write!(f, "Invalid nostr URI"),
        }
    }
}

impl From<NIP19Error> for Error {
    fn from(e: NIP19Error) -> Self {
        Self::NIP19(e)
    }
}

/// Nostr URI trait
pub trait NostrURI: Sized + ToBech32 + FromBech32
where
    Error: From<<Self as ToBech32>::Err>,
    Error: From<<Self as FromBech32>::Err>,
{
    /// Get nostr URI
    fn to_nostr_uri(&self) -> Result<String, Error> {
        Ok(format!("{SCHEME}:{}", self.to_bech32()?))
    }

    /// From `nostr` URI
    fn from_nostr_uri<S>(uri: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let uri: String = uri.into();
        let splitted: Vec<&str> = uri.split(':').collect();
        let data = splitted.get(1).ok_or(Error::InvalidURI)?;
        Ok(Self::from_bech32(*data)?)
    }
}

impl NostrURI for XOnlyPublicKey {}
impl NostrURI for EventId {}
impl NostrURI for Profile {}
impl NostrURI for Nip19Event {}
impl NostrURI for ParameterizedReplaceableEvent {}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_to_nostr_uri() {
        let pubkey = XOnlyPublicKey::from_str(
            "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4",
        )
        .unwrap();
        assert_eq!(
            pubkey.to_nostr_uri().unwrap(),
            String::from("nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
        );
    }

    #[test]
    fn test_from_nostr_uri() {
        let pubkey = XOnlyPublicKey::from_str(
            "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4",
        )
        .unwrap();
        assert_eq!(
            XOnlyPublicKey::from_nostr_uri(
                "nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy"
            )
            .unwrap(),
            pubkey
        );
    }
}
