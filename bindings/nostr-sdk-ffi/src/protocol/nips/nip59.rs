// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip59;
use nostr::EventBuilder;
use uniffi::Object;

use crate::error::Result;
use crate::protocol::signer::{NostrSigner, NostrSignerFFI2Rust};
use crate::protocol::{Event, PublicKey, Tag, UnsignedEvent};

/// Build Gift Wrap
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[uniffi::export(async_runtime = "tokio", default(extra_tags = vec![]))]
pub async fn gift_wrap(
    signer: Arc<dyn NostrSigner>,
    receiver_pubkey: &PublicKey,
    rumor: &UnsignedEvent,
    extra_tags: Vec<Tag>,
) -> Result<Event> {
    let signer = NostrSignerFFI2Rust::new(signer);
    Ok(EventBuilder::gift_wrap(
        &signer,
        receiver_pubkey.deref(),
        rumor.deref().clone(),
        extra_tags,
    )
    .await?
    .into())
}

/// Build Gift Wrap from Seal
///
/// <https://github.com/nostr-protocol/nips/blob/master/59.md>
#[uniffi::export(async_runtime = "tokio", default(extra_tags = vec![]))]
pub fn gift_wrap_from_seal(
    receiver: &PublicKey,
    seal: &Event,
    extra_tags: Vec<Tag>,
) -> Result<Event> {
    Ok(EventBuilder::gift_wrap_from_seal(receiver.deref(), seal.deref(), extra_tags)?.into())
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
    pub async fn from_gift_wrap(signer: Arc<dyn NostrSigner>, gift_wrap: &Event) -> Result<Self> {
        let signer = NostrSignerFFI2Rust::new(signer);
        Ok(Self {
            inner: nip59::UnwrappedGift::from_gift_wrap(&signer, gift_wrap.deref()).await?,
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
