// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;

use napi::Result;
use nostr_sdk::nostr::Keys as KeysSdk;
use nostr_sdk::prelude::*;

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
  pub fn from_public_key(pk: String) -> Result<Self> {
    let pk = XOnlyPublicKey::from_str(&pk).map_err(into_err)?;
    Ok(Self {
      keys: KeysSdk::from_public_key(pk),
    })
  }

  #[napi(factory)]
  pub fn from_bech32(sk: String) -> Result<Self> {
    let sk = SecretKey::from_bech32(sk).map_err(into_err)?;
    Ok(Self {
      keys: KeysSdk::new(sk),
    })
  }

  #[napi(factory)]
  pub fn from_bech32_public_key(pk: String) -> Result<Self> {
    let pk = XOnlyPublicKey::from_bech32(pk).map_err(into_err)?;
    Ok(Self {
      keys: KeysSdk::from_public_key(pk),
    })
  }

  #[napi(factory)]
  pub fn generate() -> Self {
    Self {
      keys: KeysSdk::generate(),
    }
  }

  #[napi]
  pub fn public_key(&self) -> String {
    self.keys.public_key().to_string()
  }

  #[napi]
  pub fn secret_key(&self) -> Result<String> {
    Ok(
      self
        .keys
        .secret_key()
        .map_err(into_err)?
        .display_secret()
        .to_string(),
    )
  }
}
