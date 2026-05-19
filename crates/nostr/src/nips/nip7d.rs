// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-7D: Threads
//!
//! <https://github.com/nostr-protocol/nips/blob/master/7D.md>

use alloc::string::String;
use alloc::vec;

use super::util::{missing_tag_kind, take_string, unknown_tag};
use crate::error::Error;
use crate::event::{Tag, TagCodec, impl_tag_codec_conversions};

const TITLE: &str = "title";

/// Standardized NIP-7D tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/7D.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip7DTag {
    /// `title` tag
    Title(String),
}

impl TagCodec for Nip7DTag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();

        let kind: S = iter.next().ok_or(missing_tag_kind())?;

        match kind.as_ref() {
            TITLE => Ok(Self::Title(take_string(&mut iter, "title")?)),
            _ => Err(unknown_tag()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Title(title) => Tag::new(vec![String::from(TITLE), title.clone()]),
        }
    }
}

impl_tag_codec_conversions!(Nip7DTag);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_title_tag() {
        let tag = vec!["title", "Lorem Ipsum"];
        let parsed = Nip7DTag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip7DTag::Title(String::from("Lorem Ipsum")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
