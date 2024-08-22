// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::JsonUtil;
use uniffi::Object;

pub mod builder;
pub mod id;
pub mod kind;
pub mod raw;
pub mod tag;
pub mod unsigned;

pub use self::builder::EventBuilder;
pub use self::id::EventId;
pub use self::kind::{Kind, KindEnum};
pub use self::tag::{Tag, TagKind, TagStandard};
pub use self::unsigned::UnsignedEvent;
use crate::error::Result;
use crate::nips::nip01::Coordinate;
use crate::{PublicKey, Timestamp};

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
    #[inline]
    pub fn id(&self) -> EventId {
        self.inner.id.into()
    }

    /// Get event author (`pubkey` field)
    #[inline]
    pub fn author(&self) -> PublicKey {
        self.inner.pubkey.into()
    }

    #[inline]
    pub fn created_at(&self) -> Timestamp {
        self.inner.created_at.into()
    }

    #[inline]
    pub fn kind(&self) -> Kind {
        self.inner.kind.into()
    }

    pub fn tags(&self) -> Vec<Arc<Tag>> {
        self.inner
            .tags
            .iter()
            .cloned()
            .map(|t| Arc::new(t.into()))
            .collect()
    }

    /// Get content of **first** tag that match `TagKind`.
    pub fn get_tag_content(&self, kind: TagKind) -> Option<String> {
        self.inner
            .get_tag_content(kind.into())
            .map(|c| c.to_string())
    }

    /// Get content of all tags that match `TagKind`.
    pub fn get_tags_content(&self, kind: TagKind) -> Vec<String> {
        self.inner
            .get_tags_content(kind.into())
            .into_iter()
            .map(|c| c.to_string())
            .collect()
    }

    #[inline]
    pub fn content(&self) -> String {
        self.inner.content.to_string()
    }

    #[inline]
    pub fn signature(&self) -> String {
        self.inner.sig.to_string()
    }

    /// Verify both `EventId` and `Signature`
    #[inline]
    pub fn verify(&self) -> bool {
        // TODO: return `Result` instead?
        self.inner.verify().is_ok()
    }

    /// Verify if the `EventId` it's composed correctly
    #[inline]
    pub fn verify_id(&self) -> bool {
        self.inner.verify_id()
    }

    /// Verify only event `Signature`
    #[inline]
    pub fn verify_signature(&self) -> bool {
        self.inner.verify_signature()
    }

    /// Get `Timestamp` expiration if set
    pub fn expiration(&self) -> Option<Arc<Timestamp>> {
        self.inner.expiration().map(|t| Arc::new((*t).into()))
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }

    /// Extract identifier (`d` tag), if exists.
    #[inline]
    pub fn identifier(&self) -> Option<String> {
        self.inner.identifier().map(|i| i.to_string())
    }

    /// Extract public keys from tags (`p` tag)
    ///
    /// **This method extract ONLY supported standard variants**
    pub fn public_keys(&self) -> Vec<Arc<PublicKey>> {
        self.inner
            .public_keys()
            .copied()
            .map(|p| Arc::new(p.into()))
            .collect()
    }

    /// Extract event IDs from tags (`e` tag)
    ///
    /// **This method extract ONLY supported standard variants**
    pub fn event_ids(&self) -> Vec<Arc<EventId>> {
        self.inner
            .event_ids()
            .copied()
            .map(|p| Arc::new(p.into()))
            .collect()
    }

    /// Extract coordinates from tags (`a` tag)
    ///
    /// **This method extract ONLY supported standard variants**
    pub fn coordinates(&self) -> Vec<Arc<Coordinate>> {
        self.inner
            .coordinates()
            .cloned()
            .map(|p| Arc::new(p.into()))
            .collect()
    }

    /// Extract hashtags from tags (`t` tag)
    ///
    /// **This method extract ONLY supported standard variants**
    pub fn hashtags(&self) -> Vec<String> {
        self.inner.hashtags().map(|t| t.to_owned()).collect()
    }

    /// Check if it's a protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    pub fn is_protected(&self) -> bool {
        self.inner.is_protected()
    }

    #[inline]
    #[uniffi::constructor]
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::Event::from_json(json)?,
        })
    }

    #[inline]
    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }

    #[inline]
    pub fn as_pretty_json(&self) -> Result<String> {
        Ok(self.inner.try_as_pretty_json()?)
    }
}
