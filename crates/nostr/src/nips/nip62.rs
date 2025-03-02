// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-62: Request to Vanish
//!
//! <https://github.com/nostr-protocol/nips/blob/master/62.md>

use alloc::vec::Vec;

use crate::RelayUrl;

/// Request to Vanish target
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VanishTarget {
    /// Request to vanish from all relays
    AllRelays,
    /// Request to vanish from a specific list of relays.
    Relays(Vec<RelayUrl>),
}

impl VanishTarget {
    /// Vanish from a single relay
    #[inline]
    pub fn relay(relay: RelayUrl) -> Self {
        Self::Relays(vec![relay])
    }

    /// Vanish from multiple relays
    #[inline]
    pub fn relays<I>(relays: I) -> Self
    where
        I: IntoIterator<Item = RelayUrl>,
    {
        Self::Relays(relays.into_iter().collect())
    }

    /// Vanish from all relays
    pub fn all_relays() -> Self {
        Self::AllRelays
    }
}
