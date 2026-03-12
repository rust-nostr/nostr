// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::fmt;

use super::Tag;

/// Tag codec error
#[derive(Debug, PartialEq, Eq)]
pub enum TagCodecError {
    /// Missing value
    Missing(&'static str),
    /// Invalid value
    Invalid(&'static str),
    /// Unknown tag
    Unknown,
}

impl core::error::Error for TagCodecError {}

impl fmt::Display for TagCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(value) => write!(f, "missing {value}"),
            Self::Invalid(value) => write!(f, "{value}"),
            Self::Unknown => write!(f, "unknown tag"),
        }
    }
}

impl TagCodecError {
    /// Missing tag kind
    #[inline]
    pub fn missing_tag_kind() -> Self {
        Self::Missing("tag kind")
    }
}

/// Conversion contract between a typed tag representation and the generic raw [`Tag`].
///
/// Types implementing this trait are expected to:
/// - parse themselves from a raw tag representation
/// - convert themselves back into a raw [`Tag`]
///
/// This trait is intended for NIP-specific tag enums and, where useful, for individual
/// tag structs that can be used independently of the enclosing enum.
pub trait TagCodec: Sized {
    /// Error returned when parsing a raw tag into the typed representation fails.
    type Error: core::error::Error;

    /// Parse a typed tag from a raw tag representation.
    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>;

    /// Convert this typed tag into the generic raw [`Tag`] representation.
    fn to_tag(&self) -> Tag;
}

/// Implement the standard conversions for a [`TagCodec`] type.
#[macro_export]
macro_rules! impl_tag_codec_conversions {
    ($ty:ty) => {
        impl From<&$ty> for Tag {
            #[inline]
            fn from(value: &$ty) -> Self {
                value.to_tag()
            }
        }

        impl From<$ty> for Tag {
            #[inline]
            fn from(value: $ty) -> Self {
                value.to_tag()
            }
        }

        impl TryFrom<&Tag> for $ty {
            type Error = <$ty as TagCodec>::Error;

            #[inline]
            fn try_from(tag: &Tag) -> Result<Self, Self::Error> {
                <$ty as TagCodec>::parse(tag.as_slice())
            }
        }

        impl TryFrom<Tag> for $ty {
            type Error = <$ty as TagCodec>::Error;

            #[inline]
            fn try_from(tag: Tag) -> Result<Self, Self::Error> {
                <$ty as TagCodec>::parse(tag.as_slice())
            }
        }
    };
}

pub use impl_tag_codec_conversions;
