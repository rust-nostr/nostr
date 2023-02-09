// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use napi::Result;
use nostr::prelude::*;

mod public_key;
mod secret_key;

pub use self::public_key::JsPublicKey;
pub use self::secret_key::JsSecretKey;
use crate::error::into_err;

#[napi(js_name = "Keys")]
pub struct JsKeys {
    inner: Keys,
}

impl Deref for JsKeys {
    type Target = Keys;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[napi]
impl JsKeys {
    #[napi(constructor)]
    pub fn new(secret_key: &JsSecretKey) -> Self {
        Self {
            inner: Keys::new(secret_key.into()),
        }
    }

    #[napi(factory)]
    pub fn from_public_key(public_key: &JsPublicKey) -> Self {
        Self {
            inner: Keys::from_public_key(public_key.into()),
        }
    }

    #[napi(factory)]
    pub fn from_sk_str(secret_key: String) -> Result<Self> {
        Ok(Self {
            inner: Keys::from_sk_str(&secret_key).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn from_pk_str(public_key: String) -> Result<Self> {
        Ok(Self {
            inner: Keys::from_pk_str(&public_key).map_err(into_err)?,
        })
    }

    #[napi(factory)]
    pub fn generate() -> Self {
        Self {
            inner: Keys::generate(),
        }
    }

    #[napi(factory)]
    pub fn from_mnemonic(mnemonic: String, passphrase: Option<String>) -> Result<Self> {
        Ok(Self {
            inner: Keys::from_mnemonic(mnemonic, passphrase).map_err(into_err)?,
        })
    }

    #[napi]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key().into()
    }

    #[napi]
    pub fn secret_key(&self) -> Result<JsSecretKey> {
        Ok(self.inner.secret_key().map_err(into_err)?.into())
    }
}
