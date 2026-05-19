// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-31: Dealing with unknown event kinds
//!
//! <https://github.com/nostr-protocol/nips/blob/master/31.md>

use alloc::string::String;
use alloc::vec;

use super::util::{missing_tag_kind, take_string, unknown_tag};
use crate::error::Error;
use crate::event::{Tag, TagCodec, impl_tag_codec_conversions};

const ALT: &str = "alt";

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
        let kind: S = iter.next().ok_or(missing_tag_kind())?;

        match kind.as_ref() {
            ALT => {
                let alt: String = take_string(&mut iter, "alt value")?;
                Ok(Self::Alt(alt))
            }
            _ => Err(unknown_tag()),
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
    use crate::error::ErrorKind;

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
            Nip31Tag::parse(&tag).unwrap_err().kind(),
            ErrorKind::Missing
        );
    }
}
