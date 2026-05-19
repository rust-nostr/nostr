// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-70: Protected Events
//!
//! <https://github.com/nostr-protocol/nips/blob/master/70.md>

use alloc::string::String;
use alloc::vec;

use crate::error::Error;
use crate::event::{Tag, TagCodec, impl_tag_codec_conversions};
use crate::nips::util::{missing_tag_kind, unknown_tag};

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
        let kind: S = iter.next().ok_or(missing_tag_kind())?;

        // Match kind
        match kind.as_ref() {
            "-" => Ok(Self::Protected),
            _ => Err(unknown_tag()),
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
