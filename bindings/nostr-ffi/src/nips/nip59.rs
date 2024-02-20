// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip59;
use nostr::EventBuilder;

use crate::error::Result;
use crate::{Event, Keys, PublicKey, UnsignedEvent};

#[uniffi::export]
pub fn gift_wrap(
    sender_keys: Arc<Keys>,
    receiver_pubkey: Arc<PublicKey>,
    rumor: Arc<UnsignedEvent>,
) -> Result<Arc<Event>> {
    Ok(Arc::new(
        EventBuilder::gift_wrap(
            sender_keys.as_ref().deref(),
            receiver_pubkey.as_ref().deref(),
            rumor.as_ref().deref().clone(),
        )?
        .into(),
    ))
}

/// Extract `rumor` from `gift wrap`
#[uniffi::export]
pub fn nip59_extract_rumor(keys: Arc<Keys>, gift_wrap: Arc<Event>) -> Result<Arc<UnsignedEvent>> {
    Ok(Arc::new(
        nip59::extract_rumor(keys.as_ref().deref(), gift_wrap.as_ref().deref())?
            .rumor
            .into(),
    ))
}
