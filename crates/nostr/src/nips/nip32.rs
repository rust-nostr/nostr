// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-32: Labeling
//!
//! <https://github.com/nostr-protocol/nips/blob/master/32.md>

use alloc::string::String;
use alloc::vec;
use core::fmt;

use super::util::take_string;
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};

/// NIP-32 error
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

/// Standardized NIP-32 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/32.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip32Tag {
    /// `L` tag
    LabelNamespace(String),
    /// `l` tag
    Label {
        /// Label value
        value: String,
        /// Label namespace
        namespace: String,
    },
}

impl TagCodec for Nip32Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();

        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            "L" => Ok(Self::LabelNamespace(take_string(
                &mut iter,
                "label namespace",
            )?)),
            "l" => {
                let value: String = take_string(&mut iter, "label")?;
                let namespace: String = take_string(&mut iter, "label namespace")?;

                Ok(Self::Label { value, namespace })
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::LabelNamespace(namespace) => Tag::new(vec![String::from("L"), namespace.clone()]),
            Self::Label { value, namespace } => {
                Tag::new(vec![String::from("l"), value.clone(), namespace.clone()])
            }
        }
    }
}

impl_tag_codec_conversions!(Nip32Tag);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_label_namespace_tag() {
        let tag = vec!["L", "test"];
        let parsed = Nip32Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip32Tag::LabelNamespace(String::from("test")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_label_tag() {
        let tag = vec!["l", "other", "test"];
        let parsed = Nip32Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip32Tag::Label {
                value: String::from("other"),
                namespace: String::from("test")
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
