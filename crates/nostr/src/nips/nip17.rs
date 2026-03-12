// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP17: Private Direct Message
//!
//! <https://github.com/nostr-protocol/nips/blob/master/17.md>

use crate::{Event, RelayUrl, TagStandard};

/// Extracts the relay list
///
/// This function doesn't verify if the event kind is [`Kind::InboxRelays`](crate::Kind::InboxRelays)!
pub fn extract_relay_list(event: &Event) -> impl Iterator<Item = RelayUrl> + '_ {
    event.tags.iter().filter_map(|tag| {
        if let Some(TagStandard::Relay(url)) = tag.standardized() {
            Some(url)
        } else {
            None
        }
    })
}
