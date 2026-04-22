// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-62: Request to Vanish
//!
//! <https://github.com/nostr-protocol/nips/blob/master/62.md>

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::types::url::{self, RelayUrl};

const RELAY: &str = "relay";
const ALL_RELAYS: &str = "ALL_RELAYS";

/// NIP-70 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Url error
    Url(url::Error),
    /// Codec error
    Codec(TagCodecError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
        }
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

/// Request to Vanish target
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VanishTarget {
    /// Request to vanish from all relays
    AllRelays,
    /// Request to vanish from a specific list of relays.
    Relays(Vec<RelayUrl>),
}

impl VanishTarget {
    /// Vanish from a single relay
    #[inline]
    pub fn relay(relay: RelayUrl) -> Self {
        Self::Relays(vec![relay])
    }

    /// Vanish from multiple relays
    #[inline]
    pub fn relays<I>(relays: I) -> Self
    where
        I: IntoIterator<Item = RelayUrl>,
    {
        Self::Relays(relays.into_iter().collect())
    }

    /// Vanish from all relays
    pub fn all_relays() -> Self {
        Self::AllRelays
    }
}

/// Standardized NIP-62 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/62.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip62Tag {
    /// Global Request to Vanish
    ///
    /// `["relay", "ALL_RELAYS"]`
    AllRelays,
    /// Request to Vanish from Relay
    ///
    /// `["relay", "<relay-url>"]`
    Relay(RelayUrl),
}

impl TagCodec for Nip62Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // Take iterator
        let mut iter = tag.into_iter();

        // Extract first value
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        // Match kind
        match kind.as_ref() {
            RELAY => parse_relay_tag(iter),
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::AllRelays => Tag::new(vec![String::from(RELAY), String::from(ALL_RELAYS)]),
            Self::Relay(relay) => Tag::new(vec![String::from(RELAY), relay.to_string()]),
        }
    }
}

impl_tag_codec_conversions!(Nip62Tag);

fn parse_relay_tag<T, S>(mut iter: T) -> Result<Nip62Tag, Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let relay_url: S = iter.next().ok_or(TagCodecError::Missing("relay URL"))?;

    match relay_url.as_ref() {
        ALL_RELAYS => Ok(Nip62Tag::AllRelays),
        other => {
            let relay_url: RelayUrl = RelayUrl::parse(other)?;
            Ok(Nip62Tag::Relay(relay_url))
        }
    }
}

/// Check whether a NIP-62 request to vanish applies to the given relay.
///
/// Returns `true` if the event is a `kind:62` vanish request and it contains
/// either:
/// - an `["relay", "ALL_RELAYS"]` tag, or
/// - an `["relay", "<relay-url>"]` tag matching `relay_url`.
///
/// Returns `false` for events with a different kind or for events that do not
/// target the given relay.
pub fn is_valid_vanish_request_for_relay(tags: &[Tag], relay_url: Option<&RelayUrl>) -> bool {
    tags.iter().any(|tag| match Nip62Tag::try_from(tag) {
        Ok(Nip62Tag::AllRelays) => true,
        Ok(Nip62Tag::Relay(relay)) => Some(&relay) == relay_url,
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use crate::{EventBuilder, PublicKey};

    #[test]
    fn test_standardized_relay_tag() {
        let tag = vec!["relay", "wss://relay.damus.io"];
        let parsed = Nip62Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip62Tag::Relay(RelayUrl::parse("wss://relay.damus.io").unwrap())
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_standardized_all_relays_tag() {
        let tag = vec!["relay", "ALL_RELAYS"];
        let parsed = Nip62Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip62Tag::AllRelays);
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_is_valid_vanish_request_for_relay() {
        let relay_a = RelayUrl::parse("wss://relay.a.com").unwrap();
        let relay_b = RelayUrl::parse("wss://relay.b.com").unwrap();

        let all_relays = EventBuilder::request_vanish(VanishTarget::all_relays())
            .unwrap()
            .build(
                PublicKey::from_hex(
                    "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
                )
                .unwrap(),
            );

        assert!(is_valid_vanish_request_for_relay(
            all_relays.tags.as_slice(),
            Some(&relay_a)
        ));
        assert!(is_valid_vanish_request_for_relay(
            all_relays.tags.as_slice(),
            Some(&relay_b)
        ));

        let single_relay = EventBuilder::request_vanish(VanishTarget::relay(relay_a.clone()))
            .unwrap()
            .build(
                PublicKey::from_hex(
                    "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
                )
                .unwrap(),
            );

        assert!(is_valid_vanish_request_for_relay(
            single_relay.tags.as_slice(),
            Some(&relay_a)
        ));
        assert!(!is_valid_vanish_request_for_relay(
            single_relay.tags.as_slice(),
            Some(&relay_b)
        ));

        let other_kind = EventBuilder::text_note("hello").build(
            PublicKey::from_hex("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
                .unwrap(),
        );

        assert!(!is_valid_vanish_request_for_relay(
            other_kind.tags.as_slice(),
            Some(&relay_a)
        ));
    }
}
