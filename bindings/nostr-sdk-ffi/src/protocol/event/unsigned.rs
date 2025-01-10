// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::secp256k1::schnorr::Signature;
use nostr::JsonUtil;
use uniffi::Object;

use super::EventId;
use crate::error::Result;
use crate::protocol::event::{Event, Kind, Tags, Timestamp};
use crate::protocol::key::{Keys, PublicKey};
use crate::protocol::signer::NostrSigner;

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct UnsignedEvent {
    inner: nostr::UnsignedEvent,
}

impl Deref for UnsignedEvent {
    type Target = nostr::UnsignedEvent;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr::UnsignedEvent> for UnsignedEvent {
    fn from(inner: nostr::UnsignedEvent) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl UnsignedEvent {
    pub fn id(&self) -> Option<Arc<EventId>> {
        self.inner.id.map(|id| Arc::new(id.into()))
    }

    pub fn author(&self) -> PublicKey {
        self.inner.pubkey.into()
    }

    pub fn created_at(&self) -> Timestamp {
        self.inner.created_at.into()
    }

    pub fn kind(&self) -> Kind {
        self.inner.kind.into()
    }

    pub fn tags(&self) -> Tags {
        self.inner.tags.clone().into()
    }

    pub fn content(&self) -> String {
        self.inner.content.clone()
    }

    /// Sign an unsigned event
    pub async fn sign(&self, signer: &NostrSigner) -> Result<Event> {
        Ok(self.inner.clone().sign(signer.deref()).await?.into())
    }

    /// Add signature to unsigned event
    ///
    /// Internally verify the event.
    pub fn add_signature(&self, sig: &str) -> Result<Event> {
        let sig = Signature::from_str(sig)?;
        Ok(Event::from(self.inner.clone().add_signature(sig)?))
    }

    #[uniffi::constructor]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::UnsignedEvent::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }

    pub fn as_pretty_json(&self) -> Result<String> {
        Ok(self.inner.try_as_pretty_json()?)
    }
}
