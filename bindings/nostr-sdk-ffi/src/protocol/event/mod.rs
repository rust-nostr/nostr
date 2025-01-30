// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::JsonUtil;
use uniffi::Object;

pub mod builder;
pub mod id;
pub mod kind;
pub mod tag;
pub mod unsigned;

pub use self::builder::EventBuilder;
pub use self::id::EventId;
pub use self::kind::Kind;
pub use self::tag::{Tag, TagKind, TagStandard, Tags};
pub use self::unsigned::UnsignedEvent;
use crate::error::Result;
use crate::protocol::key::PublicKey;
use crate::protocol::types::Timestamp;

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Event {
    inner: nostr::Event,
}

impl From<nostr::Event> for Event {
    fn from(inner: nostr::Event) -> Self {
        Self { inner }
    }
}

impl Deref for Event {
    type Target = nostr::Event;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Event {
    pub fn id(&self) -> EventId {
        self.inner.id.into()
    }

    /// Get event author (`pubkey` field)
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
        self.inner.content.to_string()
    }

    pub fn signature(&self) -> String {
        self.inner.sig.to_string()
    }

    /// Verify both `EventId` and `Signature`
    pub fn verify(&self) -> bool {
        // TODO: return `Result` instead?
        self.inner.verify().is_ok()
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

    #[uniffi::constructor]
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::Event::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }

    pub fn as_pretty_json(&self) -> Result<String> {
        Ok(self.inner.try_as_pretty_json()?)
    }
}
