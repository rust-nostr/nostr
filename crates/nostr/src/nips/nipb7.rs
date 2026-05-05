// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-B7: Blossom media
//!
//! <https://github.com/nostr-protocol/nips/blob/master/B7.md>

use alloc::string::{String, ToString};
use alloc::vec;
use core::fmt;

use super::util::take_and_parse_from_str;
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::types::url::{self, Url};

const SERVER: &str = "server";

/// NIP-B7 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Url error
    Url(url::ParseError),
    /// Codec error
    Codec(TagCodecError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Standardized NIP-B7 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/B7.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NipB7Tag {
    /// `server` tag
    Server(Url),
}

impl TagCodec for NipB7Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            SERVER => {
                let server_url: Url =
                    take_and_parse_from_str::<_, _, _, Error>(&mut iter, "server URL")?;
                Ok(Self::Server(server_url))
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Server(server_url) => {
                Tag::new(vec![String::from(SERVER), server_url.to_string()])
            }
        }
    }
}

impl_tag_codec_conversions!(NipB7Tag);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_tag() {
        let tag = vec!["server", "https://blossom.example.com/"];
        let parsed = NipB7Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            NipB7Tag::Server(Url::parse("https://blossom.example.com").unwrap())
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
