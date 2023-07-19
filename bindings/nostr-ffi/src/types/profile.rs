// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip19::{FromBech32, ToBech32};

use crate::error::Result;
use crate::PublicKey;

pub struct Profile {
    inner: nostr::Profile,
}

impl Profile {
    /// New [`Profile`]
    pub fn new(public_key: Arc<PublicKey>, relays: Vec<String>) -> Self {
        Self {
            inner: nostr::Profile::new(*public_key.as_ref().deref(), relays),
        }
    }

    pub fn from_bech32(bech32: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::Profile::from_bech32(bech32)?,
        })
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.public_key.into())
    }

    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.clone()
    }
}
