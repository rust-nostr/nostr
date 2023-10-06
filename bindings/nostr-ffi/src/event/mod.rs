// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::{Event as EventSdk, JsonUtil};

mod builder;
mod id;
mod tag;
mod unsigned;

pub use self::builder::EventBuilder;
pub use self::id::EventId;
pub use self::tag::{Tag, TagEnum, TagKind, TagKindKnown};
pub use self::unsigned::UnsignedEvent;
use crate::error::Result;
use crate::{PublicKey, Timestamp};

pub struct Event {
    inner: EventSdk,
}

impl From<EventSdk> for Event {
    fn from(inner: EventSdk) -> Self {
        Self { inner }
    }
}

impl Deref for Event {
    type Target = EventSdk;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Event {
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

    pub fn signature(&self) -> String {
        self.inner.sig.to_string()
    }

    pub fn verify(&self) -> bool {
        self.inner.verify().is_ok()
    }

    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            inner: EventSdk::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }
}
