// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::Arc;

use nostr::nips::nip19::{FromBech32, ToBech32};
use nostr::nips::nip21::NostrURI;
use uniffi::Object;

use crate::error::Result;
use crate::PublicKey;

#[derive(Object)]
pub struct Profile {
    inner: nostr::Profile,
}

impl From<nostr::Profile> for Profile {
    fn from(inner: nostr::Profile) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Profile {
    /// New [`Profile`]
    #[uniffi::constructor]
    pub fn new(public_key: Arc<PublicKey>, relays: Vec<String>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::Profile::new(**public_key, relays),
        })
    }

    #[uniffi::constructor]
    pub fn from_bech32(bech32: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::Profile::from_bech32(bech32)?,
        }))
    }

    #[uniffi::constructor]
    pub fn from_nostr_uri(uri: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::Profile::from_nostr_uri(uri)?,
        }))
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }

    pub fn to_nostr_uri(&self) -> Result<String> {
        Ok(self.inner.to_nostr_uri()?)
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.public_key.into())
    }

    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.clone()
    }
}
