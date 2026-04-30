// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-10: Conventions for clients' use of `e` and `p` tags in text events
//!
//! <https://github.com/nostr-protocol/nips/blob/master/10.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use super::util::{
    take_and_parse_optional_public_key, take_and_parse_optional_relay_url, take_event_id,
};
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::event::{self};
use crate::types::url;
use crate::{EventId, PublicKey, RelayUrl, key};

const EVENT: &str = "e";

/// NIP10 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Keys error
    Keys(key::Error),
    /// Event error
    Event(event::Error),
    /// Url error
    Url(url::Error),
    /// Codec error
    Codec(TagCodecError),
    /// Invalid marker
    InvalidMarker,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keys(e) => e.fmt(f),
            Self::Event(e) => e.fmt(f),
            Self::Url(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
            Self::InvalidMarker => f.write_str("invalid marker"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
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

/// Marker
///
/// <https://github.com/nostr-protocol/nips/blob/master/10.md>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Marker {
    /// Root
    Root,
    /// Reply
    Reply,
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Root => f.write_str("root"),
            Self::Reply => f.write_str("reply"),
        }
    }
}

impl FromStr for Marker {
    type Err = Error;

    fn from_str(marker: &str) -> Result<Self, Self::Err> {
        match marker {
            "root" => Ok(Self::Root),
            "reply" => Ok(Self::Reply),
            _ => Err(Error::InvalidMarker),
        }
    }
}

/// Standardized NIP-10 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/10.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip10Tag {
    /// `e` tag
    Event {
        /// Event ID
        id: EventId,
        /// Relay hint
        relay_hint: Option<RelayUrl>,
        /// Event marker
        marker: Option<Marker>,
        /// Public key hint
        public_key: Option<PublicKey>,
    },
}

impl Nip10Tag {
    /// Check if this tag is marked as a root reference.
    #[inline]
    pub fn is_root(&self) -> bool {
        matches!(
            self,
            Self::Event {
                marker: Some(Marker::Root),
                ..
            }
        )
    }

    /// Check if this tag is marked as a reply reference.
    #[inline]
    pub fn is_reply(&self) -> bool {
        matches!(
            self,
            Self::Event {
                marker: Some(Marker::Reply),
                ..
            }
        )
    }
}

impl TagCodec for Nip10Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            EVENT => {
                let (id, relay_hint, marker, public_key) = parse_e_tag(iter)?;
                Ok(Self::Event {
                    id,
                    relay_hint,
                    marker,
                    public_key,
                })
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Event {
                id,
                relay_hint,
                marker,
                public_key,
            } => {
                // ["e", <event-id>, <relay-url>, <marker>, <pubkey>]
                // <relay-url>, <marker> and <pubkey> are optional
                // <relay-url>, if empty, may be set to "" (if there are additional fields later)
                // <marker> is optional and if present is one of "reply" or "root" (so not an empty string)

                let mut tag: Vec<String> = Vec::with_capacity(
                    2 + relay_hint.is_some() as usize
                        + marker.is_some() as usize
                        + public_key.is_some() as usize,
                );

                tag.push(String::from(EVENT));
                tag.push(id.to_hex());

                // Check if relay hint exists or if there are additional fields after
                match (relay_hint, marker.is_some() || public_key.is_some()) {
                    (Some(relay_hint), ..) => tag.push(relay_hint.to_string()),
                    (None, true) => tag.push(String::new()),
                    (None, false) => {}
                }

                match (marker, public_key.is_some()) {
                    (Some(marker), _) => tag.push(marker.to_string()),
                    (None, true) => tag.push(String::new()),
                    (None, false) => {}
                }

                if let Some(public_key) = public_key {
                    tag.push(public_key.to_hex());
                }

                Tag::new(tag)
            }
        }
    }
}

impl_tag_codec_conversions!(Nip10Tag);

#[allow(clippy::type_complexity)]
fn parse_e_tag<T, S>(
    mut iter: T,
) -> Result<(EventId, Option<RelayUrl>, Option<Marker>, Option<PublicKey>), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let id: EventId = take_event_id::<_, _, Error>(&mut iter)?;
    let relay_hint: Option<RelayUrl> = take_and_parse_optional_relay_url(&mut iter)?;

    let marker: Option<Marker> = match iter.next() {
        Some(marker) => {
            let marker: &str = marker.as_ref();

            if !marker.is_empty() && marker != "mention" {
                Some(Marker::from_str(marker)?)
            } else {
                None
            }
        }
        _ => None,
    };

    let public_key: Option<PublicKey> = take_and_parse_optional_public_key(&mut iter)?;

    Ok((id, relay_hint, marker, public_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::PublicKey;

    #[test]
    fn test_parse_empty_tag() {
        let tag: Vec<String> = Vec::new();
        let err = Nip10Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::missing_tag_kind()));
    }

    #[test]
    fn test_non_existing_tag() {
        let tag = vec!["p"];
        let err = Nip10Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Unknown));
    }

    #[test]
    fn test_standardized_e_tag() {
        let relay_hint = RelayUrl::parse("wss://relay.example.com").unwrap();
        let public_key =
            PublicKey::from_hex("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        let tag = vec![
            String::from("e"),
            EventId::all_zeros().to_hex(),
            relay_hint.to_string(),
            String::from("root"),
            public_key.to_hex(),
        ];
        let parsed = Nip10Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip10Tag::Event {
                id: EventId::all_zeros(),
                relay_hint: Some(relay_hint),
                marker: Some(Marker::Root),
                public_key: Some(public_key),
            }
        );
        assert!(parsed.is_root());
        assert!(!parsed.is_reply());
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_positional_e_tag() {
        let tag = vec![String::from("e"), EventId::all_zeros().to_hex()];
        let parsed = Nip10Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip10Tag::Event {
                id: EventId::all_zeros(),
                relay_hint: None,
                marker: None,
                public_key: None,
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_standardized_e_tag_with_empty_marker() {
        let public_key =
            PublicKey::from_hex("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        let tag = vec![
            String::from("e"),
            EventId::all_zeros().to_hex(),
            String::from("wss://relay.example.com"),
            String::new(),
            public_key.to_hex(),
        ];
        let parsed = Nip10Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip10Tag::Event {
                id: EventId::all_zeros(),
                relay_hint: Some(RelayUrl::parse("wss://relay.example.com").unwrap()),
                marker: None,
                public_key: Some(public_key),
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_standardized_e_tag_without_marker_is_invalid() {
        let public_key =
            PublicKey::from_hex("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        let tag = vec![
            String::from("e"),
            EventId::all_zeros().to_hex(),
            String::from("wss://relay.example.com"),
            public_key.to_hex(),
        ];
        let err = Nip10Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::InvalidMarker);
    }

    #[test]
    fn test_e_tag_with_mention_marker() {
        let hex = "19bb195b83fd26db217b6feebb444de4808d90eb4375c31c75ba5bb5c5c10cfc";
        let id = EventId::from_hex(hex).unwrap();

        let result = Nip10Tag::parse(["e", hex, "", "mention"]);

        assert_eq!(
            result,
            Ok(Nip10Tag::Event {
                id,
                relay_hint: None,
                marker: None,
                public_key: None,
            })
        );
    }
}
