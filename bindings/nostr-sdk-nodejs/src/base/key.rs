// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use napi::Result;
use nostr_sdk_base::Keys as KeysSdk;
use secp256k1::SecretKey;

use crate::error::into_err;

#[napi]
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

#[napi]
impl Keys {
    #[napi(constructor)]
    pub fn new(sk: String) -> Result<Self> {
        let sk = SecretKey::from_str(&sk).map_err(into_err)?;

        Ok(Self {
            keys: KeysSdk::new(sk),
        })
    }

    #[napi(factory)]
    pub fn new_pub_only(pk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::new_pub_only(&pk).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn new_pub_only_from_bech32(pk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::new_pub_only_from_bech32(&pk).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn new_from_bech32(sk: String) -> Result<Self> {
        Ok(Self {
            keys: KeysSdk::new_from_bech32(&sk).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn generate_from_os_random() -> Self {
        Self {
            keys: KeysSdk::generate_from_os_random(),
        }
    }

    #[napi]
    pub fn public_key(&self) -> String {
        self.keys.public_key_as_str()
    }

    #[napi]
    pub fn secret_key(&self) -> Result<String> {
        self.keys.secret_key_as_str().map_err(into_err)
    }
}
