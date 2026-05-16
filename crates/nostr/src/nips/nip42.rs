// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP42: Authentication of clients to relays
//!
//! <https://github.com/nostr-protocol/nips/blob/master/42.md>

use alloc::string::{String, ToString};
use alloc::vec;
use core::fmt;

use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::nips::util::{take_relay_url, take_string};
use crate::types::url;
use crate::{Event, Kind, RelayUrl};

const CHALLENGE: &str = "challenge";
const RELAY: &str = "relay";

/// NIP-42 error
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

/// Standardized NIP-42 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/42.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip42Tag {
    /// Authentication challenge
    Challenge(String),
    /// Relay URL
    Relay(RelayUrl),
}

impl TagCodec for Nip42Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            CHALLENGE => Ok(Self::Challenge(take_string(&mut iter, "challenge")?)),
            RELAY => {
                let relay_url: RelayUrl = take_relay_url::<_, _, Error>(&mut iter)?;
                Ok(Self::Relay(relay_url))
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Challenge(challenge) => {
                Tag::new(vec![String::from(CHALLENGE), challenge.clone()])
            }
            Self::Relay(relay) => Tag::new(vec![String::from(RELAY), relay.to_string()]),
        }
    }
}

impl_tag_codec_conversions!(Nip42Tag);

/// Check if the [`Event`] is a valid authentication.
///
/// This function checks for:
/// - event kind, that must be [`Kind::Authentication`];
/// - `relay` tag, that must match `relay_url` arg;
/// - `challenge` tag, that must match `challenge` arg.
///
/// If all the above checks pass, returns `true`.
pub fn is_valid_auth_event(event: &Event, relay_url: &RelayUrl, challenge: &str) -> bool {
    // Check event kind
    if event.kind != Kind::Authentication {
        return false;
    }

    // Check if it has "relay" tag
    let relay_matches: bool = event.tags.iter().any(|tag| match Nip42Tag::try_from(tag) {
        Ok(Nip42Tag::Relay(url)) => &url == relay_url,
        _ => false,
    });

    if !relay_matches {
        return false;
    }

    // Check if it has the challenge
    let challenge_matches: bool = event.tags.iter().any(|tag| match Nip42Tag::try_from(tag) {
        Ok(Nip42Tag::Challenge(value)) => value == challenge,
        _ => false,
    });

    if !challenge_matches {
        return false;
    }

    // Valid
    true
}

#[cfg(all(test, feature = "std", feature = "os-rng"))]
mod tests {
    use super::*;
    use crate::event::FinalizeEvent;
    use crate::{EventBuilder, Keys};

    #[test]
    fn test_standardized_challenge_tag() {
        let tag = vec!["challenge".to_string(), "1234567890".to_string()];
        let parsed = Nip42Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip42Tag::Challenge(String::from("1234567890")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_standardized_relay_tag() {
        let relay = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let tag = vec!["relay".to_string(), relay.to_string()];
        let parsed = Nip42Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip42Tag::Relay(relay.clone()));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
        assert_eq!(
            Nip42Tag::try_from(Tag::parse(["relay", "wss://relay.damus.io"]).unwrap()).unwrap(),
            Nip42Tag::Relay(relay)
        );
    }

    #[test]
    fn test_valid_auth_event() {
        let keys = Keys::generate();
        let relay_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let challenge = "1234567890";

        let event = EventBuilder::auth(challenge, relay_url.clone())
            .finalize(&keys)
            .unwrap();

        assert!(is_valid_auth_event(&event, &relay_url, challenge));
    }

    #[test]
    fn test_invalid_auth_event() {
        let keys = Keys::generate();
        let relay_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let challenge = "1234567890";

        // Wrong challenge
        let event = EventBuilder::auth("abcd", relay_url.clone())
            .finalize(&keys)
            .unwrap();
        assert!(!is_valid_auth_event(&event, &relay_url, challenge));

        // Wrong relay url
        let event = EventBuilder::auth(challenge, RelayUrl::parse("wss://example.com").unwrap())
            .finalize(&keys)
            .unwrap();
        assert!(!is_valid_auth_event(&event, &relay_url, challenge));

        // Wrong kind
        let event = EventBuilder::text_note("abcd").finalize(&keys).unwrap();
        assert!(!is_valid_auth_event(&event, &relay_url, challenge));
    }
}
