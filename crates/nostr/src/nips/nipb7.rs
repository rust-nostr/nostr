// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-B7: Blossom media
//!
//! <https://github.com/nostr-protocol/nips/blob/master/B7.md>

use alloc::string::{String, ToString};
use alloc::vec;

use super::util::{missing_tag_kind, take_and_parse_from_str, unknown_tag};
use crate::error::Error;
use crate::event::{Tag, TagCodec, impl_tag_codec_conversions};
use crate::types::url::Url;

const SERVER: &str = "server";

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
        let kind: S = iter.next().ok_or(missing_tag_kind())?;

        match kind.as_ref() {
            SERVER => {
                let server_url: Url = take_and_parse_from_str(&mut iter, "server URL")?;
                Ok(Self::Server(server_url))
            }
            _ => Err(unknown_tag()),
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
