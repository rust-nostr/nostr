// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-70: Protected Events
//!
//! <https://github.com/nostr-protocol/nips/blob/master/70.md>

use alloc::string::String;
use alloc::vec;
use core::fmt;

use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};

/// NIP-70 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Codec error
    Codec(TagCodecError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Codec(e) => e.fmt(f),
        }
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Standardized NIP-70 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/70.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip70Tag {
    /// Protected event tag
    ///
    /// `["-"]`
    Protected,
}

impl TagCodec for Nip70Tag {
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
            "-" => Ok(Self::Protected),
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Protected => Tag::new(vec![String::from("-")]),
        }
    }
}

impl_tag_codec_conversions!(Nip70Tag);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standardized_protected_tag() {
        let tag = vec!["-"];
        let parsed = Nip70Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip70Tag::Protected);
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
