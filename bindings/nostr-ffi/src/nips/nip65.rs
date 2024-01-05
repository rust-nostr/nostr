// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip65;

use crate::{Event, RelayMetadata};

/// Extracts the relay info (url, optional read/write flag) from the event
#[uniffi::export]
pub fn extract_relay_list(event: Arc<Event>) -> HashMap<String, Option<RelayMetadata>> {
    nip65::extract_relay_list(event.deref())
        .into_iter()
        .map(|(s, r)| (s.to_string(), r.map(|r| r.into())))
        .collect()
}
