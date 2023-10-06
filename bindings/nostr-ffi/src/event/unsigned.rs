// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::secp256k1::schnorr::Signature;
use nostr::JsonUtil;

use super::EventId;
use crate::error::Result;
use crate::{Event, Keys, PublicKey, Tag, Timestamp};

pub struct UnsignedEvent {
    inner: nostr::UnsignedEvent,
}

impl From<nostr::UnsignedEvent> for UnsignedEvent {
    fn from(inner: nostr::UnsignedEvent) -> Self {
        Self { inner }
    }
}

impl UnsignedEvent {
    pub fn id(&self) -> Arc<EventId> {
        Arc::new(self.inner.id.into())
    }

    pub fn pubkey(&self) -> Arc<PublicKey> {
        Arc::new(self.inner.pubkey.into())
    }

    pub fn created_at(&self) -> Arc<Timestamp> {
        Arc::new(self.inner.created_at.into())
    }

    pub fn kind(&self) -> u64 {
        self.inner.kind.into()
    }

    pub fn tags(&self) -> Vec<Arc<Tag>> {
        self.inner
            .tags
            .clone()
            .into_iter()
            .map(|t| Arc::new(t.into()))
            .collect()
    }

    pub fn content(&self) -> String {
        self.inner.content.clone()
    }

    pub fn sign(&self, keys: Arc<Keys>) -> Result<Arc<Event>> {
        Ok(Arc::new(Event::from(
            self.inner.clone().sign(keys.as_ref().deref())?,
        )))
    }

    /// Add signature to [`UnsignedEvent`]
    pub fn add_signature(&self, sig: String) -> Result<Arc<Event>> {
        let sig = Signature::from_str(&sig)?;
        Ok(Arc::new(Event::from(
            self.inner.clone().add_signature(sig)?,
        )))
    }

    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::UnsignedEvent::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }
}
