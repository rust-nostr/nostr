// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP65
//!
//! <https://github.com/nostr-protocol/nips/blob/master/65.md>

use alloc::vec::Vec;

use crate::{Event, RelayMetadata, Tag, UncheckedUrl};

/// Extracts the relay info (url, optional read/write flag) from the event
#[inline]
pub fn extract_relay_list(event: &Event) -> Vec<(UncheckedUrl, Option<RelayMetadata>)> {
    event
        .iter_tags()
        .filter_map(|tag| {
            if let Tag::RelayMetadata(url, metadata) = tag {
                Some((url.clone(), metadata.clone()))
            } else {
                None
            }
        })
        .collect()
}
