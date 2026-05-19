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

#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use super::nip44::{AsyncNip44, Nip44};
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use super::nip59::GiftWrapBuilder;
use super::util::{missing_tag_kind, take_relay_url, unknown_tag};
use crate::error::Error;
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use crate::event::{
    AsyncSignEvent, EventBuilder, FinalizeEvent, FinalizeEventAsync, FinalizeUnsignedEvent, Kind,
    SignEvent, UnsignedEvent,
};
use crate::event::{Event, Tag, TagCodec, impl_tag_codec_conversions};
use crate::key::PublicKey;
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use crate::key::{AsyncGetPublicKey, GetPublicKey};
use crate::types::url::RelayUrl;
#[cfg(all(feature = "std", feature = "os-rng", feature = "nip59"))]
use crate::util::BoxedFuture;

const RELAY: &str = "relay";

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
#[non_exhaustive]
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
        let public_key: PublicKey = signer.get_public_key().map_err(Error::other)?;
        let rumor: UnsignedEvent = make_rumor(
            public_key,
            self.receiver,
            self.message,
            self.rumor_extra_tags,
        );
        GiftWrapBuilder::new(self.receiver, rumor)
            .extra_tags(self.extra_tags)
            .finalize(signer)
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
            let public_key: PublicKey =
                signer.get_public_key_async().await.map_err(Error::other)?;
            let rumor: UnsignedEvent = make_rumor(
                public_key,
                self.receiver,
                self.message,
                self.rumor_extra_tags,
            );
            GiftWrapBuilder::new(self.receiver, rumor)
                .extra_tags(self.extra_tags)
                .finalize_async(signer)
                .await
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

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // Take iterator
        let mut iter = tag.into_iter();

        // Extract first value
        let kind: S = iter.next().ok_or(missing_tag_kind())?;

        // Match kind
        match kind.as_ref() {
            RELAY => {
                let url: RelayUrl = take_relay_url(&mut iter)?;
                Ok(Self::Relay(url))
            }
            _ => Err(unknown_tag()),
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
    use crate::error::ErrorKind;

    #[test]
    fn test_parse_empty_tag() {
        let tag: Vec<String> = Vec::new();
        let err = Nip17Tag::parse(&tag).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Missing);
    }

    #[test]
    fn test_non_existing_tag() {
        let tag = vec!["p"];
        let err = Nip17Tag::parse(&tag).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Malformed);
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
        assert_eq!(err.kind(), ErrorKind::Missing);
    }
}
