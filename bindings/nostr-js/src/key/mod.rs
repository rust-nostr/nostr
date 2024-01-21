// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

mod public_key;
mod secret_key;

pub use self::public_key::JsPublicKey;
pub use self::secret_key::JsSecretKey;
use crate::error::{into_err, Result};

#[wasm_bindgen(js_name = Keys)]
pub struct JsKeys {
    inner: Keys,
}

impl Deref for JsKeys {
    type Target = Keys;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Keys> for JsKeys {
    fn from(keys: Keys) -> Self {
        Self { inner: keys }
    }
}

#[wasm_bindgen(js_class = Keys)]
impl JsKeys {
    /// Initialize from secret key.
    #[wasm_bindgen(constructor)]
    pub fn new(secret_key: &JsSecretKey) -> JsKeys {
        Self {
            inner: Keys::new(secret_key.into()),
        }
    }

    /// Initialize with public key only (no secret key).
    #[wasm_bindgen(js_name = fromPublicKey)]
    pub fn from_public_key(public_key: &JsPublicKey) -> JsKeys {
        Self {
            inner: Keys::from_public_key(public_key.into()),
        }
    }

    /// Init [`Keys`] from `hex` or `bech32` secret key string
    #[wasm_bindgen(js_name = fromSkStr)]
    pub fn from_sk_str(secret_key: &str) -> Result<JsKeys> {
        Ok(Self {
            inner: Keys::from_sk_str(secret_key).map_err(into_err)?,
        })
    }

    /// Init [`Keys`] from `hex` or `bech32` public key string
    #[wasm_bindgen(js_name = fromPkStr)]
    pub fn from_pk_str(public_key: &str) -> Result<JsKeys> {
        Ok(Self {
            inner: Keys::from_pk_str(public_key).map_err(into_err)?,
        })
    }

    /// Generate new random keys
    #[wasm_bindgen]
    pub fn generate() -> JsKeys {
        Self {
            inner: Keys::generate(),
        }
    }

    /// Derive keys from BIP-39 mnemonics (ENGLISH wordlist).
    #[wasm_bindgen(js_name = fromMnemonic)]
    pub fn from_mnemonic(mnemonic: &str, passphrase: Option<String>) -> Result<JsKeys> {
        Ok(Self {
            inner: Keys::from_mnemonic(mnemonic, passphrase.as_deref()).map_err(into_err)?,
        })
    }

    /// Get public key
    #[wasm_bindgen(js_name = publicKey, getter)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key().into()
    }

    /// Get secret key
    #[wasm_bindgen(js_name = secretKey, getter)]
    pub fn secret_key(&self) -> Result<JsSecretKey> {
        Ok(self.inner.secret_key().map_err(into_err)?.into())
    }
}
