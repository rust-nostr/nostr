// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use nostr::{KindBase, SubscriptionFilter as SubscriptionFilterSdk};
use secp256k1::XOnlyPublicKey;
use uuid::Uuid;

use crate::helper::unwrap_or_clone_arc;

#[derive(Clone)]
pub struct SubscriptionFilter {
    sub_filter: SubscriptionFilterSdk,
}

impl Default for SubscriptionFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl SubscriptionFilter {
    pub fn new() -> Self {
        Self {
            sub_filter: SubscriptionFilterSdk::new(),
        }
    }

    pub fn id(self: Arc<Self>, id: String) -> Result<Arc<Self>> {
        let id = Uuid::from_str(&id)?;

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.id(id);

        Ok(Arc::new(builder))
    }

    pub fn ids(self: Arc<Self>, ids: Vec<String>) -> Result<Arc<Self>> {
        let mut new_ids: Vec<Uuid> = Vec::with_capacity(ids.len());
        for id in ids.into_iter() {
            new_ids.push(Uuid::from_str(&id)?);
        }

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.ids(new_ids);

        Ok(Arc::new(builder))
    }

    pub fn kind_custom(self: Arc<Self>, kind_id: u16) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.kind_custom(kind_id);

        Arc::new(builder)
    }

    pub fn kind_base(self: Arc<Self>, kind_base: KindBase) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.kind_base(kind_base);

        Arc::new(builder)
    }

    pub fn event(self: Arc<Self>, event_id: String) -> Result<Arc<Self>> {
        let event_id = Uuid::from_str(&event_id)?;

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.event(event_id);

        Ok(Arc::new(builder))
    }

    pub fn pubkey(self: Arc<Self>, pubkey: String) -> Result<Arc<Self>> {
        let pubkey = XOnlyPublicKey::from_str(&pubkey)?;

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.pubkey(pubkey);

        Ok(Arc::new(builder))
    }

    // unix timestamp seconds
    pub fn since(self: Arc<Self>, timestamp: u64) -> Arc<Self> {
        let naive = NaiveDateTime::from_timestamp(timestamp as i64, 0);
        let since = DateTime::<Utc>::from_utc(naive, Utc);

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.since(since);

        Arc::new(builder)
    }

    pub fn until(self: Arc<Self>, timestamp: u64) -> Arc<Self> {
        let naive = NaiveDateTime::from_timestamp(timestamp as i64, 0);
        let until = DateTime::<Utc>::from_utc(naive, Utc);

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.until(until);

        Arc::new(builder)
    }

    pub fn authors(self: Arc<Self>, authors: Vec<String>) -> Result<Arc<Self>> {
        let mut new_authors: Vec<XOnlyPublicKey> = Vec::with_capacity(authors.len());
        for a in authors.into_iter() {
            new_authors.push(XOnlyPublicKey::from_str(&a)?);
        }

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.authors(new_authors);

        Ok(Arc::new(builder))
    }
}
