// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::fmt;
use core::num::ParseIntError;

use hashes::hex::HexToArrayError;

#[cfg(feature = "nip98")]
use crate::nips::nip98;
use crate::nips::{nip01, nip10, nip39, nip53, nip65, nip88};
use crate::types::image;
use crate::types::url::{Error as RelayUrlError, ParseError};
use crate::{key, secp256k1};

/// Tag error
#[allow(deprecated)]
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Keys
    Keys(key::Error),
    /// Impossible to parse integer
    ParseIntError(ParseIntError),
    /// Secp256k1
    Secp256k1(secp256k1::Error),
    /// Hex decoding error
    Hex(HexToArrayError),
    /// Relay Url parse error
    RelayUrl(RelayUrlError),
    /// Url parse error
    Url(ParseError),
    /// NIP01 error
    NIP01(nip01::Error),
    /// NIP10 error
    NIP10(nip10::Error),
    /// NIP39 error
    NIP39(nip39::Error),
    /// NIP53 error
    NIP53(nip53::Error),
    /// NIP65 error
    NIP65(nip65::Error),
    /// NIP88 error
    NIP88(nip88::Error),
    /// NIP98 error
    #[cfg(feature = "nip98")]
    NIP98(nip98::Error),
    /// Event Error
    Event(crate::event::Error),
    /// Image
    Image(image::Error),
    /// Unknown standardized tag
    UnknownStandardizedTag,
    /// Impossible to find tag kind
    KindNotFound,
    /// Empty tag
    EmptyTag,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keys(e) => e.fmt(f),
            Self::ParseIntError(e) => e.fmt(f),
            Self::Secp256k1(e) => e.fmt(f),
            Self::Hex(e) => e.fmt(f),
            Self::RelayUrl(e) => e.fmt(f),
            Self::Url(e) => e.fmt(f),
            Self::NIP01(e) => e.fmt(f),
            Self::NIP10(e) => e.fmt(f),
            Self::NIP39(e) => e.fmt(f),
            Self::NIP53(e) => e.fmt(f),
            Self::NIP65(e) => e.fmt(f),
            Self::NIP88(e) => e.fmt(f),
            #[cfg(feature = "nip98")]
            Self::NIP98(e) => e.fmt(f),
            Self::Event(e) => e.fmt(f),
            Self::Image(e) => e.fmt(f),
            Self::UnknownStandardizedTag => f.write_str("Unknown standardized tag"),
            Self::KindNotFound => f.write_str("Impossible to find tag kind"),
            Self::EmptyTag => f.write_str("Empty tag"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<HexToArrayError> for Error {
    fn from(e: HexToArrayError) -> Self {
        Self::Hex(e)
    }
}

impl From<RelayUrlError> for Error {
    fn from(e: RelayUrlError) -> Self {
        Self::RelayUrl(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<nip01::Error> for Error {
    fn from(e: nip01::Error) -> Self {
        Self::NIP01(e)
    }
}

impl From<nip10::Error> for Error {
    fn from(e: nip10::Error) -> Self {
        Self::NIP10(e)
    }
}

impl From<nip39::Error> for Error {
    fn from(e: nip39::Error) -> Self {
        Self::NIP39(e)
    }
}

impl From<nip53::Error> for Error {
    fn from(e: nip53::Error) -> Self {
        Self::NIP53(e)
    }
}

impl From<nip65::Error> for Error {
    fn from(e: nip65::Error) -> Self {
        Self::NIP65(e)
    }
}

impl From<nip88::Error> for Error {
    fn from(e: nip88::Error) -> Self {
        Self::NIP88(e)
    }
}

#[cfg(feature = "nip98")]
impl From<nip98::Error> for Error {
    fn from(e: nip98::Error) -> Self {
        Self::NIP98(e)
    }
}

impl From<crate::event::Error> for Error {
    fn from(e: crate::event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<image::Error> for Error {
    fn from(e: image::Error) -> Self {
        Self::Image(e)
    }
}
