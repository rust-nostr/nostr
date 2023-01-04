// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub pubkey: XOnlyPublicKey,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub about: Option<String>,
    pub website: Option<String>,
    pub picture: Option<String>,
    pub nip05: Option<String>,
    pub lud06: Option<String>,
    pub lud16: Option<String>,
    pub followed: bool,
    pub metadata_at: u64,
}

impl Profile {
    pub fn new(pubkey: XOnlyPublicKey) -> Self {
        Self {
            pubkey,
            name: None,
            display_name: None,
            about: None,
            website: None,
            picture: None,
            nip05: None,
            lud06: None,
            lud16: None,
            followed: false,
            metadata_at: 0,
        }
    }
}
