// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use nostr::nips::nip21;
use uniffi::{Enum, Object};

use crate::error::Result;
use crate::protocol::event::EventId;
use crate::protocol::key::PublicKey;
use crate::protocol::nips::nip19::{Nip19Coordinate, Nip19Event, Nip19Profile};

/// A representation any `NIP21` object. Useful for decoding
/// `NIP21` strings without necessarily knowing what you're decoding
/// ahead of time.
#[derive(Enum)]
pub enum Nip21Enum {
    /// nostr::npub
    Pubkey { public_key: Arc<PublicKey> },
    /// nostr::nprofile
    Profile { profile: Arc<Nip19Profile> },
    /// nostr::note (EventId)
    Note { event_id: Arc<EventId> },
    /// nostr::nevent
    Event { event: Arc<Nip19Event> },
    /// nostr::naddr
    Addr { coordinate: Arc<Nip19Coordinate> },
}

impl From<nip21::Nip21> for Nip21Enum {
    fn from(value: nip21::Nip21) -> Self {
        match value {
            nip21::Nip21::Pubkey(public_key) => Self::Pubkey {
                public_key: Arc::new(public_key.into()),
            },
            nip21::Nip21::Profile(profile) => Self::Profile {
                profile: Arc::new(profile.into()),
            },
            nip21::Nip21::EventId(event_id) => Self::Note {
                event_id: Arc::new(event_id.into()),
            },
            nip21::Nip21::Event(event) => Self::Event {
                event: Arc::new(event.into()),
            },
            nip21::Nip21::Coordinate(coordinate) => Self::Addr {
                coordinate: Arc::new(coordinate.into()),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
pub struct Nip21 {
    inner: nip21::Nip21,
}

impl From<nip21::Nip21> for Nip21 {
    fn from(inner: nip21::Nip21) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Nip21 {
    /// Parse NIP21 string
    #[uniffi::constructor]
    pub fn parse(uri: &str) -> Result<Self> {
        Ok(Self {
            inner: nip21::Nip21::parse(uri)?,
        })
    }

    pub fn as_enum(&self) -> Nip21Enum {
        self.inner.clone().into()
    }

    /// Serialize to NIP21 nostr URI
    pub fn to_nostr_uri(&self) -> Result<String> {
        Ok(self.inner.to_nostr_uri()?)
    }
}

/// Extract `nostr:` URIs from a text
#[uniffi::export]
pub fn nip21_extract_from_text(text: &str) -> Vec<Arc<Nip21>> {
    nip21::extract_from_text(text)
        .into_iter()
        .map(|i| Arc::new(i.into()))
        .collect()
}
