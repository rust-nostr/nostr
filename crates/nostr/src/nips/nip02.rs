// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP02: Follow List
//!
//! <https://github.com/nostr-protocol/nips/blob/master/02.md>

use alloc::string::String;

use crate::key::PublicKey;
use crate::types::RelayUrl;

/// Contact
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Contact {
    /// Public key
    pub public_key: PublicKey,
    /// Relay url
    pub relay_url: Option<RelayUrl>,
    /// Alias
    pub alias: Option<String>,
}

impl Contact {
    /// Create new contact
    #[inline]
    pub fn new(public_key: PublicKey) -> Self {
        Self {
            public_key,
            relay_url: None,
            alias: None,
        }
    }
}
