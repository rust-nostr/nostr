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

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMarker => f.write_str("invalid marker"),
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
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Root => f.write_str("root"),
            Self::Reply => f.write_str("reply"),
        }
    }
}

impl FromStr for Marker {
    type Err = Error;

    fn from_str(marker: &str) -> Result<Self, Self::Err> {
        match marker {
            "root" => Ok(Self::Root),
            "reply" => Ok(Self::Reply),
            _ => Err(Error::InvalidMarker),
        }
    }
}
