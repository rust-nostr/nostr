// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP65
//!
//! <https://github.com/nostr-protocol/nips/blob/master/65.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::{Event, TagStandard, Url};

/// NIP-56 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid Relay Metadata
    InvalidRelayMetadata(String),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRelayMetadata(s) => write!(f, "Invalid relay metadata: {s}"),
        }
    }
}

/// Relay Metadata
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayMetadata {
    /// Read
    Read,
    /// Write
    Write,
}

impl fmt::Display for RelayMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
        }
    }
}

impl FromStr for RelayMetadata {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            s => Err(Error::InvalidRelayMetadata(s.to_string())),
        }
    }
}

/// Extracts the relay info (url, optional read/write flag) from the event
#[inline]
pub fn extract_relay_list(event: &Event) -> Vec<(&Url, &Option<RelayMetadata>)> {
    event
        .iter_tags()
        .filter_map(|tag| {
            if let Some(TagStandard::RelayMetadata {
                relay_url,
                metadata,
            }) = tag.as_standardized()
            {
                Some((relay_url, metadata))
            } else {
                None
            }
        })
        .collect()
}
