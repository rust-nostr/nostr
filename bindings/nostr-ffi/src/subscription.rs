// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Sha256Hash, SubscriptionFilter as SubscriptionFilterSdk};

use crate::error::Result;
use crate::event::kind::Kind;
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

impl Deref for SubscriptionFilter {
    type Target = SubscriptionFilterSdk;
    fn deref(&self) -> &Self::Target {
        &self.sub_filter
    }
}

impl From<SubscriptionFilterSdk> for SubscriptionFilter {
    fn from(f: SubscriptionFilterSdk) -> Self {
        Self { sub_filter: f }
    }
}

impl SubscriptionFilter {
    pub fn new() -> Self {
        Self {
            sub_filter: SubscriptionFilterSdk::new(),
        }
    }

    pub fn id(self: Arc<Self>, id: String) -> Result<Arc<Self>> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.id(id);

        Ok(Arc::new(builder))
    }

    pub fn ids(self: Arc<Self>, ids: Vec<String>) -> Result<Arc<Self>> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.ids(ids);

        Ok(Arc::new(builder))
    }

    pub fn kind(self: Arc<Self>, kind: Kind) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.kind(kind.into());

        Arc::new(builder)
    }

    pub fn event(self: Arc<Self>, event_id: String) -> Result<Arc<Self>> {
        let event_id = Sha256Hash::from_str(&event_id)?;

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.events(vec![event_id]);

        Ok(Arc::new(builder))
    }

    pub fn pubkey(self: Arc<Self>, pubkey: String) -> Result<Arc<Self>> {
        let pubkey = XOnlyPublicKey::from_str(&pubkey)?;

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.pubkey(pubkey);

        Ok(Arc::new(builder))
    }

    pub fn since(self: Arc<Self>, timestamp: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.since(timestamp);

        Arc::new(builder)
    }

    pub fn until(self: Arc<Self>, timestamp: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.until(timestamp);

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
