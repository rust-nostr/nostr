// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::fmt;
use core::num::ParseIntError;

use crate::event::id;
use crate::hashes::hex::HexToArrayError;
use crate::nips::{nip01, nip10, nip26, nip39, nip53, nip65, nip98};
use crate::types::image;
use crate::types::url::{Error as RelayUrlError, ParseError};
use crate::{key, secp256k1};

/// Tag error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Keys
    Keys(key::Error),
    /// Impossible to find tag kind
    KindNotFound,
    /// Empty tag
    EmptyTag,
    /// Invalid Zap Request
    InvalidZapRequest,
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
    /// EventId error
    EventId(id::Error),
    /// NIP01 error
    NIP01(nip01::Error),
    /// NIP10 error
    NIP10(nip10::Error),
    /// NIP26 error
    NIP26(nip26::Error),
    /// NIP39 error
    NIP39(nip39::Error),
    /// NIP53 error
    NIP53(nip53::Error),
    /// NIP65 error
    NIP65(nip65::Error),
    /// NIP98 error
    NIP98(nip98::Error),
    /// Event Error
    Event(crate::event::Error),
    /// Image
    Image(image::Error),
    /// Unknown standardized tag
    UnknownStardardizedTag,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keys(e) => write!(f, "{e}"),
            Self::ParseIntError(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::Hex(e) => write!(f, "{e}"),
            Self::RelayUrl(e) => write!(f, "{e}"),
            Self::Url(e) => write!(f, "{e}"),
            Self::EventId(e) => write!(f, "{e}"),
            Self::NIP01(e) => write!(f, "{e}"),
            Self::NIP10(e) => write!(f, "{e}"),
            Self::NIP26(e) => write!(f, "{e}"),
            Self::NIP39(e) => write!(f, "{e}"),
            Self::NIP53(e) => write!(f, "{e}"),
            Self::NIP65(e) => write!(f, "{e}"),
            Self::NIP98(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::Image(e) => write!(f, "{e}"),
            Self::UnknownStardardizedTag => write!(f, "Unknown standardized tag"),
            Self::KindNotFound => write!(f, "Impossible to find tag kind"),
            Self::EmptyTag => write!(f, "Empty tag"),
            Self::InvalidZapRequest => write!(f, "Invalid Zap request"),
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

impl From<id::Error> for Error {
    fn from(e: id::Error) -> Self {
        Self::EventId(e)
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

impl From<nip26::Error> for Error {
    fn from(e: nip26::Error) -> Self {
        Self::NIP26(e)
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
