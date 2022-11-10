// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use secp256k1::XOnlyPublicKey;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Contact {
    pub alias: String,
    pub pk: XOnlyPublicKey,
    pub relay_url: String,
}

impl Contact {
    pub fn new(alias: &str, pk: XOnlyPublicKey, relay_url: &str) -> Self {
        Self {
            alias: alias.into(),
            pk,
            relay_url: relay_url.into(),
        }
    }
}
