// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Contact

use bitcoin::secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

/// Contact
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct Contact {
    /// Public key
    pub pk: XOnlyPublicKey,
    /// Relay url
    pub relay_url: Option<String>,
    /// Alias
    pub alias: Option<String>,
}

impl Contact {
    /// Create new [`Contact`]
    pub fn new<S>(pk: XOnlyPublicKey, relay_url: Option<S>, alias: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            pk,
            relay_url: relay_url.map(|a| a.into()),
            alias: alias.map(|a| a.into()),
        }
    }
}
