// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP59: Gift Wrap
//!
//! <https://github.com/nostr-protocol/nips/blob/master/59.md>

use alloc::string::String;
use core::fmt;
use core::ops::Range;

use bitcoin::secp256k1::{Secp256k1, Verification};

use crate::event::unsigned::{self, UnsignedEvent};
use crate::event::{self, Event};
use crate::signer::SignerError;
#[cfg(feature = "std")]
use crate::{EventBuilder, Timestamp, SECP256K1};
use crate::{JsonUtil, Kind, NostrSigner, PublicKey};

/// Range for random timestamp tweak (up to 2 days)
pub const RANGE_RANDOM_TIMESTAMP_TWEAK: Range<u64> = 0..172800; // From 0 secs to 2 days

/// NIP59 error
#[derive(Debug)]
pub enum Error {
    /// Signer error
    Signer(SignerError),
    /// Event error
    Event(event::Error),
    /// Unsigned event error
    Unsigned(unsigned::Error),
    /// Not Gift Wrap event
    NotGiftWrap,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signer(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "Event: {e}"),
            Self::Unsigned(e) => write!(f, "Unsigned event: {e}"),
            Self::NotGiftWrap => write!(f, "Not Gift Wrap event"),
        }
    }
}

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

impl From<unsigned::Error> for Error {
    fn from(e: unsigned::Error) -> Self {
        Self::Unsigned(e)
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
    pub async fn from_gift_wrap<T>(signer: &T, gift_wrap: &Event) -> Result<Self, Error>
    where
        T: NostrSigner,
    {
        Self::from_gift_wrap_with_ctx(&SECP256K1, signer, gift_wrap).await
    }

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    pub async fn from_gift_wrap_with_ctx<C, T>(
        secp: &Secp256k1<C>,
        signer: &T,
        gift_wrap: &Event,
    ) -> Result<Self, Error>
    where
        C: Verification,
        T: NostrSigner,
    {
        // Check event kind
        if gift_wrap.kind != Kind::GiftWrap {
            return Err(Error::NotGiftWrap);
        }

        // Decrypt and verify seal
        let seal: String = signer
            .nip44_decrypt(&gift_wrap.pubkey, &gift_wrap.content)
            .await?;
        let seal: Event = Event::from_json(seal)?;
        seal.verify_with_ctx(secp)?;

        // Decrypt rumor
        let rumor: String = signer.nip44_decrypt(&seal.pubkey, &seal.content).await?;

        Ok(UnwrappedGift {
            sender: seal.pubkey,
            rumor: UnsignedEvent::from_json(rumor)?,
        })
    }
}

/// Extract `rumor` from Gift Wrap event
#[inline]
#[cfg(feature = "std")]
pub async fn extract_rumor<T>(signer: &T, gift_wrap: &Event) -> Result<UnwrappedGift, Error>
where
    T: NostrSigner,
{
    UnwrappedGift::from_gift_wrap(signer, gift_wrap).await
}

/// Create seal
#[cfg(feature = "std")]
pub async fn make_seal<T>(
    signer: &T,
    receiver_pubkey: &PublicKey,
    rumor: EventBuilder,
) -> Result<EventBuilder, Error>
where
    T: NostrSigner,
{
    // Get public key
    let public_key = signer.get_public_key().await?;

    // Build unsigned event
    let mut rumor: UnsignedEvent = rumor.build(public_key);

    // Make sure that rumor has event ID
    rumor.ensure_id();

    // Encrypt content
    let content: String = signer
        .nip44_encrypt(receiver_pubkey, &rumor.as_json())
        .await?;

    // Compose builder
    Ok(EventBuilder::new(Kind::Seal, content)
        .custom_created_at(Timestamp::tweaked(RANGE_RANDOM_TIMESTAMP_TWEAK)))
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventBuilder, Keys};

    #[tokio::test]
    async fn test_extract_rumor() {
        let sender_keys =
            Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_keys =
            Keys::parse("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();

        // Compose Gift Wrap event
        let rumor: EventBuilder = EventBuilder::text_note("Test");
        let event: Event =
            EventBuilder::gift_wrap(&sender_keys, &receiver_keys.public_key(), rumor.clone(), [])
                .await
                .unwrap();
        let unwrapped = extract_rumor(&receiver_keys, &event).await.unwrap();
        assert_eq!(unwrapped.sender, sender_keys.public_key());
        assert_eq!(unwrapped.rumor.kind, Kind::TextNote);
        assert_eq!(unwrapped.rumor.content, "Test");
        assert!(unwrapped.rumor.tags.is_empty());
        assert!(extract_rumor(&sender_keys, &event).await.is_err());

        let event: Event = EventBuilder::text_note("")
            .sign(&sender_keys)
            .await
            .unwrap();
        assert!(matches!(
            extract_rumor(&receiver_keys, &event).await.unwrap_err(),
            Error::NotGiftWrap
        ));
    }
}
