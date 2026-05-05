// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-58: Badges
//!
//! <https://github.com/nostr-protocol/nips/blob/master/58.md>

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use super::nip01::Nip01Tag;
use super::util::{take_and_parse_from_str, take_and_parse_optional_from_str, take_string};
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::types::url::{self, Url};
use crate::types::{RelayUrl, image};
use crate::{Event, ImageDimensions, Kind, PublicKey};

const IDENTIFIER: &str = "d";
const NAME: &str = "name";
const DESCRIPTION: &str = "description";
const IMAGE: &str = "image";
const THUMB: &str = "thumb";

#[derive(Debug, PartialEq, Eq)]
/// Badge Award error
pub enum Error {
    /// Image error
    Image(image::Error),
    /// Url error
    Url(url::ParseError),
    /// Codec error
    Codec(TagCodecError),
    /// Invalid length
    InvalidLength,
    /// Invalid kind
    InvalidKind,
    /// Identifier tag not found
    IdentifierTagNotFound,
    /// Mismatched badge definition or award
    MismatchedBadgeDefinitionOrAward,
    /// Badge awards lack the awarded public key
    BadgeAwardsLackAwardedPublicKey,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image(e) => e.fmt(f),
            Self::Url(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
            Self::InvalidLength => f.write_str("invalid length"),
            Self::InvalidKind => f.write_str("invalid kind"),
            Self::IdentifierTagNotFound => f.write_str("identifier tag not found"),
            Self::MismatchedBadgeDefinitionOrAward => {
                f.write_str("mismatched badge definition/award")
            }
            Self::BadgeAwardsLackAwardedPublicKey => {
                f.write_str("badge award events lack the awarded public key")
            }
        }
    }
}

impl From<image::Error> for Error {
    fn from(e: image::Error) -> Self {
        Self::Image(e)
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

/// Standardized NIP-58 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/58.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip58Tag {
    /// `d` tag
    Identifier(String),
    /// `name` tag
    Name(String),
    /// `description` tag
    Description(String),
    /// `image` tag
    Image(Url, Option<ImageDimensions>),
    /// `thumb` tag
    Thumb(Url, Option<ImageDimensions>),
}

impl TagCodec for Nip58Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            IDENTIFIER => Ok(Self::Identifier(take_string(&mut iter, "identifier")?)),
            NAME => Ok(Self::Name(take_string(&mut iter, "name")?)),
            DESCRIPTION => Ok(Self::Description(take_string(&mut iter, "description")?)),
            IMAGE => {
                let (url, dimensions) = parse_url_and_dimensions_tag(iter, "image URL")?;
                Ok(Self::Image(url, dimensions))
            }
            THUMB => {
                let (url, dimensions) = parse_url_and_dimensions_tag(iter, "thumbnail URL")?;
                Ok(Self::Thumb(url, dimensions))
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Identifier(identifier) => {
                Tag::new(vec![String::from(IDENTIFIER), identifier.clone()])
            }
            Self::Name(name) => Tag::new(vec![String::from(NAME), name.clone()]),
            Self::Description(description) => {
                Tag::new(vec![String::from(DESCRIPTION), description.clone()])
            }
            Self::Image(url, dimensions) => to_url_and_dimensions_tag(IMAGE, url, dimensions),
            Self::Thumb(url, dimensions) => to_url_and_dimensions_tag(THUMB, url, dimensions),
        }
    }
}

impl_tag_codec_conversions!(Nip58Tag);

fn parse_url_and_dimensions_tag<T, S>(
    mut iter: T,
    missing_error: &'static str,
) -> Result<(Url, Option<ImageDimensions>), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let url: Url = take_and_parse_from_str::<_, _, _, Error>(&mut iter, missing_error)?;
    let dimensions: Option<ImageDimensions> = take_and_parse_optional_from_str(&mut iter)?;

    Ok((url, dimensions))
}

fn to_url_and_dimensions_tag(kind: &str, url: &Url, dimensions: &Option<ImageDimensions>) -> Tag {
    let mut tag: Vec<String> = Vec::with_capacity(2 + dimensions.is_some() as usize);
    tag.push(String::from(kind));
    tag.push(url.to_string());

    if let Some(dimensions) = dimensions {
        tag.push(dimensions.to_string());
    }

    Tag::new(tag)
}

/// Helper function to filter events for a specific [`Kind`]
#[inline]
pub(crate) fn filter_for_kind(events: Vec<Event>, kind_needed: &Kind) -> Vec<Event> {
    events
        .into_iter()
        .filter(|e| &e.kind == kind_needed)
        .collect()
}

/// Helper function to extract the awarded public key from an array of PubKey tags
pub(crate) fn extract_awarded_public_key(
    tags: &[Tag],
    awarded_public_key: PublicKey,
) -> Option<(PublicKey, Option<RelayUrl>)> {
    tags.iter().find_map(|t| match Nip01Tag::try_from(t) {
        Ok(Nip01Tag::PublicKey {
            public_key,
            relay_hint,
        }) if public_key == awarded_public_key => Some((public_key, relay_hint)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_tag() {
        let tag = vec!["d", "bravery"];
        let parsed = Nip58Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip58Tag::Identifier(String::from("bravery")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_name_tag() {
        let tag = vec!["name", "Medal of Bravery"];
        let parsed = Nip58Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip58Tag::Name(String::from("Medal of Bravery")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_description_tag() {
        let tag = vec!["description", "Awarded to users demonstrating bravery"];
        let parsed = Nip58Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip58Tag::Description(String::from("Awarded to users demonstrating bravery"))
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_image_tag() {
        let tag = vec![
            "image",
            "https://nostr.academy/awards/bravery.png",
            "1024x1024",
        ];
        let parsed = Nip58Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip58Tag::Image(
                Url::parse("https://nostr.academy/awards/bravery.png").unwrap(),
                Some(ImageDimensions::new(1024, 1024))
            )
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_thumb_tag() {
        let tag = vec![
            "thumb",
            "https://nostr.academy/awards/bravery_256x256.png",
            "256x256",
        ];
        let parsed = Nip58Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip58Tag::Thumb(
                Url::parse("https://nostr.academy/awards/bravery_256x256.png").unwrap(),
                Some(ImageDimensions::new(256, 256))
            )
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
