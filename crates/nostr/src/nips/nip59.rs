// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP59: Gift Wrap
//!
//! <https://github.com/nostr-protocol/nips/blob/master/59.md>

use alloc::string::String;
use core::fmt;
use core::ops::Range;

use bitcoin::secp256k1::{Secp256k1, Verification};

use super::nip44;
use crate::event::unsigned::{self, UnsignedEvent};
use crate::event::{self, Event};
use crate::key::{self, Keys, SecretKey};
#[cfg(feature = "std")]
use crate::SECP256K1;
use crate::{JsonUtil, Kind, PublicKey};

/// Range for random timestamp tweak (up to 2 days)
pub const RANGE_RANDOM_TIMESTAMP_TWEAK: Range<u64> = 0..172800; // From 0 secs to 2 days

/// NIP59 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Key error
    Key(key::Error),
    /// Event error
    Event(event::Error),
    /// Unsigned event error
    Unsigned(unsigned::Error),
    /// NIP44 error
    NIP44(nip44::Error),
    /// Not Gift Wrap event
    NotGiftWrap,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "Key: {e}"),
            Self::Event(e) => write!(f, "Event: {e}"),
            Self::Unsigned(e) => write!(f, "Unsigned event: {e}"),
            Self::NIP44(e) => write!(f, "NIP44: {e}"),
            Self::NotGiftWrap => write!(f, "Not Gift Wrap event"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
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

impl From<nip44::Error> for Error {
    fn from(e: nip44::Error) -> Self {
        Self::NIP44(e)
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
    pub fn from_gift_wrap(receiver_keys: &Keys, gift_wrap: &Event) -> Result<Self, Error> {
        Self::from_gift_wrap_with_ctx(&SECP256K1, receiver_keys, gift_wrap)
    }

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    pub fn from_gift_wrap_with_ctx<C>(
        secp: &Secp256k1<C>,
        receiver_keys: &Keys,
        gift_wrap: &Event,
    ) -> Result<Self, Error>
    where
        C: Verification,
    {
        // Check event kind
        if gift_wrap.kind != Kind::GiftWrap {
            return Err(Error::NotGiftWrap);
        }

        let secret_key: &SecretKey = receiver_keys.secret_key();

        // Decrypt and verify seal
        let seal: String = nip44::decrypt(secret_key, &gift_wrap.pubkey, &gift_wrap.content)?;
        let seal: Event = Event::from_json(seal)?;
        seal.verify_with_ctx(secp)?;

        // Decrypt rumor
        let rumor: String = nip44::decrypt(secret_key, &seal.pubkey, &seal.content)?;

        Ok(UnwrappedGift {
            sender: seal.pubkey,
            rumor: UnsignedEvent::from_json(rumor)?,
        })
    }
}

/// Extract `rumor` from Gift Wrap event
#[inline]
#[cfg(feature = "std")]
pub fn extract_rumor(receiver_keys: &Keys, gift_wrap: &Event) -> Result<UnwrappedGift, Error> {
    UnwrappedGift::from_gift_wrap(receiver_keys, gift_wrap)
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::EventBuilder;

    #[test]
    fn test_extract_rumor() {
        let sender_keys = Keys::new(
            SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap(),
        );
        let receiver_keys = Keys::new(
            SecretKey::from_str("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap(),
        );

        // Compose Gift Wrap event
        let rumor: UnsignedEvent =
            EventBuilder::text_note("Test", []).to_unsigned_event(sender_keys.public_key());
        let event: Event = EventBuilder::gift_wrap(
            &sender_keys,
            &receiver_keys.public_key(),
            rumor.clone(),
            None,
        )
        .unwrap();
        assert_eq!(
            extract_rumor(&receiver_keys, &event).unwrap(),
            UnwrappedGift {
                sender: sender_keys.public_key(),
                rumor,
            }
        );
        assert!(extract_rumor(&sender_keys, &event).is_err());

        let event: Event = EventBuilder::text_note("", [])
            .to_event(&sender_keys)
            .unwrap();
        assert_eq!(
            extract_rumor(&receiver_keys, &event).unwrap_err(),
            Error::NotGiftWrap
        );
    }
}
