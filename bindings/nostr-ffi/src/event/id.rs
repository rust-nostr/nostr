// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip21::NostrURI;
use nostr::prelude::{FromBech32, ToBech32};
use nostr::{Kind, Tag};
use uniffi::Object;

use crate::error::Result;
use crate::{PublicKey, Timestamp};

#[derive(Object)]
pub struct EventId {
    inner: nostr::EventId,
}

impl Deref for EventId {
    type Target = nostr::EventId;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr::EventId> for EventId {
    fn from(inner: nostr::EventId) -> Self {
        Self { inner }
    }
}

impl From<EventId> for nostr::EventId {
    fn from(event_id: EventId) -> Self {
        event_id.inner
    }
}

impl From<&EventId> for nostr::EventId {
    fn from(event_id: &EventId) -> Self {
        event_id.inner
    }
}

#[uniffi::export]
impl EventId {
    #[uniffi::constructor]
    pub fn new(
        pubkey: Arc<PublicKey>,
        created_at: Arc<Timestamp>,
        kind: u64,
        tags: Vec<Vec<String>>,
        content: String,
    ) -> Result<Arc<Self>> {
        let mut new_tags: Vec<Tag> = Vec::new();
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag)?);
        }
        Ok(Arc::new(Self {
            inner: nostr::EventId::new(
                pubkey.as_ref().deref(),
                *created_at.as_ref().deref(),
                &Kind::from(kind),
                &new_tags,
                &content,
            ),
        }))
    }

    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventId::from_slice(&bytes)?,
        }))
    }

    #[uniffi::constructor]
    pub fn from_hex(hex: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventId::from_hex(hex)?,
        }))
    }

    #[uniffi::constructor]
    pub fn from_bech32(id: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventId::from_bech32(id)?,
        }))
    }

    #[uniffi::constructor]
    pub fn from_nostr_uri(uri: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: nostr::EventId::from_nostr_uri(uri)?,
        }))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    pub fn to_bech32(&self) -> Result<String> {
        Ok(self.inner.to_bech32()?)
    }

    pub fn to_nostr_uri(&self) -> Result<String> {
        Ok(self.inner.to_nostr_uri()?)
    }
}
