// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use anyhow::Result;
use nostr_sdk_base::Keys as KeysSdk;
use secp256k1::SecretKey;

#[derive(Clone)]
pub struct Keys {
    keys: KeysSdk,
}

impl Deref for Keys {
    type Target = KeysSdk;
    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

impl Keys {
    pub fn new(sk: String) -> Result<Self> {
        let sk = SecretKey::from_str(&sk)?;

        Ok(Self {
            keys: KeysSdk::new(sk),
        })
    }

    pub fn new_pub_only(pk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::new_pub_only(&pk)?,
        })
    }

    pub fn new_pub_only_from_bech32(pk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::new_pub_only_from_bech32(&pk)?,
        })
    }

    pub fn new_from_bech32(sk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::new_from_bech32(&sk)?,
        })
    }

    pub fn generate_from_os_random() -> Self {
        Self {
            keys: KeysSdk::generate_from_os_random(),
        }
    }

    pub fn public_key(&self) -> String {
        self.keys.public_key_as_str()
    }

    pub fn secret_key(&self) -> Result<String> {
        Ok(self.keys.secret_key_as_str()?)
    }
}
