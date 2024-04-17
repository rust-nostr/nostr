// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip59;
use nostr::EventBuilder;
use uniffi::Object;

use crate::error::Result;
use crate::{Event, Keys, PublicKey, Timestamp, UnsignedEvent};

/// Build Gift Wrap
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[uniffi::export]
pub fn gift_wrap(
    sender_keys: &Keys,
    receiver_pubkey: &PublicKey,
    rumor: &UnsignedEvent,
    expiration: Option<Arc<Timestamp>>,
) -> Result<Event> {
    Ok(EventBuilder::gift_wrap(
        sender_keys.deref(),
        receiver_pubkey.deref(),
        rumor.deref().clone(),
        expiration.map(|t| **t),
    )?
    .into())
}

/// Build Gift Wrap from Seal
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[uniffi::export]
pub fn gift_wrap_from_seal(
    receiver: &PublicKey,
    seal: &Event,
    expiration: Option<Arc<Timestamp>>,
) -> Result<Event> {
    Ok(
        EventBuilder::gift_wrap_from_seal(receiver.deref(), seal.deref(), expiration.map(|t| **t))?
            .into(),
    )
}

/// Unwrapped Gift Wrap
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct UnwrappedGift {
    inner: nip59::UnwrappedGift,
}

#[uniffi::export]
impl UnwrappedGift {
    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    #[uniffi::constructor]
    pub fn from_gift_wrap(receiver_keys: &Keys, gift_wrap: &Event) -> Result<Self> {
        Ok(Self {
            inner: nip59::UnwrappedGift::from_gift_wrap(receiver_keys.deref(), gift_wrap.deref())?,
        })
    }

    /// Get sender public key
    pub fn sender(&self) -> PublicKey {
        self.inner.sender.into()
    }

    /// Get rumor
    pub fn rumor(&self) -> UnsignedEvent {
        self.inner.rumor.clone().into()
    }
}
