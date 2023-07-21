// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP65
//!
//! <https://github.com/nostr-protocol/nips/blob/master/65.md>

use crate::Event;

/// Extracts the relay info (url, optional read/write flag) from the event
pub fn get_relay_list(event: Event) -> Vec<(String, Option<String>)> {
    event
        .tags
        .iter()
        .filter(|t| t.as_vec()[0] == "r")
        .map(|t| (t.as_vec()[1].clone(), t.as_vec().get(2).cloned()))
        .collect()
}
