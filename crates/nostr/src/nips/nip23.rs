// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-23: Long-form Content
//!
//! <https://github.com/nostr-protocol/nips/blob/master/23.md>

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::error::Error;
use crate::event::{Tag, TagCodec, impl_tag_codec_conversions};
use crate::nips::util::{
    missing_tag_kind, take_and_parse_from_str, take_and_parse_optional_from_str, take_string,
    take_timestamp, unknown_tag,
};
use crate::types::url::Url;
use crate::{ImageDimensions, Timestamp};

const TITLE: &str = "title";
const IMAGE: &str = "image";
const SUMMARY: &str = "summary";
const PUBLISHED_AT: &str = "published_at";
const HASHTAG: &str = "t";

/// Standardized NIP-23 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/23.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip23Tag {
    /// `title` tag
    Title(String),
    /// `image` tag
    Image(Url, Option<ImageDimensions>),
    /// `summary` tag
    Summary(String),
    /// `published_at` tag
    PublishedAt(Timestamp),
    /// `t` tag
    Hashtag(String),
}

impl TagCodec for Nip23Tag {
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
            IMAGE => {
                let (url, dimensions) = parse_image_tag(iter)?;
                Ok(Self::Image(url, dimensions))
            }
            SUMMARY => Ok(Self::Summary(take_string(&mut iter, "summary")?)),
            PUBLISHED_AT => {
                let timestamp: Timestamp = take_timestamp(&mut iter)?;
                Ok(Self::PublishedAt(timestamp))
            }
            HASHTAG => Ok(Self::Hashtag(
                take_string(&mut iter, "hashtag")?.to_lowercase(),
            )),
            _ => Err(unknown_tag()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Title(title) => Tag::new(vec![String::from(TITLE), title.clone()]),
            Self::Image(url, dimensions) => {
                let mut tag: Vec<String> = Vec::with_capacity(2 + dimensions.is_some() as usize);
                tag.push(String::from(IMAGE));
                tag.push(url.to_string());
                if let Some(dimensions) = dimensions {
                    tag.push(dimensions.to_string());
                }
                Tag::new(tag)
            }
            Self::Summary(summary) => Tag::new(vec![String::from(SUMMARY), summary.clone()]),
            Self::PublishedAt(timestamp) => {
                Tag::new(vec![String::from(PUBLISHED_AT), timestamp.to_string()])
            }
            Self::Hashtag(hashtag) => Tag::new(vec![String::from(HASHTAG), hashtag.to_lowercase()]),
        }
    }
}

impl_tag_codec_conversions!(Nip23Tag);

fn parse_image_tag<T, S>(mut iter: T) -> Result<(Url, Option<ImageDimensions>), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let url: Url = take_and_parse_from_str(&mut iter, "image URL")?;
    let dimensions: Option<ImageDimensions> = take_and_parse_optional_from_str(&mut iter)?;

    Ok((url, dimensions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_title_tag() {
        let tag = vec!["title", "Lorem Ipsum"];
        let parsed = Nip23Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip23Tag::Title(String::from("Lorem Ipsum")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_image_tag() {
        let tag = vec!["image", "https://example.com/image.jpg", "1024x768"];
        let parsed = Nip23Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip23Tag::Image(
                Url::parse("https://example.com/image.jpg").unwrap(),
                Some(ImageDimensions::new(1024, 768))
            )
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_summary_tag() {
        let tag = vec!["summary", "Article summary"];
        let parsed = Nip23Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip23Tag::Summary(String::from("Article summary")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_published_at_tag() {
        let tag = vec!["published_at", "1296962229"];
        let parsed = Nip23Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip23Tag::PublishedAt(Timestamp::from(1296962229)));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_hashtag_tag() {
        let tag = vec!["t", "Placeholder"];
        let parsed = Nip23Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip23Tag::Hashtag(String::from("placeholder")));
        assert_eq!(
            parsed.to_tag(),
            Tag::parse(vec!["t", "placeholder"]).unwrap()
        );
    }
}
