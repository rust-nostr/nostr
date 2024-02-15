// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip01;
use nostr::nips::nip19::{FromBech32, ToBech32};
use nostr::nips::nip21::NostrURI;
use uniffi::Object;

use crate::error::Result;
use crate::PublicKey;

/// Coordinate for event (`a` tag)
#[derive(Object)]
pub struct Coordinate {
    inner: nip01::Coordinate,
}

#[uniffi::export]
impl Coordinate {
    #[uniffi::constructor]
    pub fn new(kind: u64, public_key: Arc<PublicKey>) -> Self {
        Self {
            inner: nip01::Coordinate::new(kind.into(), **public_key),
        }
    }

    #[uniffi::constructor]
    pub fn parse(coordinate: String) -> Result<Self> {
        Ok(nip01::Coordinate::from_str(&coordinate)?.into())
    }

    #[uniffi::constructor]
    pub fn from_bech32(bech32: String) -> Result<Self> {
        Ok(nip01::Coordinate::from_bech32(bech32)?.into())
    }

    #[uniffi::constructor]
    pub fn from_nostr_uri(uri: String) -> Result<Self> {
        Ok(nip01::Coordinate::from_nostr_uri(uri)?.into())
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }

    pub fn to_nostr_uri(&self) -> Result<String> {
        Ok(self.inner.to_nostr_uri()?)
    }

    pub fn kind(&self) -> u64 {
        self.inner.kind.into()
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.public_key.into())
    }

    pub fn identifier(&self) -> String {
        self.inner.identifier.clone()
    }

    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.clone()
    }
}

impl From<Coordinate> for nip01::Coordinate {
    fn from(value: Coordinate) -> Self {
        Self {
            kind: value.inner.kind,
            public_key: value.inner.public_key,
            identifier: value.inner.identifier,
            relays: value.inner.relays,
        }
    }
}

impl From<nip01::Coordinate> for Coordinate {
    fn from(inner: nip01::Coordinate) -> Self {
        Self { inner }
    }
}
