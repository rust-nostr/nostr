// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP65
//!
//! <https://github.com/nostr-protocol/nips/blob/master/65.md>

use alloc::vec::Vec;

use crate::{Event, RelayMetadata, TagStandard, Url};

/// Extracts the relay info (url, optional read/write flag) from the event
#[inline]
pub fn extract_relay_list(event: &Event) -> Vec<(&Url, &Option<RelayMetadata>)> {
    event
        .iter_tags()
        .filter_map(|tag| {
            if let Some(TagStandard::RelayMetadata {
                relay_url,
                metadata,
            }) = tag.as_standardized()
            {
                Some((relay_url, metadata))
            } else {
                None
            }
        })
        .collect()
}
