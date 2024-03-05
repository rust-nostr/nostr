// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip59;
use nostr::EventBuilder;

use crate::error::Result;
use crate::{Event, Keys, PublicKey, Timestamp, UnsignedEvent};

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

/// Extract `rumor` from `gift wrap`
#[uniffi::export]
pub fn nip59_extract_rumor(keys: &Keys, gift_wrap: &Event) -> Result<UnsignedEvent> {
    Ok(nip59::extract_rumor(keys.deref(), gift_wrap.deref())?
        .rumor
        .into())
}
