// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-17: Private Direct Message
//!
//! <https://github.com/nostr-protocol/nips/blob/master/17.md>

#![allow(rustdoc::redundant_explicit_links)]

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use super::nip44::{AsyncNip44, Nip44};
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use super::nip59::{self, GiftWrapBuilder};
use super::util::take_relay_url;
use crate::event::Event;
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use crate::event::unsigned::FinalizeUnsignedEvent;
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use crate::event::{EventBuilder, FinalizeEvent, FinalizeEventAsync, Kind, UnsignedEvent};
use crate::key::PublicKey;
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use crate::signer::{AsyncGetPublicKey, AsyncSignEvent, GetPublicKey, SignEvent, SignerError};
use crate::types::url::{self, RelayUrl};
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use crate::util::{BoxedFuture, UnwrapInfallible};

const RELAY: &str = "relay";

/// NIP-17 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Url error
    Url(url::Error),
    /// Codec error
    Codec(TagCodecError),
    /// Signer error
    #[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
    Signer(SignerError),
    /// NIP-59 error
    #[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
    NIP59(nip59::Error),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
            #[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
            Self::Signer(e) => e.fmt(f),
            #[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
            Self::NIP59(e) => e.fmt(f),
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

#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
impl From<SignerError> for Error {
    fn from(e: SignerError) -> Self {
        Self::Signer(e)
    }
}

#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
impl From<nip59::Error> for Error {
    fn from(e: nip59::Error) -> Self {
        Self::NIP59(e)
    }
}

/// Private Direct Message event builder.
///
/// # Example
///
/// ```rust,no_run
/// # use nostr::prelude::*;
/// # #[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let receiver = PublicKey::from_hex("<receiver-public-key>")?;
/// let signer = Keys::parse("<my-secret-key>")?;
/// let private_msg: Event =
///     PrivateDirectMessageBuilder::new(receiver, "Hello, world!").finalize(&signer)?;
/// # Ok(())
/// # }
/// # #[cfg(not(all(feature = "std", feature = "os-rng", feature = "nip59")))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone)]
pub struct PrivateDirectMessageBuilder {
    /// Receiver public key.
    pub receiver: PublicKey,
    /// Message.
    pub message: String,
    /// Extra tags to add to the **rumor** event.
    pub rumor_extra_tags: Vec<Tag>,
    /// Extra tags to add to the **gift wrap** event.
    pub extra_tags: Vec<Tag>,
}

// TODO: should this be under the required features, like for the Finalize traits?
impl PrivateDirectMessageBuilder {
    /// Create a new private direct message event builder.
    #[inline]
    pub fn new<M>(receiver: PublicKey, message: M) -> Self
    where
        M: Into<String>,
    {
        Self {
            receiver,
            message: message.into(),
            rumor_extra_tags: Vec::new(),
            extra_tags: Vec::new(),
        }
    }

    /// Extra tags to add to the **rumor** event.
    #[inline]
    pub fn rumor_extra_tags<T>(mut self, tags: T) -> Self
    where
        T: IntoIterator<Item = Tag>,
    {
        self.rumor_extra_tags.extend(tags);
        self
    }

    /// Extra tags to add to the **gift wrap** event.
    #[inline]
    pub fn extra_tags<T>(mut self, tags: T) -> Self
    where
        T: IntoIterator<Item = Tag>,
    {
        self.extra_tags.extend(tags);
        self
    }
}

#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
impl<S> FinalizeEvent<S> for PrivateDirectMessageBuilder
where
    S: GetPublicKey + SignEvent + Nip44,
{
    type Error = Error;

    fn finalize(self, signer: &S) -> Result<Event, Self::Error> {
        let public_key: PublicKey = signer.get_public_key()?;
        let rumor: UnsignedEvent = make_rumor(
            public_key,
            self.receiver,
            self.message,
            self.rumor_extra_tags,
        );
        Ok(GiftWrapBuilder::new(rumor, self.receiver)
            .extra_tags(self.extra_tags)
            .finalize(signer)?)
    }
}

#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
impl<S> FinalizeEventAsync<S> for PrivateDirectMessageBuilder
where
    S: AsyncGetPublicKey + AsyncSignEvent + AsyncNip44,
{
    type Error = Error;

    fn finalize_async<'a>(self, signer: &'a S) -> BoxedFuture<'a, Result<Event, Self::Error>>
    where
        Self: 'a,
        S: 'a,
    {
        Box::pin(async move {
            let public_key: PublicKey = signer.get_public_key().await?;
            let rumor: UnsignedEvent = make_rumor(
                public_key,
                self.receiver,
                self.message,
                self.rumor_extra_tags,
            );
            Ok(GiftWrapBuilder::new(rumor, self.receiver)
                .extra_tags(self.extra_tags)
                .finalize_async(signer)
                .await?)
        })
    }
}

#[inline]
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
fn make_rumor(
    sender: PublicKey,
    receiver: PublicKey,
    message: String,
    extra_tags: Vec<Tag>,
) -> UnsignedEvent {
    EventBuilder::new(Kind::PrivateDirectMessage, message)
        .tag(Tag::public_key(receiver))
        .tags(extra_tags)
        .finalize_unsigned(sender)
        .unwrap_infallible()
}

/// Standardized NIP-17 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/17.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip17Tag {
    /// Relay
    ///
    /// `["relay", <relay URL>]`
    Relay(RelayUrl),
}

impl TagCodec for Nip17Tag {
    type Error = Error;

    /// Parse NIP-17 standardized tag
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
            RELAY => {
                let url: RelayUrl = take_relay_url::<_, _, Error>(&mut iter)?;
                Ok(Self::Relay(url))
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        let Self::Relay(url) = self;
        let tag: Vec<String> = vec![String::from(RELAY), url.to_string()];
        Tag::new(tag)
    }
}

impl_tag_codec_conversions!(Nip17Tag);

/// Extracts the relay list
///
/// This function doesn't verify if the event kind is [`Kind::InboxRelays`](crate::Kind::InboxRelays)!
pub fn extract_relay_list(event: &Event) -> impl Iterator<Item = RelayUrl> + '_ {
    event
        .tags
        .iter()
        .filter_map(|tag| match Nip17Tag::parse(tag.as_slice()) {
            Ok(Nip17Tag::Relay(url)) => Some(url),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_tag() {
        let tag: Vec<String> = Vec::new();
        let err = Nip17Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::missing_tag_kind()));
    }

    #[test]
    fn test_non_existing_tag() {
        let tag = vec!["p"];
        let err = Nip17Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Unknown));
    }

    #[test]
    fn test_standardized_relay_tag() {
        let relay = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let tag = vec!["relay".to_string(), relay.to_string()];

        let parsed = Nip17Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip17Tag::Relay(relay.clone()));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_missing_relay_url() {
        let tag = vec!["relay"];
        let err = Nip17Tag::parse(&tag).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("relay URL")));
    }
}
