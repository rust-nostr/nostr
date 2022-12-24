// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::secp256k1::XOnlyPublicKey;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct Contact {
    pub pk: XOnlyPublicKey,
    pub relay_url: String,
    pub alias: String,
}

impl Contact {
    pub fn new<S>(pk: XOnlyPublicKey, relay_url: S, alias: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            pk,
            relay_url: relay_url.into(),
            alias: alias.into(),
        }
    }
}
