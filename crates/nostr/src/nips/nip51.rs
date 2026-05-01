// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-51: Lists
//!
//! <https://github.com/nostr-protocol/nips/blob/master/51.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use super::nip01::Coordinate;
use super::util::{take_event_id, take_public_key, take_relay_url, take_string};
use crate::event::tag::{Tag, TagCodec, TagCodecError, TagStandard, impl_tag_codec_conversions};
use crate::types::url::{self, RelayUrl, Url};
use crate::{EventId, PublicKey, event, key};

const WORD: &str = "word";
const PUBLIC_KEY: &str = "p";
const HASHTAG: &str = "t";
const EVENT: &str = "e";
const RELAY: &str = "relay";

/// NIP-51 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Event error
    Event(event::Error),
    /// Key error
    Key(key::Error),
    /// Url error
    Url(url::Error),
    /// Codec error
    Codec(TagCodecError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Event(e) => e.fmt(f),
            Self::Key(e) => e.fmt(f),
            Self::Url(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
        }
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
    }
}

impl From<url::Error> for Error {
    fn from(e: url::Error) -> Self {
        Self::Url(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Standardized NIP-51 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/51.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip51Tag {
    /// `p` tag
    PublicKey(PublicKey),
    /// `t` tag
    Hashtag(String),
    /// `e` tag
    Event(EventId),
    /// `relay` tag
    Relay(RelayUrl),
    /// `word` tag
    Word(String),
}

impl TagCodec for Nip51Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            PUBLIC_KEY => {
                let public_key: PublicKey = take_public_key::<_, _, Error>(&mut iter)?;
                Ok(Self::PublicKey(public_key))
            }
            HASHTAG => {
                let hashtag: String = take_string(&mut iter, "hashtag")?;
                Ok(Self::Hashtag(hashtag.to_lowercase()))
            }
            EVENT => {
                let event_id: EventId = take_event_id::<_, _, Error>(&mut iter)?;
                Ok(Self::Event(event_id))
            }
            RELAY => {
                let relay_url: RelayUrl = take_relay_url::<_, _, Error>(&mut iter)?;
                Ok(Self::Relay(relay_url))
            }
            WORD => Ok(Self::Word(take_string(&mut iter, "word")?)),
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::PublicKey(public_key) => {
                Tag::new(vec![String::from(PUBLIC_KEY), public_key.to_hex()])
            }
            Self::Hashtag(hashtag) => Tag::new(vec![String::from(HASHTAG), hashtag.to_lowercase()]),
            Self::Event(event_id) => Tag::new(vec![String::from(EVENT), event_id.to_hex()]),
            Self::Relay(relay_url) => Tag::new(vec![String::from(RELAY), relay_url.to_string()]),
            Self::Word(word) => Tag::new(vec![String::from(WORD), word.clone()]),
        }
    }
}

impl_tag_codec_conversions!(Nip51Tag);

/// Things the user doesn't want to see in their feeds
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MuteList {
    /// Public Keys
    pub public_keys: Vec<PublicKey>,
    /// Hashtags
    pub hashtags: Vec<String>,
    /// Event IDs
    pub event_ids: Vec<EventId>,
    /// Words
    pub words: Vec<String>,
}

impl From<MuteList> for Vec<Tag> {
    fn from(
        MuteList {
            public_keys,
            hashtags,
            event_ids,
            words,
        }: MuteList,
    ) -> Self {
        let mut tags =
            Vec::with_capacity(public_keys.len() + hashtags.len() + event_ids.len() + words.len());

        tags.extend(
            public_keys
                .into_iter()
                .map(Nip51Tag::PublicKey)
                .map(Into::into),
        );
        tags.extend(hashtags.into_iter().map(Nip51Tag::Hashtag).map(Into::into));
        tags.extend(event_ids.into_iter().map(Nip51Tag::Event).map(Into::into));
        tags.extend(words.into_iter().map(Nip51Tag::Word).map(Into::into));

        tags
    }
}

/// Uncategorized, "global" list of things a user wants to save
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bookmarks {
    /// Event IDs
    pub event_ids: Vec<EventId>,
    /// Coordinates
    pub coordinate: Vec<Coordinate>,
}

impl From<Bookmarks> for Vec<Tag> {
    fn from(
        Bookmarks {
            event_ids,
            coordinate,
        }: Bookmarks,
    ) -> Self {
        let mut tags = Vec::with_capacity(event_ids.len() + coordinate.len());

        tags.extend(event_ids.into_iter().map(Tag::event));
        tags.extend(coordinate.into_iter().map(Tag::from));

        tags
    }
}

/// Topics a user may be interested in and pointers
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interests {
    /// Hashtags
    pub hashtags: Vec<String>,
    /// Coordinates
    pub coordinate: Vec<Coordinate>,
}

impl From<Interests> for Vec<Tag> {
    fn from(
        Interests {
            hashtags,
            coordinate,
        }: Interests,
    ) -> Self {
        let mut tags = Vec::with_capacity(hashtags.len() + coordinate.len());

        tags.extend(hashtags.into_iter().map(Tag::hashtag));
        tags.extend(coordinate.into_iter().map(Tag::from));

        tags
    }
}

/// User preferred emojis and pointers to emoji sets
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Emojis {
    /// Emojis
    pub emojis: Vec<(String, Url)>,
    /// Coordinates
    pub coordinate: Vec<Coordinate>,
}

impl From<Emojis> for Vec<Tag> {
    fn from(Emojis { emojis, coordinate }: Emojis) -> Self {
        let mut tags = Vec::with_capacity(emojis.len() + coordinate.len());

        tags.extend(
            emojis
                .into_iter()
                .map(|(s, url)| Tag::from_standardized(TagStandard::Emoji { shortcode: s, url })),
        );
        tags.extend(coordinate.into_iter().map(Tag::from));

        tags
    }
}

/// Groups of articles picked by users as interesting and/or belonging to the same category
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArticlesCuration {
    /// Coordinates
    pub coordinate: Vec<Coordinate>,
    /// Event IDs
    pub event_ids: Vec<EventId>,
}

impl From<ArticlesCuration> for Vec<Tag> {
    fn from(
        ArticlesCuration {
            coordinate,
            event_ids,
        }: ArticlesCuration,
    ) -> Self {
        let mut tags = Vec::with_capacity(coordinate.len() + event_ids.len());

        tags.extend(coordinate.into_iter().map(Tag::from));
        tags.extend(event_ids.into_iter().map(Tag::event));

        tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_tag() {
        let public_key =
            PublicKey::from_hex("04c915daefee38317fa734444acee390a8269fe5810b2241e5e6dd343dfbecc9")
                .unwrap();
        let tag = vec![
            "p",
            "04c915daefee38317fa734444acee390a8269fe5810b2241e5e6dd343dfbecc9",
        ];
        let parsed = Nip51Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip51Tag::PublicKey(public_key));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_hashtag_tag() {
        let tag = vec!["t", "Nostr"];
        let parsed = Nip51Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip51Tag::Hashtag(String::from("nostr")));
        assert_eq!(parsed.to_tag(), Tag::parse(["t", "nostr"]).unwrap());
    }

    #[test]
    fn test_event_tag() {
        let event_id =
            EventId::from_hex("9ae37aa68f48645127299e9453eb5d908a0cbb6058ff340d528ed4d37c8994fb")
                .unwrap();
        let tag = vec![
            "e",
            "9ae37aa68f48645127299e9453eb5d908a0cbb6058ff340d528ed4d37c8994fb",
        ];
        let parsed = Nip51Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip51Tag::Event(event_id));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_word_tag() {
        let tag = vec!["word", "spam"];
        let parsed = Nip51Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip51Tag::Word(String::from("spam")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_relay_tag() {
        let tag = vec!["relay", "wss://relay.damus.io"];
        let parsed = Nip51Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip51Tag::Relay(RelayUrl::parse("wss://relay.damus.io").unwrap())
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
