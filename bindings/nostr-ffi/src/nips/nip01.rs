// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip01;
use nostr::nips::nip19::{FromBech32, ToBech32};
use nostr::nips::nip21::NostrURI;
use uniffi::Object;

use crate::error::Result;
use crate::{Kind, PublicKey};

/// Coordinate for event (`a` tag)
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Coordinate {
    inner: nip01::Coordinate,
}

impl Deref for Coordinate {
    type Target = nip01::Coordinate;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nip01::Coordinate> for Coordinate {
    fn from(inner: nip01::Coordinate) -> Self {
        Self { inner }
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

#[uniffi::export]
impl Coordinate {
    #[uniffi::constructor]
    pub fn new(kind: &Kind, public_key: &PublicKey) -> Self {
        Self {
            inner: nip01::Coordinate::new(**kind, public_key.deref().clone()),
        }
    }

    #[uniffi::constructor]
    pub fn parse(coordinate: String) -> Result<Self> {
        Ok(nip01::Coordinate::from_str(&coordinate)?.into())
    }

    #[uniffi::constructor]
    pub fn from_bech32(bech32: &str) -> Result<Self> {
        Ok(nip01::Coordinate::from_bech32(bech32)?.into())
    }

    #[uniffi::constructor]
    pub fn from_nostr_uri(uri: &str) -> Result<Self> {
        Ok(nip01::Coordinate::from_nostr_uri(uri)?.into())
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }

    pub fn to_nostr_uri(&self) -> Result<String> {
        Ok(self.inner.to_nostr_uri()?)
    }

    pub fn kind(&self) -> Kind {
        self.inner.kind.into()
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.public_key.clone().into())
    }

    pub fn identifier(&self) -> String {
        self.inner.identifier.clone()
    }

    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.clone()
    }
}
