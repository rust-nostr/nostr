// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use napi::Result;
use nostr::prelude::*;

use crate::error::into_err;

#[napi(js_name = "Keys")]
pub struct JsKeys {
    keys: Keys,
}

impl Deref for JsKeys {
    type Target = Keys;
    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

#[napi]
impl JsKeys {
    #[napi(factory)]
    pub fn from_secret_key(secret_key: String) -> Result<Self> {
        Ok(Self {
            keys: Keys::from_sk_str(&secret_key).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn from_public_key(public_key: String) -> Result<Self> {
        Ok(Self {
            keys: Keys::from_pk_str(&public_key).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn generate() -> Self {
        Self {
            keys: Keys::generate(),
        }
    }

    #[napi(factory)]
    pub fn from_mnemonic(mnemonic: String, passphrase: Option<String>) -> Result<Self> {
        Ok(Self {
            keys: Keys::from_mnemonic(mnemonic, passphrase).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn public_key(&self) -> String {
        self.keys.public_key().to_string()
    }

    #[napi]
    pub fn public_key_bech32(&self) -> Result<String> {
        self.keys.public_key().to_bech32().map_err(into_err)
    }

    #[napi]
    pub fn secret_key(&self) -> Result<String> {
        Ok(self
            .keys
            .secret_key()
            .map_err(into_err)?
            .display_secret()
            .to_string())
    }

    #[napi]
    pub fn secret_key_bech32(&self) -> Result<String> {
        self.keys
            .secret_key()
            .map_err(into_err)?
            .to_bech32()
            .map_err(into_err)
    }
}
