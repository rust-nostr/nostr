// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use nostr::nips::nip19::{self, FromBech32, ToBech32};
use nostr::nips::nip21::NostrURI;
use nostr::Url;
use uniffi::{Enum, Object};

use super::nip01::Coordinate;
use super::nip49::EncryptedSecretKey;
use crate::error::Result;
use crate::{EventId, Kind, PublicKey, SecretKey};

/// A representation any `NIP19` bech32 nostr object. Useful for decoding
/// `NIP19` bech32 strings without necessarily knowing what you're decoding
/// ahead of time.
#[derive(Enum, o2o::o2o)]
#[from_owned(nip19::Nip19)]
pub enum Nip19Enum {
    /// nsec
    #[o2o(repeat())]
    #[type_hint(as())]
    Secret {
        #[o2o(repeat(permeate()))]
        #[from(Arc::new(~.into()))]
        nsec: Arc<SecretKey>,
    },
    /// Encrypted Secret Key
    EncryptedSecret { ncryptsec: Arc<EncryptedSecretKey> },
    /// npub
    Pubkey { npub: Arc<PublicKey> },
    /// nprofile
    Profile { nprofile: Arc<Nip19Profile> },
    /// note
    #[map(EventId)]
    Note { event_id: Arc<EventId> },
    /// nevent
    Event { event: Arc<Nip19Event> },
    /// naddr
    #[map(Coordinate)]
    Coord { coordinate: Arc<Coordinate> },
    /// nrelay
    Relay { relay: Arc<Nip19Relay> },
}

#[derive(Debug, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
pub struct Nip19 {
    inner: nip19::Nip19,
}

impl From<nip19::Nip19> for Nip19 {
    fn from(inner: nip19::Nip19) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Nip19 {
    #[uniffi::constructor]
    pub fn from_bech32(bech32: &str) -> Result<Self> {
        Ok(nip19::Nip19::from_bech32(bech32)?.into())
    }

    pub fn as_enum(&self) -> Nip19Enum {
        self.inner.clone().into()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Nip19Event {
    inner: nip19::Nip19Event,
}

impl From<nip19::Nip19Event> for Nip19Event {
    fn from(inner: nip19::Nip19Event) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Nip19Event {
    #[uniffi::constructor(default(author = None, kind = None, relays = []))]
    pub fn new(
        event_id: &EventId,
        author: Option<Arc<PublicKey>>,
        kind: Option<Arc<Kind>>,
        relays: &[String],
    ) -> Self {
        let mut inner = nip19::Nip19Event::new(**event_id, relays);
        inner.author = author.map(|p| **p);
        inner.kind = kind.map(|k| **k);
        Self { inner }
    }

    #[uniffi::constructor]
    pub fn from_bech32(bech32: &str) -> Result<Self> {
        Ok(Self {
            inner: nip19::Nip19Event::from_bech32(bech32)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_nostr_uri(uri: &str) -> Result<Self> {
        Ok(Self {
            inner: nip19::Nip19Event::from_nostr_uri(uri)?,
        })
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }

    pub fn to_nostr_uri(&self) -> Result<String> {
        Ok(self.inner.to_nostr_uri()?)
    }

    pub fn event_id(&self) -> Arc<EventId> {
        Arc::new(self.inner.event_id.into())
    }

    pub fn author(&self) -> Option<Arc<PublicKey>> {
        self.inner.author.map(|p| Arc::new(p.into()))
    }

    pub fn kind(&self) -> Option<Arc<Kind>> {
        self.inner.kind.map(|k| Arc::new(k.into()))
    }

    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.clone()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Nip19Profile {
    inner: nip19::Nip19Profile,
}

impl From<nip19::Nip19Profile> for Nip19Profile {
    fn from(inner: nip19::Nip19Profile) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Nip19Profile {
    /// New NIP19 profile
    #[uniffi::constructor(default(relays = []))]
    pub fn new(public_key: &PublicKey, relays: &[String]) -> Result<Self> {
        Ok(Self {
            inner: nip19::Nip19Profile::new(**public_key, relays)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_bech32(bech32: &str) -> Result<Self> {
        Ok(Self {
            inner: nip19::Nip19Profile::from_bech32(bech32)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_nostr_uri(uri: &str) -> Result<Self> {
        Ok(Self {
            inner: nip19::Nip19Profile::from_nostr_uri(uri)?,
        })
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
        self.inner.relays.iter().map(|u| u.to_string()).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Nip19Relay {
    inner: nip19::Nip19Relay,
}

impl From<nip19::Nip19Relay> for Nip19Relay {
    fn from(inner: nip19::Nip19Relay) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Nip19Relay {
    #[uniffi::constructor]
    pub fn new(url: &str) -> Result<Self> {
        let url: Url = Url::parse(url)?;
        Ok(Self {
            inner: nip19::Nip19Relay::new(url),
        })
    }

    #[uniffi::constructor]
    pub fn from_bech32(bech32: &str) -> Result<Self> {
        Ok(Self {
            inner: nip19::Nip19Relay::from_bech32(bech32)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_nostr_uri(uri: &str) -> Result<Self> {
        Ok(Self {
            inner: nip19::Nip19Relay::from_nostr_uri(uri)?,
        })
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }

    pub fn to_nostr_uri(&self) -> Result<String> {
        Ok(self.inner.to_nostr_uri()?)
    }

    pub fn url(&self) -> String {
        self.inner.url.to_string()
    }
}
