// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP59: Gift Wrap
//!
//! <https://github.com/nostr-protocol/nips/blob/master/59.md>

#[cfg(all(feature = "std", feature = "os-rng"))]
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
#[cfg(all(feature = "std", feature = "os-rng"))]
use core::ops::Range;

use secp256k1::{Secp256k1, Verification};

#[cfg(feature = "std")]
use crate::SECP256K1;
use crate::event::unsigned::UnsignedEvent;
use crate::event::{self, Event};
#[cfg(all(feature = "std", feature = "os-rng"))]
use crate::event::{FinalizeEvent, FinalizeEventAsync};
#[cfg(all(feature = "std", feature = "os-rng"))]
use crate::key::Keys;
#[cfg(all(feature = "std", feature = "os-rng"))]
use crate::nips::nip44;
use crate::nips::nip44::{AsyncNip44, Nip44};
#[cfg(all(feature = "std", feature = "os-rng"))]
use crate::signer::{AsyncGetPublicKey, AsyncSignEvent, GetPublicKey, SignEvent, SignerError};
#[cfg(all(feature = "std", feature = "os-rng"))]
use crate::util::BoxedFuture;
#[cfg(all(feature = "std", feature = "os-rng"))]
use crate::{EventBuilder, Timestamp};
use crate::{JsonUtil, Kind, PublicKey, Tag};

#[cfg(all(feature = "std", feature = "os-rng"))]
const RANGE_RANDOM_TIMESTAMP_TWEAK: Range<u64> = 0..172800; // From 0 secs to 2 days

/// NIP59 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Signer error
    #[cfg(all(feature = "std", feature = "os-rng"))]
    Signer(SignerError),
    /// Event error
    Event(event::Error),
    /// NIP-44 error
    NIP44(String),
    /// Not Gift Wrap event
    NotGiftWrap,
    /// Rumor author does not match the seal signer
    SenderMismatch,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(all(feature = "std", feature = "os-rng"))]
            Self::Signer(e) => e.fmt(f),
            Self::Event(e) => e.fmt(f),
            Self::NIP44(e) => e.fmt(f),
            Self::NotGiftWrap => f.write_str("Not a Gift Wrap"),
            Self::SenderMismatch => f.write_str("sender public key mismatch"),
        }
    }
}

#[cfg(all(feature = "std", feature = "os-rng"))]
impl From<SignerError> for Error {
    fn from(e: SignerError) -> Self {
        Self::Signer(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

/// Unwrapped Gift Wrap (NIP59)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnwrappedGift {
    /// The public key of the sender included in the `seal`
    pub sender: PublicKey,
    /// The rumor
    pub rumor: UnsignedEvent,
}

impl UnwrappedGift {
    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    #[inline]
    #[cfg(feature = "std")]
    pub fn from_gift_wrap<T>(signer: &T, gift_wrap: &Event) -> Result<Self, Error>
    where
        T: Nip44,
    {
        Self::from_gift_wrap_with_ctx(&SECP256K1, signer, gift_wrap)
    }

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    #[inline]
    #[cfg(feature = "std")]
    pub async fn from_gift_wrap_async<T>(signer: &T, gift_wrap: &Event) -> Result<Self, Error>
    where
        T: AsyncNip44,
    {
        Self::from_gift_wrap_with_ctx_async(&SECP256K1, signer, gift_wrap).await
    }

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    pub fn from_gift_wrap_with_ctx<C, T>(
        secp: &Secp256k1<C>,
        signer: &T,
        gift_wrap: &Event,
    ) -> Result<Self, Error>
    where
        C: Verification,
        T: Nip44,
    {
        // Check event kind
        if gift_wrap.kind != Kind::GiftWrap {
            return Err(Error::NotGiftWrap);
        }

        // Decrypt and verify seal
        let seal: String = signer
            .nip44_decrypt(&gift_wrap.pubkey, &gift_wrap.content)
            .map_err(|e| Error::NIP44(e.to_string()))?;
        let seal: Event = parse_and_verify_seal(secp, seal)?;

        // Decrypt rumor
        let rumor: String = signer
            .nip44_decrypt(&seal.pubkey, &seal.content)
            .map_err(|e| Error::NIP44(e.to_string()))?;

        parse_unwrapped_gift(seal, rumor)
    }

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    pub async fn from_gift_wrap_with_ctx_async<C, T>(
        secp: &Secp256k1<C>,
        signer: &T,
        gift_wrap: &Event,
    ) -> Result<Self, Error>
    where
        C: Verification,
        T: AsyncNip44,
    {
        // Check event kind
        if gift_wrap.kind != Kind::GiftWrap {
            return Err(Error::NotGiftWrap);
        }

        // Decrypt and verify seal
        let seal: String = signer
            .nip44_decrypt(&gift_wrap.pubkey, &gift_wrap.content)
            .await
            .map_err(|e| Error::NIP44(e.to_string()))?;
        let seal: Event = parse_and_verify_seal(secp, seal)?;

        // Decrypt rumor
        let rumor: String = signer
            .nip44_decrypt(&seal.pubkey, &seal.content)
            .await
            .map_err(|e| Error::NIP44(e.to_string()))?;

        parse_unwrapped_gift(seal, rumor)
    }
}

/// Gift wrap seal event builder.
///
/// # Example
///
/// ```rust,no_run
/// # use nostr::prelude::*;
/// # #[cfg(all(feature = "std", feature = "os-rng"))]
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let receiver = PublicKey::from_hex("<receiver-public-key>")?;
/// let rumor = UnsignedEvent::from_json("<rumor-json>")?;
/// let signer = Keys::parse("<my-secret-key>")?;
/// let seal: Event = GiftWrapSealBuilder::new(rumor, receiver).finalize(&signer)?;
/// # Ok(())
/// # }
/// # #[cfg(not(all(feature = "std", feature = "os-rng")))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone)]
pub struct GiftWrapSealBuilder {
    /// The rumor
    pub rumor: UnsignedEvent,
    /// The public key of the receiver
    pub receiver: PublicKey,
}

impl GiftWrapSealBuilder {
    /// Create a new gift wrap builder.
    #[inline]
    pub fn new(rumor: UnsignedEvent, receiver: PublicKey) -> Self {
        Self { rumor, receiver }
    }
}

#[cfg(all(feature = "std", feature = "os-rng"))]
impl<S> FinalizeEvent<S> for GiftWrapSealBuilder
where
    S: GetPublicKey + SignEvent + Nip44,
{
    type Error = Error;

    fn finalize(self, signer: &S) -> Result<Event, Self::Error> {
        // Encrypt content
        let content: String = signer
            .nip44_encrypt(&self.receiver, &self.rumor.as_json())
            .map_err(|e| Error::NIP44(e.to_string()))?;

        let seal: EventBuilder = build_seal(self.rumor, content);
        Ok(seal.finalize(signer)?)
    }
}

#[cfg(all(feature = "std", feature = "os-rng"))]
impl<S> FinalizeEventAsync<S> for GiftWrapSealBuilder
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
            // Encrypt content
            let content: String = signer
                .nip44_encrypt(&self.receiver, &self.rumor.as_json())
                .await
                .map_err(|e| Error::NIP44(e.to_string()))?;

            let seal: EventBuilder = build_seal(self.rumor, content);
            Ok(seal.finalize_async(signer).await?)
        })
    }
}

/// Gift wrap event builder.
///
/// # Example
///
/// ```rust,no_run
/// # use nostr::prelude::*;
/// # #[cfg(all(feature = "std", feature = "os-rng"))]
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let receiver = PublicKey::from_hex("<receiver-public-key>")?;
/// let rumor = UnsignedEvent::from_json("<rumor-json>")?;
/// let signer = Keys::parse("<my-secret-key>")?;
/// let gift_wrap: Event = GiftWrapBuilder::new(rumor, receiver).finalize(&signer)?;
/// # Ok(())
/// # }
/// # #[cfg(not(all(feature = "std", feature = "os-rng")))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone)]
pub struct GiftWrapBuilder {
    /// The rumor
    pub rumor: UnsignedEvent,
    /// The public key of the receiver
    pub receiver: PublicKey,
    /// Extra tags to add to the event
    pub extra_tags: Vec<Tag>,
}

impl GiftWrapBuilder {
    /// Create a new gift wrap builder.
    #[inline]
    pub fn new(rumor: UnsignedEvent, receiver: PublicKey) -> Self {
        Self {
            rumor,
            receiver,
            extra_tags: Vec::new(),
        }
    }

    /// Add extra tags.
    #[inline]
    pub fn extra_tags<T>(mut self, tags: T) -> Self
    where
        T: IntoIterator<Item = Tag>,
    {
        self.extra_tags.extend(tags);
        self
    }
}

#[cfg(all(feature = "std", feature = "os-rng"))]
impl<S> FinalizeEvent<S> for GiftWrapBuilder
where
    S: GetPublicKey + SignEvent + Nip44,
{
    type Error = Error;

    fn finalize(self, signer: &S) -> Result<Event, Self::Error> {
        let seal: Event = GiftWrapSealBuilder::new(self.rumor, self.receiver).finalize(signer)?;
        make_gift_wrap(seal, self.receiver, self.extra_tags)
    }
}

#[cfg(all(feature = "std", feature = "os-rng"))]
impl<S> FinalizeEventAsync<S> for GiftWrapBuilder
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
            let seal: Event = GiftWrapSealBuilder::new(self.rumor, self.receiver)
                .finalize_async(signer)
                .await?;
            make_gift_wrap(seal, self.receiver, self.extra_tags)
        })
    }
}

#[cfg(all(feature = "std", feature = "os-rng"))]
fn make_gift_wrap(seal: Event, receiver: PublicKey, extra_tags: Vec<Tag>) -> Result<Event, Error> {
    // Generate the random keys
    let keys: Keys = Keys::generate();

    // Encrypt content
    let content: String = nip44::encrypt(
        keys.secret_key(),
        &receiver,
        seal.as_json(),
        nip44::Version::default(),
    )
    .map_err(|e| Error::NIP44(e.to_string()))?;

    // Collect extra tags
    let mut tags: Vec<Tag> = extra_tags;

    // Push received public key
    tags.push(Tag::public_key(receiver));

    // Build the event with a tweaked timestamp
    Ok(EventBuilder::new(Kind::GiftWrap, content)
        .tags(tags)
        .custom_created_at(Timestamp::tweaked(RANGE_RANDOM_TIMESTAMP_TWEAK))
        .finalize(&keys)?)
}

fn parse_and_verify_seal<C>(secp: &Secp256k1<C>, seal: String) -> Result<Event, Error>
where
    C: Verification,
{
    let seal: Event = Event::from_json(seal)?;
    seal.verify_with_ctx(secp)?;
    Ok(seal)
}

fn parse_unwrapped_gift(seal: Event, rumor: String) -> Result<UnwrappedGift, Error> {
    let rumor: UnsignedEvent = UnsignedEvent::from_json(rumor)?;

    // Ensure the rumor author matches the seal
    if rumor.pubkey != seal.pubkey {
        return Err(Error::SenderMismatch);
    }

    Ok(UnwrappedGift {
        sender: seal.pubkey,
        rumor,
    })
}

/// Extract `rumor` from Gift Wrap event
#[inline]
#[cfg(feature = "std")]
pub fn extract_rumor<T>(signer: &T, gift_wrap: &Event) -> Result<UnwrappedGift, Error>
where
    T: Nip44,
{
    UnwrappedGift::from_gift_wrap(signer, gift_wrap)
}

/// Extract `rumor` from Gift Wrap event
#[inline]
#[cfg(feature = "std")]
pub async fn extract_rumor_async<T>(signer: &T, gift_wrap: &Event) -> Result<UnwrappedGift, Error>
where
    T: AsyncNip44,
{
    UnwrappedGift::from_gift_wrap_async(signer, gift_wrap).await
}

#[cfg(all(feature = "std", feature = "os-rng"))]
fn build_seal(mut rumor: UnsignedEvent, content: String) -> EventBuilder {
    // Take an `UnsignedEvent` as rumor and not an `EventBuilder`!
    // May be useful to take an `EventBuilder` but it can create issues:
    // if a dev passes the same cloned `EventBuilder` to send the rumor to multiple users,
    // the final rumor will have different `created_at` timestamps, so different event IDs.

    // Make sure that rumor has event ID
    rumor.ensure_id();

    // Compose builder
    EventBuilder::new(Kind::Seal, content)
        .custom_created_at(Timestamp::tweaked(RANGE_RANDOM_TIMESTAMP_TWEAK))
}

#[cfg(test)]
#[cfg(all(feature = "std", feature = "os-rng"))]
mod tests {
    use super::{Error, *};
    use crate::prelude::*;

    #[test]
    fn test_extract_rumor() {
        let sender_keys =
            Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_keys =
            Keys::parse("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();

        // Compose Gift Wrap event
        let rumor: UnsignedEvent = EventBuilder::text_note("Test")
            .finalize_unsigned(sender_keys.public_key)
            .unwrap_infallible();
        let event: Event = GiftWrapBuilder::new(rumor.clone(), receiver_keys.public_key())
            .finalize(&sender_keys)
            .unwrap();
        let unwrapped = extract_rumor(&receiver_keys, &event).unwrap();
        assert_eq!(unwrapped.sender, sender_keys.public_key());
        assert_eq!(unwrapped.rumor.kind, Kind::TextNote);
        assert_eq!(unwrapped.rumor.content, "Test");
        assert!(unwrapped.rumor.tags.is_empty());
        assert!(extract_rumor(&sender_keys, &event).is_err());

        let event: Event = EventBuilder::text_note("").finalize(&sender_keys).unwrap();
        assert!(matches!(
            extract_rumor(&receiver_keys, &event).unwrap_err(),
            Error::NotGiftWrap
        ));
    }

    #[tokio::test]
    async fn test_extract_rumor_async() {
        let sender_keys =
            Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_keys =
            Keys::parse("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();

        let rumor: UnsignedEvent = EventBuilder::text_note("Test")
            .finalize_unsigned(sender_keys.public_key)
            .unwrap_infallible();
        let event: Event = GiftWrapBuilder::new(rumor, receiver_keys.public_key())
            .finalize(&sender_keys)
            .unwrap();

        let unwrapped = extract_rumor_async(&receiver_keys, &event).await.unwrap();
        assert_eq!(unwrapped.sender, sender_keys.public_key());
        assert_eq!(unwrapped.rumor.kind, Kind::TextNote);
        assert_eq!(unwrapped.rumor.content, "Test");
    }

    #[test]
    fn test_make_seal() {
        let sender_keys =
            Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_keys =
            Keys::parse("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();

        let rumor: UnsignedEvent = EventBuilder::text_note("Test")
            .finalize_unsigned(sender_keys.public_key)
            .unwrap_infallible();
        let seal = GiftWrapSealBuilder::new(rumor, receiver_keys.public_key())
            .finalize(&sender_keys)
            .unwrap();

        assert_eq!(seal.kind, Kind::Seal);
        assert!(!seal.content.is_empty());
    }

    #[tokio::test]
    async fn test_make_seal_async() {
        let sender_keys =
            Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_keys =
            Keys::parse("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();

        let rumor: UnsignedEvent = EventBuilder::text_note("Test")
            .finalize_unsigned(sender_keys.public_key)
            .unwrap_infallible();
        let seal = GiftWrapSealBuilder::new(rumor, receiver_keys.public_key())
            .finalize_async(&sender_keys)
            .await
            .unwrap();

        assert_eq!(seal.kind, Kind::Seal);
        assert!(!seal.content.is_empty());
    }

    #[tokio::test]
    async fn test_sender_mismatch() {
        let sender_keys =
            Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_keys =
            Keys::parse("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let impersonated_keys =
            Keys::parse("5b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();

        // Construct a rumor that lies about its pubkey but is still wrapped/signed
        // by `sender_keys`. This mimics a spoofing attempt the recipient must reject.
        let rumor: UnsignedEvent = EventBuilder::text_note("spoofed")
            .finalize_unsigned(impersonated_keys.public_key())
            .unwrap_infallible();

        let gift_wrap: Event = GiftWrapBuilder::new(rumor, receiver_keys.public_key())
            .finalize(&sender_keys)
            .unwrap();

        match extract_rumor(&receiver_keys, &gift_wrap) {
            Err(Error::SenderMismatch) => {}
            other => panic!("expected SenderMismatch, got {other:?}"),
        }
    }
}
