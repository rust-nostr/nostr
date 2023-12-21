// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use nostr::nips::nip01;
use uniffi::Record;

use crate::{error::Result, PublicKey};

/// Coordinate for event (`a` tag)
#[derive(Record)]
pub struct Coordinate {
    /// Kind
    pub kind: u64,
    /// Public Key
    pub pubkey: Arc<PublicKey>,
    /// `d` tag identifier
    ///
    /// Needed for a parametrized replaceable event.
    /// Leave empty for a replaceable event.
    pub identifier: String,
    /// Relays
    pub relays: Vec<String>,
}

#[uniffi::export]
impl Coordinate {
    #[uniffi::constructor]
    pub fn parse(coordinate: String) -> Result<Self> {
        let coordinate = nip01::Coordinate::from_str(&coordinate);
        Ok(coordinate.map(|c| c.into())?)
    }
}

impl From<Coordinate> for nip01::Coordinate {
    fn from(value: Coordinate) -> Self {
        Self {
            kind: value.kind.into(),
            pubkey: **value.pubkey,
            identifier: value.identifier,
            relays: value.relays,
        }
    }
}

impl From<nip01::Coordinate> for Coordinate {
    fn from(value: nip01::Coordinate) -> Self {
        Self {
            kind: value.kind.into(),
            pubkey: Arc::new(value.pubkey.into()),
            identifier: value.identifier,
            relays: value.relays,
        }
    }
}
