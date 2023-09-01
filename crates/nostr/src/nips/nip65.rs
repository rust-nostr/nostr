// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP65
//!
//! <https://github.com/nostr-protocol/nips/blob/master/65.md>

use alloc::vec::Vec;

use crate::{Event, RelayMetadata, Tag, UncheckedUrl};

/// Extracts the relay info (url, optional read/write flag) from the event
pub fn extract_relay_list(event: &Event) -> Vec<(UncheckedUrl, Option<RelayMetadata>)> {
    let mut list = Vec::new();
    for tag in event.tags.iter() {
        if let Tag::RelayMetadata(url, metadata) = tag {
            list.push((url.clone(), metadata.clone()))
        }
    }
    list
}
