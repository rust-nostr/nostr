// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP10: Conventions for clients' use of `e` and `p` tags in text events
//!
//! <https://github.com/nostr-protocol/nips/blob/master/10.md>

use core::fmt;
use core::str::FromStr;

/// NIP10 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid marker
    InvalidMarker,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMarker => write!(f, "invalid marker"),
        }
    }
}

/// Marker
///
/// <https://github.com/nostr-protocol/nips/blob/master/10.md>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Marker {
    /// Root
    Root,
    /// Reply
    Reply,
    /// Mention
    Mention,
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Root => write!(f, "root"),
            Self::Reply => write!(f, "reply"),
            Self::Mention => write!(f, "mention"),
        }
    }
}

impl FromStr for Marker {
    type Err = Error;

    fn from_str(marker: &str) -> Result<Self, Self::Err> {
        match marker {
            "root" => Ok(Self::Root),
            "reply" => Ok(Self::Reply),
            "mention" => Ok(Self::Mention),
            _ => Err(Error::InvalidMarker),
        }
    }
}
