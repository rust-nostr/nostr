// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-B0: Web Bookmarks
//!
//! <https://github.com/nostr-protocol/nips/blob/master/B0.md>

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;

use super::util::{take_string, take_timestamp};
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::{EventBuilder, Kind, Timestamp};

const URL: &str = "d";
const PUBLISHED_AT: &str = "published_at";
const TITLE: &str = "title";
const HASHTAG: &str = "t";

/// NIP-B0 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Parse Int error
    ParseInt(ParseIntError),
    /// Codec error
    Codec(TagCodecError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseInt(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Standardized NIP-B0 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/B0.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NipB0Tag {
    /// `d` tag containing the bookmarked URL without scheme
    Url(String),
    /// `published_at` tag
    PublishedAt(Timestamp),
    /// `title` tag
    Title(String),
    /// `t` tag
    Hashtag(String),
}

impl TagCodec for NipB0Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();

        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            URL => Ok(Self::Url(take_string(&mut iter, "URL")?)),
            PUBLISHED_AT => {
                let timestamp: Timestamp = take_timestamp::<_, _, Error>(&mut iter)?;
                Ok(Self::PublishedAt(timestamp))
            }
            TITLE => Ok(Self::Title(take_string(&mut iter, "title")?)),
            HASHTAG => Ok(Self::Hashtag(
                take_string(&mut iter, "hashtag")?.to_lowercase(),
            )),
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Url(url) => Tag::new(vec![String::from(URL), url.clone()]),
            Self::PublishedAt(timestamp) => {
                Tag::new(vec![String::from(PUBLISHED_AT), timestamp.to_string()])
            }
            Self::Title(title) => Tag::new(vec![String::from(TITLE), title.clone()]),
            Self::Hashtag(hashtag) => Tag::new(vec![String::from(HASHTAG), hashtag.to_lowercase()]),
        }
    }
}

impl_tag_codec_conversions!(NipB0Tag);

/// Web Bookmark
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WebBookmark {
    /// Description of the web bookmark.
    pub description: String,
    /// URL of the web bookmark.
    pub url: String,
    /// Timestamp when the web bookmark was first published.
    pub published_at: Option<Timestamp>,
    /// Title of the web bookmark.
    pub title: Option<String>,
    /// Hashtags for the web bookmark.
    pub hashtags: Vec<String>,
}

impl WebBookmark {
    /// Create a new web bookmark
    #[inline]
    pub fn new<T>(description: T, url: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            description: description.into(),
            url: url.into(),
            published_at: None,
            title: None,
            hashtags: Vec::new(),
        }
    }

    /// Set the title.
    #[inline]
    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    /// Set the timestamp at which the web bookmark was published.
    #[inline]
    pub fn published_at(mut self, timestamp: Timestamp) -> Self {
        self.published_at = Some(timestamp);
        self
    }

    /// Add a hashtag/tag.
    pub fn hashtags<T>(mut self, hashtag: T) -> Self
    where
        T: Into<String>,
    {
        let hashtag = hashtag.into().to_lowercase();
        if !self.hashtags.contains(&hashtag) {
            self.hashtags.push(hashtag);
        }
        self
    }

    /// Convert the web bookmark to an event builder
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let mut tags: Vec<Tag> = vec![NipB0Tag::Url(self.url).into()];

        let mut add_if_some = |tag: Option<NipB0Tag>| {
            if let Some(tag) = tag {
                tags.push(tag.into());
            }
        };

        add_if_some(self.published_at.map(NipB0Tag::PublishedAt));
        add_if_some(self.title.map(NipB0Tag::Title));

        for hashtag in self.hashtags.into_iter() {
            tags.push(NipB0Tag::Hashtag(hashtag).into());
        }

        EventBuilder::new(Kind::WebBookmark, self.description).tags(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_tag() {
        let tag = vec!["d", "alice.blog/post"];
        let parsed = NipB0Tag::parse(&tag).unwrap();
        assert_eq!(parsed, NipB0Tag::Url(String::from("alice.blog/post")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_published_at_tag() {
        let tag = vec!["published_at", "1738863000"];
        let parsed = NipB0Tag::parse(&tag).unwrap();
        assert_eq!(parsed, NipB0Tag::PublishedAt(Timestamp::from(1738863000)));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_title_tag() {
        let tag = vec!["title", "Blog insights by Alice"];
        let parsed = NipB0Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            NipB0Tag::Title(String::from("Blog insights by Alice"))
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_hashtag_tag() {
        let tag = vec!["t", "Insight"];
        let parsed = NipB0Tag::parse(&tag).unwrap();
        assert_eq!(parsed, NipB0Tag::Hashtag(String::from("insight")));
        assert_eq!(parsed.to_tag(), Tag::parse(vec!["t", "insight"]).unwrap());
    }
}
