// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip21::NostrURI;
use nostr::prelude::{FromBech32, ToBech32};
use uniffi::Object;

use super::Kind;
use crate::error::Result;
use crate::{PublicKey, Tag, Timestamp};

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

#[uniffi::export]
impl EventId {
    #[uniffi::constructor]
    pub fn new(
        public_key: &PublicKey,
        created_at: &Timestamp,
        kind: &Kind,
        tags: &[Arc<Tag>],
        content: &str,
    ) -> Result<Self> {
        let tags: Vec<nostr::Tag> = tags.iter().map(|t| t.as_ref().deref().clone()).collect();
        Ok(Self {
            inner: nostr::EventId::new(
                public_key.deref(),
                **created_at,
                kind.deref(),
                &tags,
                content,
            ),
        })
    }

    /// Try to parse event ID from `hex`, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    #[uniffi::constructor]
    pub fn parse(id: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventId::parse(id)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventId::from_slice(bytes)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_hex(hex: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventId::from_hex(hex)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_bech32(bech32: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::EventId::from_bech32(bech32)?,
        })
    }

    #[uniffi::constructor]
    pub fn from_nostr_uri(uri: &str) -> Result<Self> {
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
