// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::message::subscription::Alphabet;

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;
use crate::{EventId, PublicKey, Timestamp};

#[derive(Clone)]
pub struct Filter {
    inner: nostr::Filter,
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Filter {
    type Target = nostr::Filter;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr::Filter> for Filter {
    fn from(f: nostr::Filter) -> Self {
        Self { inner: f }
    }
}

impl Filter {
    pub fn new() -> Self {
        Self {
            inner: nostr::Filter::new(),
        }
    }

    pub fn id(self: Arc<Self>, id: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.id(id);
        Arc::new(builder)
    }

    pub fn ids(self: Arc<Self>, ids: Vec<String>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.ids(ids);
        Arc::new(builder)
    }

    pub fn author(self: Arc<Self>, author: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.author(author);
        Arc::new(builder)
    }

    pub fn authors(self: Arc<Self>, authors: Vec<String>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.authors(authors);
        Arc::new(builder)
    }

    pub fn kind(self: Arc<Self>, kind: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.kind(kind.into());
        Arc::new(builder)
    }

    pub fn kinds(self: Arc<Self>, kinds: Vec<u64>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder
            .inner
            .kinds(kinds.into_iter().map(|k| k.into()).collect());
        Arc::new(builder)
    }

    pub fn event(self: Arc<Self>, event_id: Arc<EventId>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.event(event_id.as_ref().into());
        Arc::new(builder)
    }

    pub fn events(self: Arc<Self>, ids: Vec<Arc<EventId>>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder
            .inner
            .events(ids.into_iter().map(|id| id.as_ref().into()).collect());
        Arc::new(builder)
    }

    pub fn pubkey(self: Arc<Self>, pubkey: Arc<PublicKey>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.pubkey(*pubkey.as_ref().deref());
        Arc::new(builder)
    }

    pub fn pubkeys(self: Arc<Self>, pubkeys: Vec<Arc<PublicKey>>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder
            .inner
            .pubkeys(pubkeys.into_iter().map(|id| *id.as_ref().deref()).collect());
        Arc::new(builder)
    }

    pub fn search(self: Arc<Self>, text: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.search(text);
        Arc::new(builder)
    }

    pub fn since(self: Arc<Self>, timestamp: Arc<Timestamp>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.since(*timestamp.as_ref().deref());
        Arc::new(builder)
    }

    pub fn until(self: Arc<Self>, timestamp: Arc<Timestamp>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.until(*timestamp.as_ref().deref());
        Arc::new(builder)
    }

    pub fn limit(self: Arc<Self>, limit: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.limit(limit as usize);
        Arc::new(builder)
    }

    pub fn custom_tag(self: Arc<Self>, tag: Alphabet, content: Vec<String>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.custom_tag(tag, content);
        Arc::new(builder)
    }

    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::Filter::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> String {
        self.inner.as_json()
    }
}
