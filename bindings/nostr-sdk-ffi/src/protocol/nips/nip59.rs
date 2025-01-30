// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip59;
use uniffi::Object;

use crate::error::Result;
use crate::protocol::event::{Event, Tag, UnsignedEvent};
use crate::protocol::key::PublicKey;
use crate::protocol::signer::NostrSigner;

/// Build Gift Wrap
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[uniffi::export(async_runtime = "tokio", default(extra_tags = []))]
pub async fn gift_wrap(
    signer: &NostrSigner,
    receiver_pubkey: &PublicKey,
    rumor: &UnsignedEvent,
    extra_tags: Vec<Arc<Tag>>,
) -> Result<Event> {
    Ok(nostr::EventBuilder::gift_wrap(
        signer.deref(),
        receiver_pubkey.deref(),
        rumor.deref().clone(),
        extra_tags.into_iter().map(|t| t.as_ref().deref().clone()),
    )
    .await?
    .into())
}

/// Build Gift Wrap from Seal
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[uniffi::export(default(extra_tags = []))]
pub fn gift_wrap_from_seal(
    receiver: &PublicKey,
    seal: &Event,
    extra_tags: Vec<Arc<Tag>>,
) -> Result<Event> {
    Ok(nostr::EventBuilder::gift_wrap_from_seal(
        receiver.deref(),
        seal.deref(),
        extra_tags.into_iter().map(|t| t.as_ref().deref().clone()),
    )?
    .into())
}

/// Unwrapped Gift Wrap
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct UnwrappedGift {
    inner: nip59::UnwrappedGift,
}

impl From<nip59::UnwrappedGift> for UnwrappedGift {
    fn from(inner: nip59::UnwrappedGift) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl UnwrappedGift {
    // `#[uniffi::export(async_runtime = "tokio")]` require an async method
    async fn _none(&self) {}

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    #[uniffi::constructor]
    pub async fn from_gift_wrap(signer: &NostrSigner, gift_wrap: &Event) -> Result<Self> {
        Ok(Self {
            inner: nip59::UnwrappedGift::from_gift_wrap(signer.deref(), gift_wrap.deref()).await?,
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
