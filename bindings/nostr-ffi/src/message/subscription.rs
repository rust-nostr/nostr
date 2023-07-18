// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::Filter as FilterSdk;

use crate::helper::unwrap_or_clone_arc;
use crate::{EventId, PublicKey, Timestamp};

#[derive(Clone)]
pub struct Filter {
    sub_filter: FilterSdk,
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Filter {
    type Target = FilterSdk;
    fn deref(&self) -> &Self::Target {
        &self.sub_filter
    }
}

impl From<FilterSdk> for Filter {
    fn from(f: FilterSdk) -> Self {
        Self { sub_filter: f }
    }
}

impl Filter {
    pub fn new() -> Self {
        Self {
            sub_filter: FilterSdk::new(),
        }
    }

    pub fn id(self: Arc<Self>, id: String) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.id(id);
        Arc::new(builder)
    }

    pub fn ids(self: Arc<Self>, ids: Vec<String>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.ids(ids);
        Arc::new(builder)
    }

    pub fn kind(self: Arc<Self>, kind: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.kind(kind.into());
        Arc::new(builder)
    }

    pub fn event(self: Arc<Self>, event_id: Arc<EventId>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.event(event_id.as_ref().into());
        Arc::new(builder)
    }

    pub fn events(self: Arc<Self>, ids: Vec<Arc<EventId>>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder
            .sub_filter
            .events(ids.into_iter().map(|id| id.as_ref().into()).collect());
        Arc::new(builder)
    }

    pub fn pubkey(self: Arc<Self>, pubkey: Arc<PublicKey>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.pubkey(*pubkey.as_ref().deref());
        Arc::new(builder)
    }

    pub fn pubkeys(self: Arc<Self>, pubkeys: Vec<Arc<PublicKey>>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder
            .sub_filter
            .pubkeys(pubkeys.into_iter().map(|id| *id.as_ref().deref()).collect());
        Arc::new(builder)
    }

    pub fn since(self: Arc<Self>, timestamp: Arc<Timestamp>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.since(*timestamp.as_ref().deref());
        Arc::new(builder)
    }

    pub fn until(self: Arc<Self>, timestamp: Arc<Timestamp>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.until(*timestamp.as_ref().deref());
        Arc::new(builder)
    }

    pub fn authors(self: Arc<Self>, authors: Vec<String>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.authors(authors);
        Arc::new(builder)
    }
}
