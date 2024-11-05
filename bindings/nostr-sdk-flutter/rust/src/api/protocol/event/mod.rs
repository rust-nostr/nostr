// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use anyhow::Result;
use flutter_rust_bridge::frb;
use nostr_sdk::prelude::*;

use super::key::public_key::_PublicKey;

#[frb(name = "Event")]
pub struct _Event {
    pub(crate) inner: Event,
}

impl _Event {
    pub fn id(&self) -> String {
        self.inner.id.to_hex()
    }

    /// Get event author (`pubkey` field)
    pub fn author(&self) -> _PublicKey {
        self.inner.pubkey.into()
    }

    pub fn created_at(&self) -> u64 {
        self.inner.created_at.as_u64()
    }

    pub fn kind(&self) -> u16 {
        self.inner.kind.as_u16()
    }

    pub fn tags(&self) -> Vec<Vec<String>> {
        self.inner
            .tags
            .iter()
            .map(|tag| tag.as_slice().to_vec())
            .collect()
    }

    pub fn content(&self) -> String {
        self.inner.content.to_string()
    }

    pub fn signature(&self) -> String {
        self.inner.sig.to_string()
    }

    /// Verify both `EventId` and `Signature`
    pub fn verify(&self) -> Result<()> {
        Ok(self.inner.verify()?)
    }

    /// Verify if the `EventId` it's composed correctly
    pub fn verify_id(&self) -> bool {
        self.inner.verify_id()
    }

    /// Verify only event `Signature`
    pub fn verify_signature(&self) -> bool {
        self.inner.verify_signature()
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no expiration tag, then it will return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    pub fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }

    /// Check if it's a protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    pub fn is_protected(&self) -> bool {
        self.inner.is_protected()
    }

    #[frb(sync)]
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(Self {
            inner: Event::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }

    pub fn as_pretty_json(&self) -> Result<String> {
        Ok(self.inner.try_as_pretty_json()?)
    }
}
