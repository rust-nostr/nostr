// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip21::NostrURI;
use nostr::prelude::{FromBech32, ToBech32};
use nostr::{Kind, Tag};

use crate::error::Result;
use crate::{PublicKey, Timestamp};

pub struct EventId {
    inner: nostr::EventId,
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

impl EventId {
    pub fn new(
        pubkey: Arc<PublicKey>,
        created_at: Arc<Timestamp>,
        kind: u64,
        tags: Vec<Vec<String>>,
        content: String,
    ) -> Result<Self> {
        let mut new_tags: Vec<Tag> = Vec::new();
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag)?);
        }
        Ok(Self {
            inner: nostr::EventId::new(
                pubkey.as_ref().deref(),
                *created_at.as_ref().deref(),
                &Kind::from(kind),
                &new_tags,
                &content,
            ),
        })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventId::from_slice(&bytes)?,
        })
    }

    pub fn from_hex(hex: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventId::from_hex(hex)?,
        })
    }

    pub fn from_bech32(id: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventId::from_bech32(id)?,
        })
    }

    pub fn from_nostr_uri(uri: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventId::from_nostr_uri(uri)?,
        })
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
