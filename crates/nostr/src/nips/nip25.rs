// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP25: Reactions
//!
//! <https://github.com/nostr-protocol/nips/blob/master/25.md>

use alloc::string::{String, ToString};
use core::fmt;
use core::num::ParseIntError;

use super::nip01::{Coordinate, Nip01Tag};
use super::util::take_and_parse_from_str;
use crate::event::tag::{Tag, TagCodec, TagCodecError, Tags, impl_tag_codec_conversions};
use crate::{Event, EventId, Kind, PublicKey, RelayUrl};

/// NIP-25 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Failed to parse integer
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

/// Standardized NIP-25 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/25.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip25Tag {
    /// `k` tag
    Kind(Kind),
}

impl TagCodec for Nip25Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            "k" => {
                let kind: Kind = take_and_parse_from_str::<_, _, _, Error>(&mut iter, "kind")?;
                Ok(Self::Kind(kind))
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Kind(kind) => Tag::new(vec![String::from("k"), kind.to_string()]),
        }
    }
}

impl_tag_codec_conversions!(Nip25Tag);

/// Reaction target
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactionTarget {
    /// Event ID
    pub event_id: EventId,
    /// Public Key
    pub public_key: PublicKey,
    /// Coordinate
    pub coordinate: Option<Coordinate>,
    /// Kind
    pub kind: Option<Kind>,
    /// Relay hint
    pub relay_hint: Option<RelayUrl>,
}

impl ReactionTarget {
    /// Construct a new reaction target
    pub fn new(event: &Event, relay_hint: Option<RelayUrl>) -> Self {
        Self {
            event_id: event.id,
            public_key: event.pubkey,
            coordinate: event.coordinate(),
            kind: Some(event.kind),
            relay_hint,
        }
    }

    pub(crate) fn into_tags(self) -> Tags {
        let mut tags: Tags = Tags::with_capacity(
            2 + usize::from(self.coordinate.is_some()) + usize::from(self.kind.is_some()),
        );

        // Serialization order: keep the `e` and `a` tags together, followed by the `p` and other tags.

        tags.push(
            Nip01Tag::Event {
                id: self.event_id,
                relay_hint: self.relay_hint.clone(),
                public_key: Some(self.public_key),
            }
            .to_tag(),
        );

        if let Some(coordinate) = self.coordinate {
            tags.push(
                Nip01Tag::Coordinate {
                    coordinate,
                    relay_hint: self.relay_hint.clone(),
                }
                .to_tag(),
            );
        }

        tags.push(
            Nip01Tag::PublicKey {
                public_key: self.public_key,
                relay_hint: self.relay_hint,
            }
            .to_tag(),
        );

        if let Some(kind) = self.kind {
            tags.push(Nip25Tag::Kind(kind).to_tag());
        }

        tags
    }
}

impl From<&Event> for ReactionTarget {
    fn from(event: &Event) -> Self {
        Self {
            event_id: event.id,
            public_key: event.pubkey,
            coordinate: event.coordinate(),
            kind: Some(event.kind),
            relay_hint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_tag() {
        let tag = vec!["k", "1"];
        let parsed = Nip25Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip25Tag::Kind(Kind::TextNote));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
