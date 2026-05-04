// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-31: Dealing with unknown event kinds
//!
//! <https://github.com/nostr-protocol/nips/blob/master/31.md>

use alloc::string::String;
use alloc::vec;
use core::fmt;

use super::util::take_string;
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};

const ALT: &str = "alt";

/// NIP-31 error
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

/// Standardized NIP-31 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/31.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip31Tag {
    /// `alt` tag
    Alt(String),
}

impl TagCodec for Nip31Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            ALT => {
                let alt: String = take_string(&mut iter, "alt value")?;
                Ok(Self::Alt(alt))
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Alt(value) => Tag::new(vec![String::from(ALT), value.clone()]),
        }
    }
}

impl_tag_codec_conversions!(Nip31Tag);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nip31_alt_tag() {
        let tag = vec!["alt", "Something"];
        let parsed = Nip31Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip31Tag::Alt(String::from("Something")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_invalid_alt_tag_missing_value() {
        let tag = vec!["alt"];
        assert_eq!(
            Nip31Tag::parse(&tag).unwrap_err(),
            Error::Codec(TagCodecError::Missing("alt value"))
        );
    }
}
