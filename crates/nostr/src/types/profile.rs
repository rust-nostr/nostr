// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Profile

use bitcoin::secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

/// Profile
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Profile {
    /// Public key
    pub public_key: XOnlyPublicKey,
    /// Relays
    pub relays: Vec<String>,
}

impl Profile {
    /// New [`Profile`]
    pub fn new<S>(public_key: XOnlyPublicKey, relays: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            public_key,
            relays: relays.into_iter().map(|u| u.into()).collect(),
        }
    }
}
