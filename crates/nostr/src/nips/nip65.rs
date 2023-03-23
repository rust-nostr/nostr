//! NIP65
//!
//! <https://github.com/nostr-protocol/nips/blob/master/65.md>

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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
