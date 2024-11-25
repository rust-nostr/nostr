// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Contact

use alloc::string::String;

use crate::key::PublicKey;
use crate::types::RelayUrl;

/// Contact
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Contact {
    /// Public key
    pub public_key: PublicKey,
    /// Relay url
    pub relay_url: Option<RelayUrl>,
    /// Alias
    pub alias: Option<String>,
}

impl Contact {
    /// Create new [`Contact`]
    #[inline]
    pub fn new<S>(public_key: PublicKey, relay_url: Option<RelayUrl>, alias: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            public_key,
            relay_url,
            alias: alias.map(|a| a.into()),
        }
    }
}
