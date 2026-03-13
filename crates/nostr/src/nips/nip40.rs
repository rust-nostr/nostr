// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-40: Expiration Timestamp
//!
//! <https://github.com/nostr-protocol/nips/blob/master/40.md>

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;

use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::nips::util::take_timestamp;
use crate::types::time::Timestamp;

const EXPIRATION: &str = "expiration";

/// NIP-40 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Parse Int error
    ParseInt(ParseIntError),
    /// Codec error
    Codec(TagCodecError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseInt(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Standardized NIP-40 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/40.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip40Tag {
    /// Expiration timestamp
    Expiration(Timestamp),
}

impl TagCodec for Nip40Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // Take iterator
        let mut iter = tag.into_iter();

        // Extract first value
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        // Match kind
        match kind.as_ref() {
            EXPIRATION => {
                let timestamp: Timestamp = take_timestamp::<_, _, Error>(&mut iter)?;
                Ok(Self::Expiration(timestamp))
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        let Self::Expiration(timestamp) = self;
        let tag: Vec<String> = vec![String::from(EXPIRATION), timestamp.to_string()];
        Tag::new(tag)
    }
}

impl_tag_codec_conversions!(Nip40Tag);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_tag() {
        let tag: Vec<String> = Vec::new();
        let err = Nip40Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::missing_tag_kind()));
    }

    #[test]
    fn test_non_existing_tag() {
        let tag = vec!["hello"];
        let err = Nip40Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Unknown));
    }

    #[test]
    fn test_standardized_expiration_tag() {
        let raw = 1600000000;
        let timestamp = Timestamp::from_secs(raw);

        // Simple
        let tag = vec!["expiration".to_string(), raw.to_string()];
        let parsed = Nip40Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip40Tag::Expiration(timestamp));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        // Invalid timestamp
        let tag = vec!["expiration", "hello"];
        let err = Nip40Tag::parse(&tag).unwrap_err();
        assert!(matches!(err, Error::ParseInt(_)));

        // Missing timestamp
        let tag = vec!["expiration"];
        let err = Nip40Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("timestamp")));
    }
}
