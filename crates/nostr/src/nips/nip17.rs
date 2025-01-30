// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP17: Private Direct Message
//!
//! <https://github.com/nostr-protocol/nips/blob/master/17.md>

use alloc::boxed::Box;
use core::iter;

use crate::{Event, Kind, RelayUrl, TagStandard};

/// Extracts the relay list
pub fn extract_relay_list<'a>(event: &'a Event) -> Box<dyn Iterator<Item = &'a RelayUrl> + 'a> {
    if event.kind != Kind::InboxRelays {
        return Box::new(iter::empty());
    }

    Box::new(event.tags.iter().filter_map(|tag| {
        if let Some(TagStandard::Relay(url)) = tag.as_standardized() {
            Some(url)
        } else {
            None
        }
    }))
}

/// Extracts the relay list
pub fn extract_owned_relay_list(event: Event) -> Box<dyn Iterator<Item = RelayUrl>> {
    if event.kind != Kind::InboxRelays {
        return Box::new(iter::empty());
    }

    Box::new(event.tags.into_iter().filter_map(|tag| {
        if let Some(TagStandard::Relay(url)) = tag.to_standardized() {
            Some(url)
        } else {
            None
        }
    }))
}
